# Installation Guide

This guide provides detailed installation instructions for Memscan on all supported platforms.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Platform-Specific Setup](#platform-specific-setup)
  - [Linux](#linux)
  - [macOS](#macos)
  - [Windows](#windows)
- [Building from Source](#building-from-source)
- [Installation Methods](#installation-methods)
- [Post-Installation Setup](#post-installation-setup)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required
- **Rust 1.70+** with Cargo
- **Git** for cloning the repository

### Install Rust
```bash
# Install Rust via rustup (all platforms)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

## Platform-Specific Setup

### Linux

#### Ubuntu/Debian
```bash
# Update package lists
sudo apt update

# Install dependencies
sudo apt install -y \
    build-essential \
    pkg-config

# For process memory access
sudo apt install -y libc6-dev
```

#### Arch Linux
```bash
# Install dependencies
sudo pacman -Syu
sudo pacman -S \
    base-devel \
    pkgconf
```

#### Fedora/RHEL/CentOS
```bash
# Install dependencies
sudo dnf groupinstall "Development Tools"
sudo dnf install -y pkgconfig
```

#### Alpine Linux
```bash
# Install dependencies
sudo apk add \
    build-base \
    rust \
    cargo \
    pkgconfig
```

### macOS

#### Using Homebrew (Recommended)
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install pkg-config

# Install Xcode command line tools
xcode-select --install
```

#### Using MacPorts
```bash
# Install MacPorts dependencies
sudo port install pkgconfig
```

### Windows

#### Visual Studio (Recommended)
```bash
# Install Visual Studio Build Tools 2019 or later
# Download from: https://visualstudio.microsoft.com/downloads/

# Required components:
# - MSVC v143 compiler toolset
# - Windows 10/11 SDK
# - CMake tools for Visual Studio
```

#### MSYS2/MinGW-w64
```bash
# Install MSYS2 from https://www.msys2.org/

# In MSYS2 terminal:
pacman -S \
    mingw-w64-x86_64-gcc \
    mingw-w64-x86_64-pkg-config

# Add to PATH: C:\msys64\mingw64\bin
```

## Building from Source

### 1. Clone Repository
```bash
git clone https://github.com/yourusername/memscan.git
cd memscan
```

### 2. Build Options

#### Development Build (Fast compilation)
```bash
cargo build
./target/debug/memscan-cli
```

#### Release Build (Optimized)
```bash
cargo build --release
./target/release/memscan-cli
```

#### Minimal Size Build (Ultra-optimized)
```bash
cargo build --profile release-small
./target/release-small/memscan-cli
```

### 3. Run Tests
```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_process_enumeration
```

## Installation Methods

### Method 1: Direct Binary Usage
```bash
# After building
cp target/release/memscan-cli ~/.local/bin/
# Or add target/release to your PATH
```

### Method 2: Cargo Install (Future)
```bash
# Will be available when published to crates.io
cargo install memscan-cli
```

### Method 3: Package Managers (Future)

#### Linux (APT)
```bash
# Future Ubuntu/Debian package
sudo apt install memscan-cli
```

#### macOS (Homebrew)
```bash
# Future Homebrew formula
brew install memscan-cli
```

#### Windows (Scoop)
```bash
# Future Scoop package
scoop install memscan-cli
```

## Post-Installation Setup

### Linux Permissions

#### Sudo Access
```bash
# Run with sudo when needed
sudo ./target/release/memscan-cli
```

### macOS Permissions
```bash
# Grant Terminal/iTerm debugging permissions in:
# System Preferences > Security & Privacy > Privacy > Developer Tools
# Add Terminal/iTerm to the list

# For System Integrity Protection (SIP) restrictions:
# Disable SIP temporarily (not recommended for production)
# Boot into Recovery Mode and run:
# csrutil disable
```

### Windows Permissions
```bash
# Run as Administrator for process access
# Right-click memscan-cli.exe -> "Run as administrator"

# Or add to Windows Defender exclusions if antivirus blocks it
```

## Verification

### Test Installation
```bash
# Check version
./target/release/memscan-cli --version

# List available options
./target/release/memscan-cli --help

# Test process enumeration (should show running processes)
./target/release/memscan-cli list
```

### Performance Test
```bash
# Run with memory monitoring
RUST_LOG=debug ./target/release/memscan-cli

# Check memory usage stays under 100MB
ps aux | grep memscan-cli
```

## Troubleshooting

### Build Issues

#### Missing Dependencies
```bash
# Error: "pkg-config not found"
# Solution: Install pkg-config

# Linux (Ubuntu/Debian)
sudo apt install pkg-config

# macOS
brew install pkg-config
```

#### Linker Errors
```bash
# Error: linker issues
# Solution: Install build essentials

# Linux (Ubuntu/Debian)
sudo apt install build-essential
# macOS
xcode-select --install
```

#### Missing C++ Compiler
```bash
# Error: "C++ compiler not found"
# Linux
sudo apt install build-essential
# macOS
xcode-select --install
# Windows
# Install Visual Studio Build Tools
```

#### Cross-compilation Issues
```bash
# Install target for cross-compilation
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu

# Cross-compile example (Linux to Windows)
cargo build --target x86_64-pc-windows-gnu --release
```

### Runtime Issues

#### Permission Denied Errors
```bash
# Linux: Process attachment fails
# Solution 1: Use capabilities
sudo setcap cap_sys_ptrace=eip ./target/release/memscan-cli

# Solution 2: Run as root
sudo ./target/release/memscan-cli

# Solution 3: Adjust ptrace_scope
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope

# macOS: Operation not permitted
# Solution: Add to Developer Tools in Security & Privacy
```

#### Terminal Display Issues
```bash
# Error: Terminal display problems
# Ensure terminal supports UTF-8
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
```

#### Memory Issues
```bash
# Error: Memory limit exceeded
# Check current usage
ps -o pid,ppid,cmd,%mem,%cpu --sort=-%mem

# Verify memory optimization is working
RUST_LOG=trace ./target/release/memscan-cli 2>&1 | grep -i memory
```

### Dependency Issues

#### Outdated Rust Version
```bash
# Update Rust
rustup update stable
rustup default stable

# Check version
rustc --version  # Should be 1.70+
```

#### Missing System Libraries
```bash
# Linux: Check for missing libraries
ldd target/release/memscan-cli

# Install missing libraries if needed
# Usually minimal dependencies for CLI version
```

#### Conflicting Dependencies
```bash
# Clean build artifacts
cargo clean

# Remove lock file and rebuild
rm Cargo.lock
cargo build --release

# Update dependencies
cargo update
```

### Platform-Specific Issues

#### Linux-Specific
```bash
# systemd service restrictions
sudo systemctl edit user@.service
# Add: Environment="PATH=/usr/local/bin:/usr/bin:/bin"

# SELinux issues (Fedora/RHEL)
setsebool -P allow_ptrace 1
```

#### macOS-Specific
```bash
# Code signing issues
codesign --force --deep --sign - target/release/memscan-cli

# Gatekeeper bypass for development
sudo spctl --master-disable

# Xcode license agreement
sudo xcodebuild -license accept
```

#### Windows-Specific
```bash
# Windows Defender SmartScreen
# Add exclusion or click "More info" -> "Run anyway"

# MSVC not found
# Install "C++ Build Tools" workload in Visual Studio Installer

# Path issues
# Add C:\Users\%USERNAME%\.cargo\bin to PATH
```

## Advanced Configuration

### Environment Variables
```bash
# Rust compilation flags
export RUSTFLAGS="-C target-cpu=native"

# Memory optimization
export MEMSCAN_MAX_MEMORY=50  # Limit to 50MB

# Debug logging
export RUST_LOG=memscan=debug

# Cross-compilation
export PKG_CONFIG_ALLOW_CROSS=1
```

### Configuration File
```toml
# ~/.config/memscan/config.toml
[memory]
max_usage_mb = 100
track_allocations = true

[scanning]
default_scan_type = "i32"
max_results = 10000
```

### Build Optimizations
```bash
# Link-time optimization
export CARGO_PROFILE_RELEASE_LTO=true

# Minimal binary size
export CARGO_PROFILE_RELEASE_OPT_LEVEL="z"
export CARGO_PROFILE_RELEASE_STRIP=true

# Parallel compilation
export CARGO_BUILD_JOBS=8
```

## Getting Help

### Documentation
- [README.md](README.md) - Project overview
- [API Documentation](https://docs.rs/memscan) - Code documentation
- [GitHub Wiki](https://github.com/yourusername/memscan/wiki) - Additional guides

### Community Support
- [GitHub Issues](https://github.com/yourusername/memscan/issues) - Bug reports
- [GitHub Discussions](https://github.com/yourusername/memscan/discussions) - Questions
- [Discord Server](https://discord.gg/memscan) - Real-time chat

### Debugging Information
When reporting issues, please include:
```bash
# System information
uname -a
rustc --version
cargo --version

# Build information
cargo build --verbose 2>&1 | tee build.log

# Runtime information
RUST_BACKTRACE=full RUST_LOG=debug ./target/release/memscan-cli 2>&1 | tee runtime.log
```

---

**Installation complete!** 🎉

For usage instructions, see the main [README.md](README.md).

Built by Danielscos 🐐 - The GOAT of memory scanning
