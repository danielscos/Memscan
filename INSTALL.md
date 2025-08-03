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
- **C++ compiler** (for FLTK compilation)

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
    libfltk1.3-dev \
    libx11-dev \
    libxext-dev \
    libxft-dev \
    libxinerama-dev \
    libxcursor-dev \
    libxrender-dev \
    libxfixes-dev \
    libpng-dev \
    libjpeg-dev \
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
    fltk \
    libx11 \
    libxext \
    libxft \
    libxinerama \
    libxcursor \
    libxrender \
    libxfixes \
    libpng \
    libjpeg-turbo

# Development tools
sudo pacman -S pkgconf
```

#### Fedora/RHEL/CentOS
```bash
# Install dependencies
sudo dnf groupinstall "Development Tools"
sudo dnf install -y \
    fltk-devel \
    libX11-devel \
    libXext-devel \
    libXft-devel \
    libXinerama-devel \
    libXcursor-devel \
    libXrender-devel \
    libXfixes-devel \
    libpng-devel \
    libjpeg-turbo-devel \
    pkgconfig
```

#### Alpine Linux
```bash
# Install dependencies
sudo apk add \
    build-base \
    rust \
    cargo \
    fltk-dev \
    libx11-dev \
    libxext-dev \
    libxft-dev \
    libxinerama-dev \
    libxcursor-dev \
    libxrender-dev \
    libxfixes-dev \
    libpng-dev \
    jpeg-dev \
    pkgconfig
```

### macOS

#### Using Homebrew (Recommended)
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install fltk
brew install pkg-config

# Install Xcode command line tools
xcode-select --install
```

#### Using MacPorts
```bash
# Install MacPorts dependencies
sudo port install fltk +universal
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
    mingw-w64-x86_64-fltk \
    mingw-w64-x86_64-pkg-config \
    mingw-w64-x86_64-cmake

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
./target/debug/memscan
```

#### Release Build (Optimized)
```bash
cargo build --release
./target/release/memscan
```

#### Minimal Size Build (Ultra-optimized)
```bash
cargo build --profile release-small
./target/release-small/memscan
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
cp target/release/memscan ~/.local/bin/
# Or add target/release to your PATH
```

### Method 2: Cargo Install (Future)
```bash
# Will be available when published to crates.io
cargo install memscan
```

### Method 3: Package Managers (Future)

#### Linux (APT)
```bash
# Future Ubuntu/Debian package
sudo apt install memscan
```

#### macOS (Homebrew)
```bash
# Future Homebrew formula
brew install memscan
```

#### Windows (Scoop)
```bash
# Future Scoop package
scoop install memscan
```

## Post-Installation Setup

### Linux Permissions

#### Option 1: Capabilities (Recommended)
```bash
# Add ptrace capability to binary
sudo setcap cap_sys_ptrace=eip $(which memscan)

# Verify capabilities
getcap $(which memscan)
```

#### Option 2: Sudo Access
```bash
# Run with sudo when needed
sudo memscan
```

#### Option 3: Add to ptrace_scope (Temporary)
```bash
# Allow ptrace for current session
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope

# To make permanent, add to /etc/sysctl.d/10-ptrace.conf:
# kernel.yama.ptrace_scope = 0
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
# Right-click memscan.exe -> "Run as administrator"

# Or add to Windows Defender exclusions if antivirus blocks it
```

## Verification

### Test Installation
```bash
# Check version
memscan --version

# List available options
memscan --help

# Test process enumeration (should show running processes)
memscan
# Click "List Processes" in GUI
```

### Performance Test
```bash
# Run with memory monitoring
RUST_LOG=debug memscan

# Check memory usage stays under 100MB
ps aux | grep memscan
```

## Troubleshooting

### Build Issues

#### FLTK Build Errors
```bash
# Error: "fltk-config not found"
# Solution: Install FLTK development packages

# Linux (Ubuntu/Debian)
sudo apt install libfltk1.3-dev

# macOS
brew install fltk

# Check if fltk-config is in PATH
which fltk-config
```

#### Linker Errors
```bash
# Error: "cannot find -lfltk"
# Solution: Set PKG_CONFIG_PATH

# Linux
export PKG_CONFIG_PATH=/usr/lib/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig
# macOS (Homebrew)
export PKG_CONFIG_PATH=/opt/homebrew/lib/pkgconfig
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
sudo setcap cap_sys_ptrace=eip ./target/release/memscan

# Solution 2: Run as root
sudo ./target/release/memscan

# Solution 3: Adjust ptrace_scope
echo 0 | sudo tee /proc/sys/kernel/yama/ptrace_scope

# macOS: Operation not permitted
# Solution: Add to Developer Tools in Security & Privacy
```

#### GUI Display Issues
```bash
# Error: "cannot connect to X server"
# Linux (WSL/SSH)
export DISPLAY=:0
# Or use X11 forwarding
ssh -X username@hostname

# Error: "Wayland display issues"
# Force X11 mode
export GDK_BACKEND=x11
```

#### Memory Issues
```bash
# Error: Memory limit exceeded
# Check current usage
ps -o pid,ppid,cmd,%mem,%cpu --sort=-%mem

# Verify memory optimization is working
RUST_LOG=trace ./target/release/memscan 2>&1 | grep -i memory
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
ldd target/release/memscan

# Install missing libraries
# Ubuntu/Debian
sudo apt install libx11-6 libxext6 libxft2 libxinerama1

# Arch Linux
sudo pacman -S libx11 libxext libxft libxinerama
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
# AppImage creation (optional)
cargo install cargo-appimage
cargo appimage

# systemd service restrictions
sudo systemctl edit user@.service
# Add: Environment="PATH=/usr/local/bin:/usr/bin:/bin"

# SELinux issues (Fedora/RHEL)
setsebool -P allow_ptrace 1
```

#### macOS-Specific
```bash
# Code signing issues
codesign --force --deep --sign - target/release/memscan

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

[gui]
theme = "dark"
window_size = [900, 650]

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
RUST_BACKTRACE=full RUST_LOG=debug ./target/release/memscan 2>&1 | tee runtime.log
```

---

**Installation complete!** 🎉

For usage instructions, see the main [README.md](README.md).

Built by Danielscos 🐐 - The GOAT of memory scanning