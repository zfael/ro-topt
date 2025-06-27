use iced::{
    executor, theme, time,
    widget::{button, column, container, row, text, text_input, vertical_space},
    Alignment, Application, Command, Element, Length, Settings, Subscription, Theme,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, TOTP};
use base32;
use clipboard::ClipboardProvider;

fn main() -> iced::Result {
    TotpGenerator::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    SecretKeyChanged(String, usize), // Added tab index parameter
    DigitsChanged(u8),
    PeriodChanged(u64),
    GenerateToken, // Kept for backward compatibility
    CopyToClipboard(usize), // Added tab index parameter
    Tick,
    ClearMessage(usize), // Added tab index parameter
    AddTab,
    RemoveTab(usize),
    SelectTab(usize),
    RenameTabStarted(usize),
    TabNameChanged(String, usize),
    TabNameConfirmed(usize),
}

#[derive(Debug, Clone)]
struct Tab {
    name: String,
    secret_key: String,
    token: String,
    error: Option<String>,
    time_remaining: u64,
    editing_name: bool,
}

impl Default for Tab {
    fn default() -> Self {
        Self {
            name: String::from("New Tab"),
            secret_key: String::new(),
            token: String::new(),
            error: None,
            time_remaining: 30,
            editing_name: true,
        }
    }
}

struct TotpGenerator {
    tabs: Vec<Tab>,
    active_tab: usize,
    digits: u8,
    period: u64,
}

impl Default for TotpGenerator {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::default()],
            active_tab: 0,
            digits: 6,
            period: 30,
        }
    }
}

