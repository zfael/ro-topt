# ro-topt

A Rust application with a GUI for generating TOTP (Time-based One-Time Password) codes.

## Features

- Enter a secret key and generate TOTP codes
- Customize the number of digits (4-8)
- Customize the token period (15-60 seconds)
- Real-time countdown timer showing when the token will expire
- Automatic token regeneration when expired

## Continuous Integration

This project uses GitHub Actions for continuous integration and release automation:

- Automatic versioning: Each push to the main branch increments the patch version.
- Cross-platform builds: Executables are built for:
  - Linux (x86_64)
  - Windows (x86_64)
  - macOS (Intel x86_64 and Apple Silicon ARM64)
- GitHub Releases: Built binaries are automatically attached to GitHub releases.

### Release Process

When code is pushed to the main branch:
1. A new tag is created, incrementing the patch version.
2. A GitHub release is created with the new tag.
3. The application is built for all supported platforms.
4. Built executables are attached to the release.

You can also manually trigger a release from the GitHub Actions tab.

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