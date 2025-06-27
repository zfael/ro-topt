# ro-topt

A Rust application with a GUI for generating TOTP (Time-based One-Time Password) codes.

## Features

- Enter a secret key and generate TOTP codes
- Customize the number of digits (4-8)
- Customize the token period (15-60 seconds)
- Real-time countdown timer showing when the token will expire
- Automatic token regeneration when expired

## Usage

1. Enter your secret key in the input field
   - Example key: `JBSWY3DPEHPK3PXP`
2. Adjust the number of digits (default: 6)
3. Adjust the token period in seconds (default: 30)
4. Click "Generate TOTP" to create your code
5. The code will automatically refresh when it expires

## Building and Running

```bash
# Clone the repository
git clone https://github.com/zfael/ro-topt.git
cd ro-topt

# Build and run
cargo run --release
```

## Dependencies

- [iced](https://github.com/iced-rs/iced) - A cross-platform GUI library for Rust
- [totp-rs](https://github.com/constantoine/totp-rs) - TOTP implementation for Rust
- [base32](https://github.com/andreasots/base32-rs) - Base32 encoding/decoding
- [chrono](https://github.com/chronotope/chrono) - Date and time library for Rust