impl Application for TotpGenerator {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("TOTP Token Generator")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SecretKeyChanged(value, tab_index) => {
                if tab_index < self.tabs.len() {
                    let tab = &mut self.tabs[tab_index];
                    tab.secret_key = value;
                    tab.error = None;
                    
                    // Generate token automatically if secret key is not empty
                    if !tab.secret_key.is_empty() {
                        self.generate_token(tab_index);
                    } else {
                        tab.token = String::new();
                    }
                }
            }
            Message::DigitsChanged(_) => {
                // We keep the default value of 6 digits
                self.digits = 6;
            }
            Message::PeriodChanged(_) => {
                // We keep the default value of 30 seconds
                self.period = 30;
            }
            Message::GenerateToken => {
                // For backward compatibility - uses active tab
                self.generate_token(self.active_tab);
            }
            Message::CopyToClipboard(tab_index) => {
                if tab_index < self.tabs.len() && !self.tabs[tab_index].token.is_empty() {
                    let token = self.tabs[tab_index].token.clone();
                    let mut ctx: clipboard::ClipboardContext = match ClipboardProvider::new() {
                        Ok(ctx) => ctx,
                        Err(e) => {
                            self.tabs[tab_index].error = Some(format!("Failed to access clipboard: {}", e));
                            return Command::none();
                        }
                    };
                    
                    if let Err(e) = ctx.set_contents(token.replace(" ", "")) {
                        self.tabs[tab_index].error = Some(format!("Failed to copy to clipboard: {}", e));
                    } else {
                        self.tabs[tab_index].error = Some("Code copied to clipboard!".to_string());
                        // Clear the message after 3 seconds
                        return Command::perform(
                            async move {
                                std::thread::sleep(std::time::Duration::from_secs(3));
                                tab_index
                            },
                            |idx| Message::ClearMessage(idx),
                        );
                    }
                }
            }
            Message::ClearMessage(tab_index) => {
                // Clear any success/error message for the specified tab
                if tab_index < self.tabs.len() {
                    self.tabs[tab_index].error = None;
                }
            }
            Message::Tick => {
                // Update time remaining for all tabs
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                
                // Collect indices that need regeneration
                let mut indices_to_regenerate = Vec::new();
                
                // First pass: update time remaining
                for (idx, tab) in self.tabs.iter_mut().enumerate() {
                    if !tab.token.is_empty() {
                        tab.time_remaining = self.period - (now % self.period);
                        
                        // Mark for token regeneration when time expires
                        if tab.time_remaining == self.period {
                            indices_to_regenerate.push(idx);
                        }
                    }
                }
                
                // Second pass: regenerate tokens for expired tabs
                for idx in indices_to_regenerate {
                    self.generate_token(idx);
                }
            }
            Message::AddTab => {
                // Create a new tab with default values and add it to the list
                let new_tab = Tab {
                    name: format!("Tab {}", self.tabs.len() + 1),
                    ..Default::default()
                };
                self.tabs.push(new_tab);
                self.active_tab = self.tabs.len() - 1;
            }
            Message::RemoveTab(idx) => {
                if self.tabs.len() > 1 && idx < self.tabs.len() {
                    self.tabs.remove(idx);
                    // Adjust active_tab if necessary
                    if self.active_tab >= self.tabs.len() {
                        self.active_tab = self.tabs.len() - 1;
                    }
                }
            }
            Message::SelectTab(idx) => {
                if idx < self.tabs.len() {
                    self.active_tab = idx;
                }
            }
            Message::RenameTabStarted(idx) => {
                if idx < self.tabs.len() {
                    self.tabs[idx].editing_name = true;
                }
            }
            Message::TabNameChanged(name, idx) => {
                if idx < self.tabs.len() {
                    self.tabs[idx].name = name;
                }
            }
            Message::TabNameConfirmed(idx) => {
                if idx < self.tabs.len() {
                    self.tabs[idx].editing_name = false;
                }
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_secs(1))
            .map(|_| Message::Tick)
    }

    fn view(&self) -> Element<Message> {
        // Title with improved styling
        let title = container(
            text("TOTP Token Generator")
                .size(30)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.1, 0.1, 0.1)))
        )
        .width(Length::Fill)
        .center_x()
        .padding([0, 0, 10, 0]);

        // Create the tab bar with a bottom border
        let mut tab_row = row![].spacing(2).padding([5, 5, 0, 5]);
        
        // Add tabs
        for (idx, tab) in self.tabs.iter().enumerate() {
            let is_active = idx == self.active_tab;
            
            // Create content for the tab
            let tab_content = if tab.editing_name {
                // Show text input for rename with a save button
                let tab_name_input = text_input("Tab name", &tab.name)
                    .on_input(move |name| Message::TabNameChanged(name, idx))
                    .on_submit(Message::TabNameConfirmed(idx))
                    .width(Length::Fixed(100.0));
                
                container(
                    row![
                        tab_name_input,
                    ].spacing(5)
                )
                .padding(5)
            } else {
                // Show tab name with styling
                container(text(&tab.name).size(14))
            };
            
            // Use button for the tab instead of container
            let tab_button = button(tab_content)
                .padding(8)
                .style(if is_active {
                    theme::Button::Custom(Box::new(ActiveTabButtonStyle))
                } else {
                    theme::Button::Custom(Box::new(InactiveTabButtonStyle))
                })
                .on_press(Message::SelectTab(idx));
                
            // For the editing tab, we just use the tab_button directly
            // For non-editing tabs, we want double-click to trigger rename
            let tab_with_rename = match (tab.editing_name, is_active) {
                (true, _) => tab_button, // In edit mode, just use the button as-is
                (false, true) => {
                    // For active tab, allow double click to rename
                    button(tab_button)
                        .padding(0)
                        .style(theme::Button::Text)
                        .on_press(Message::RenameTabStarted(idx))
                }
                (false, false) => {
                    // For inactive tabs, clicking just selects them
                    tab_button
                }
            };
            
            // Only add X button if we have more than one tab
            let tab_with_close_button = if self.tabs.len() > 1 {
                row![
                    tab_with_rename,
                    button(text("×").size(14))
                        .on_press(Message::RemoveTab(idx))
                        .padding(5)
                        .style(theme::Button::Destructive)
                ]
                .align_items(Alignment::Center)
                .spacing(5)
            } else {
                row![tab_with_rename]
            };
            
            tab_row = tab_row.push(tab_with_close_button);
        }
        
        // Add "+" button to create new tab
        let add_tab_button = button(
            text("+")
                .size(20)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.0, 0.5, 0.0)))
        )
        .on_press(Message::AddTab)
        .padding(5)
        .style(theme::Button::Secondary);
        
        tab_row = tab_row.push(add_tab_button);
        
        // Add a horizontal separator line below the tabs
        let tab_separator = container(
            iced::widget::horizontal_rule(1)
                .style(theme::Rule::Default)
        )
        .width(Length::Fill);
        
        // Get the currently active tab
        let active_tab = &self.tabs[self.active_tab];
        
        // Secret Key Input with placeholder text
        let secret_key_input = text_input("Enter your secret key", &active_tab.secret_key)
            .padding(12)
            .size(16)
            .style(theme::TextInput::Default)
            .on_input(|value| Message::SecretKeyChanged(value, self.active_tab));

        // Progress Bar for Countdown
        let progress_percentage = if !active_tab.token.is_empty() {
            (active_tab.time_remaining as f32) / (self.period as f32)
        } else {
            0.0
        };

        // Improved progress bar with better visibility
        let progress_bar = iced::widget::progress_bar(0.0..=1.0, progress_percentage)
            .height(iced::Length::Fixed(6.0))  // Slightly taller for better visibility
            .width(Length::Fill)
            .style(theme::ProgressBar::Primary);

        let timer_text = if !active_tab.token.is_empty() {
            text(format!("Code expires in {} seconds", active_tab.time_remaining))
                .size(14)
                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.3, 0.3, 0.3)))
        } else {
            text("").size(14)
        };

        // Token Output
        let token_display = if !active_tab.token.is_empty() {
            // Format the token with spaces for better readability
            // e.g., "123456" becomes "123 456" if 6 digits
            let formatted_token = if active_tab.token.len() == 6 {
                format!("{} {}", &active_tab.token[..3], &active_tab.token[3..])
            } else if active_tab.token.len() == 8 {
                format!("{} {}", &active_tab.token[..4], &active_tab.token[4..])
            } else {
                active_tab.token.clone()
            };

            let token_container = container(
                text(&formatted_token)
                    .size(48)
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.2, 0.2)))
            )
            .width(Length::Fill)
            .padding(25)
            .center_x();

            // Regular button with blue background for copy functionality
            let copy_button = button(
                text("Copy")
                    .size(16)
                    .style(iced::theme::Text::Color(iced::Color::WHITE))
            )
            .padding(10)
            .style(theme::Button::Custom(Box::new(BlueButtonStyle)))
            .on_press(Message::CopyToClipboard(self.active_tab));

            row![
                token_container,
                copy_button
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        } else {
            row![container(text("").size(0)).width(Length::Fill)]
        };

        // Error or success message with improved styling
        let message_display = if let Some(error) = &active_tab.error {
            // Determine if this is actually a success message
            let (message, color, icon) = if error.contains("copied to clipboard") {
                (error.as_str(), iced::Color::from_rgb(0.0, 0.5, 0.0), "✓ ") // Green for success with checkmark
            } else {
                (error.as_str(), iced::Color::from_rgb(0.8, 0.0, 0.0), "⚠ ") // Red for error with warning icon
            };
            
            let styled_message = container(
                text(format!("{}{}", icon, message))
                    .size(14)
                    .style(iced::theme::Text::Color(color))
            )
            .padding([8, 12, 8, 12])
            .style(if error.contains("copied to clipboard") {
                theme::Container::Custom(Box::new(SuccessMessageStyle))
            } else {
                theme::Container::Custom(Box::new(ErrorMessageStyle))
            });
            
            styled_message
        } else {
            container(text("").size(0))
        };

        // Simplified section without the label
        let secret_key_section = container(secret_key_input)
            .width(Length::Fill);
        
        let content = column![
            title,
            tab_row,
            vertical_space(10),
            tab_separator,
            vertical_space(10),
            secret_key_section,
            vertical_space(30),  // Increased space before timer
            token_display,
            vertical_space(20),  // Consistent spacing
            timer_text,
            vertical_space(5),
            progress_bar,
            vertical_space(20),  // More space for messages
            message_display
        ]
        .spacing(0)
        .padding(30)  // Increased padding for better spacing
        .max_width(500)  // Slightly reduced for a more compact look
        .align_items(Alignment::Center);  // Center-align everything
        
        // Make the entire application use the light gray background
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(theme::Container::Box)
            .into()
    }
}

impl TotpGenerator {
    // Helper function to decode secret keys
    fn decode_secret(input: &str) -> Vec<u8> {
        // Normalize the input: remove spaces and convert to uppercase
        let normalized = input.to_uppercase().replace(" ", "");
        
        // Characters that are valid in Base32 encoding (RFC4648)
        const BASE32_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        
        // First, try the normalized input directly
        if let Some(decoded) = base32::decode(
            base32::Alphabet::RFC4648 { padding: false },
            &normalized
        ) {
            if decoded.len() >= 16 {
                return decoded;
            } else {
                return Self::pad_key(decoded);
            }
        }
        
        // Try with padding added
        let mut padded = normalized.clone();
        while padded.len() % 8 != 0 {
            padded.push('=');
        }
        
        if let Some(decoded) = base32::decode(
            base32::Alphabet::RFC4648 { padding: true },
            &padded
        ) {
            if decoded.len() >= 16 {
                return decoded;
            } else {
                return Self::pad_key(decoded);
            }
        }
        
        // Try filtering out invalid characters
        let filtered: String = normalized.chars()
            .filter(|c| BASE32_CHARS.contains(*c))
            .collect();
            
        if filtered != normalized {
            // Try the filtered string
            if let Some(decoded) = base32::decode(
                base32::Alphabet::RFC4648 { padding: false },
                &filtered
            ) {
                if decoded.len() >= 16 {
                    return decoded;
                } else {
                    return Self::pad_key(decoded);
                }
            }
            
            // Try the filtered string with padding
            let mut padded_filtered = filtered.clone();
            while padded_filtered.len() % 8 != 0 {
                padded_filtered.push('=');
            }
            
            if let Some(decoded) = base32::decode(
                base32::Alphabet::RFC4648 { padding: true },
                &padded_filtered
            ) {
                if decoded.len() >= 16 {
                    return decoded;
                } else {
                    return Self::pad_key(decoded);
                }
            }
        }
        
        // Handle the case where 'I' might be confused with '1' or 'L', and 'O' with '0'
        let substituted = normalized
            .replace('1', "I")
            .replace('0', "O")
            .replace('8', "B")
            .replace('L', "I");
            
        if substituted != normalized {
            if let Some(decoded) = base32::decode(
                base32::Alphabet::RFC4648 { padding: false },
                &substituted
            ) {
                if decoded.len() >= 16 {
                    return decoded;
                } else {
                    return Self::pad_key(decoded);
                }
            }
        }
        
        // Last resort - use the raw bytes and extend if needed
        let raw_bytes = normalized.as_bytes().to_vec();
        if raw_bytes.len() >= 16 {
            raw_bytes
        } else {
            Self::pad_key(raw_bytes)
        }
    }
    
    // Helper function to pad a key to at least 16 bytes (128 bits)
    fn pad_key(key: Vec<u8>) -> Vec<u8> {
        // If the key is too short, extend it with zeros
        // This matches how standard TOTP implementations handle short keys
        if key.len() < 16 {
            let mut padded = key.clone();
            padded.resize(16, 0); // Zero-pad to 16 bytes
            return padded;
        }
        key
    }
    
    fn generate_token(&mut self, tab_index: usize) {
        if tab_index >= self.tabs.len() {
            return;
        }
        
        let tab = &mut self.tabs[tab_index];
        
        if tab.secret_key.is_empty() {
            tab.error = Some("Please enter a secret key".to_string());
            tab.token = String::new();
            return;
        }

        // Get current time
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Decode the key
        let decoded_key = Self::decode_secret(&tab.secret_key);
        
        // Create the TOTP with the decoded key
        match TOTP::new(
            Algorithm::SHA1,
            self.digits as usize,
            1,
            self.period,
            decoded_key,
        ) {
            Ok(totp) => {
                match totp.generate_current() {
                    Ok(token) => {
                        tab.token = token;
                        tab.error = None;
                        
                        // Update time remaining
                        tab.time_remaining = self.period - (now % self.period);
                    }
                    Err(e) => {
                        tab.error = Some(format!("Failed to generate token: {}", e));
                        tab.token = String::new();
                    }
                }
            }
            Err(e) => {
                tab.error = Some(format!("Invalid secret key: {}", e));
                tab.token = String::new();
            }
        }
    }
}

// Custom styles for message containers, buttons and tabs
struct SuccessMessageStyle;
struct ErrorMessageStyle;
struct BlueButtonStyle;
struct ActiveTabButtonStyle;
struct InactiveTabButtonStyle;

impl iced::widget::container::StyleSheet for SuccessMessageStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.5, 0.0, 0.1))),
            border_radius: 4.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.0, 0.5, 0.0),
            ..Default::default()
        }
    }
}

impl iced::widget::container::StyleSheet for ErrorMessageStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.8, 0.0, 0.0, 0.1))),
            border_radius: 4.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.8, 0.0, 0.0),
            ..Default::default()
        }
    }
}

impl iced::widget::button::StyleSheet for BlueButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.0, 0.5, 0.9))),
            border_radius: 4.0,
            text_color: iced::Color::WHITE,
            ..Default::default()
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let active = self.active(style);
        
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.1, 0.6, 1.0))),
            ..active
        }
    }
}

impl iced::widget::button::StyleSheet for ActiveTabButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.95, 0.95, 0.95))),
            border_radius: 6.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.7, 0.7, 0.7),
            shadow_offset: iced::Vector::new(0.0, 0.0),
            text_color: iced::Color::from_rgb(0.1, 0.1, 0.1),
            ..Default::default()
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let active = self.active(style);
        
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(1.0, 1.0, 1.0))),
            ..active
        }
    }
}

impl iced::widget::button::StyleSheet for InactiveTabButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.8, 0.8, 0.8))),
            border_radius: 6.0,
            border_width: 1.0,
            border_color: iced::Color::from_rgb(0.6, 0.6, 0.6),
            shadow_offset: iced::Vector::new(0.0, 0.0),
            text_color: iced::Color::from_rgb(0.4, 0.4, 0.4),
            ..Default::default()
        }
    }
    
    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let active = self.active(style);
        
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.85, 0.85, 0.85))),
            text_color: iced::Color::from_rgb(0.2, 0.2, 0.2),
            ..active
        }
    }
}
