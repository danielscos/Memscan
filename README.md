# Memscan

**High performance, cross-platform memory scanner built with Rust.**

Memscan is a lightweight, efficient command-line utility for scanning and analyzing the memory of live processes. Designed for reverse engineering, game modification, and security analysis, it maintains a strict memory footprint under 100MB while providing powerful memory inspection capabilities.

## Features

- **Cross-Platform Support**: Works on Linux, macOS, and Windows
- **Memory-Optimized**: Custom allocator ensures <100MB RAM usage
- **Real-Time Process Scanning**: Live process enumeration and attachment
- **High-Performance Memory Access**: Platform-specific optimized memory APIs
- **Command-Line Interface**: Efficient CLI for memory scanning and analysis
- **Value Search**: Find specific integers, floats, and strings in process memory
- **Memory Region Analysis**: Inspect memory layouts and permissions

## Memory Access Implementation

Memscan uses platform-specific APIs for optimal performance:

| Platform | Memory Read/Write APIs |
|----------|------------------------|
| **Linux** | `process_vm_readv` / `process_vm_writev` (via nix crate) |
| **macOS** | Mach Virtual Memory APIs |
| **Windows** | `ReadProcessMemory` / `WriteProcessMemory` (via windows-sys) |

## Requirements

### Build Dependencies
- **Rust** 1.8+ (2024 edition)
- **Cargo** package manager

### Platform-Specific Requirements

#### Linux
```bash
# Ubuntu/Debian
sudo apt install libfltk1.3-dev

# Arch Linux
sudo pacman -S fltk

# Fedora
sudo dnf install fltk-devel
```

#### macOS
```bash
# Using Homebrew
brew install fltk

# Using MacPorts
sudo port install fltk
```

#### Windows
- FLTK will be built automatically via vcpkg integration
- Requires Visual Studio Build Tools or MSVC

## 📦 Installation

For detailed installation instructions, see [INSTALL.md](INSTALL.md).

### Quick Start
```bash
# Clone and build
git clone https://github.com/yourusername/memscan.git
cd memscan
cargo build --release

```

## Build Profiles

Memscan includes optimized build profiles:

```bash
# Standard release (balanced performance/size)
cargo build --release

# Maximum optimization (smallest binary)
cargo build --profile release-small

# Development (faster compilation)
cargo build
```

## Usage

### Basic Usage
```bash
# Run the CLI tool
./target/release/memscan-cli

# Or run with automatic privilege handling
./run_memscan.sh
```

### Memory Scanner Workflow
1. **Start test target**: `./test_target` (in separate terminal)
2. **Launch Memscan**: `./target/release/memscan-cli` (or use interactive mode)
3. **List processes**: Use `list` command to enumerate running processes
4. **Scan for values**: Use `scan <PID> <VALUE> <TYPE>` command
5. **View results**: Memory addresses containing your value will be displayed

### Example: Finding a Value
1. Start test target: `./test_target`
2. Launch memscan: `./target/release/memscan-cli`
3. List processes: `list`
4. Scan for value: `scan <PID> 12345 i32` (known test value)
5. Should find 1-2 matches
6. Try other test values: `scan <PID> testplayer string`

## Architecture

```
memscan/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── memory_optimization.rs  # Custom allocator & memory tracking
│   ├── process.rs              # Cross-platform process management
│   ├── memory.rs               # Memory region handling
│   ├── scanner.rs              # Value scanning algorithms
│   └── utils.rs                # Helper functions
│   └── bin/
│       └── memscan-cli.rs      # command-line interface
├── Cargo.toml                  # Dependencies & build config
├── README.md                   # This file
└── INSTALL.md                  # Detailed installation guide
```

## Development Status

| Feature | Linux | macOS | Windows | Status |
|---------|-------|-------|---------|---------|
| Process Enumeration | ✅ | ❌️ | ❌️ | Implemented |
| Process Attachment | ✅ | ❌️ | ❌️ | Implemented |
| Memory Reading | ✅ | ❌️ | ❌️ | Implemented |
| Value Scanning | ✅ | ❌️ | ❌️ | Implemented |
| Memory Writing | ✅ | ❌️ | ❌️ | Implemented |

## Troubleshooting

### Common Issues

**"Permission denied" when attaching to process:**
- Use the provided script: `./run_memscan.sh`
- This temporarily adjusts system security settings safely
- Or run with: `sudo ./target/release/memscan-cli`

**"No results found" during scanning:**
- Ensure the test target is running: `./test_target`
- Try scanning for known values: 12345, "testplayer", "sword"
- Check that you're attached to the correct process

**CLI permission issues:**
- Use `./run_memscan.sh` for automatic privilege handling
- Or run with: `sudo ./target/release/memscan-cli`

For more troubleshooting, see [INSTALL.md](INSTALL.md#troubleshooting).

## Roadmap

- [x] **v0.2.0**: Complete memory reading implementation
- [x] **v0.3.0**: Add value filtering and scanning algorithms
- [x] **v0.4.0**: Memory writing capabilities
- [ ] **v0.5.0**: Windows and macOS platform support
- [ ] **v1.0.0**: Full cross-platform release

## Memory Optimization Details

Memscan implements several memory optimization techniques:

- **Custom Global Allocator**: Tracks all allocations in real-time
- **Object Pools**: Reuse frequently allocated objects
- **String Interning**: Eliminate duplicate string allocations
- **Lazy Loading**: Load features only when needed
- **Minimal Dependencies**: Carefully selected lightweight crates

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Commit changes: `git commit -m 'Add amazing feature'`
6. Push to branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

### Development Guidelines
- Follow Rust naming conventions
- Maintain memory efficiency (<100MB limit)
- Add platform-specific implementations in feature gates
- Include tests for critical functionality
- Document public APIs

## Support

- **Issues**: [GitHub Issues](https://github.com/danielscos/memscan/issues)
- **Discussions**: [GitHub Discussions](https://github.com/danielscos/memscan/discussions)
- **Documentation**: [INSTALL.md](INSTALL.md)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author
```bash
▓█████▄  ▄▄▄       ███▄    █  ██▓▓█████  ██▓      ██████  ▒█████   ▄████▄    ██████
▒██▀ ██▌▒████▄     ██ ▀█   █ ▓██▒▓█   ▀ ▓██▒    ▒██    ▒ ▒██▒  ██▒▒██▀ ▀█  ▒██    ▒
░██   █▌▒██  ▀█▄  ▓██  ▀█ ██▒▒██▒▒███   ▒██░    ░ ▓██▄   ▒██░  ██▒▒▓█    ▄ ░ ▓██▄
░▓█▄   ▌░██▄▄▄▄██ ▓██▒  ▐▌██▒░██░▒▓█  ▄ ▒██░      ▒   ██▒▒██   ██░▒▓▓▄ ▄██▒  ▒   ██▒
░▒████▓  ▓█   ▓██▒▒██░   ▓██░░██░░▒████▒░██████▒▒██████▒▒░ ████▓▒░▒ ▓███▀ ░▒██████▒▒
 ▒▒▓  ▒  ▒▒   ▓▒█░░ ▒░   ▒ ▒ ░▓  ░░ ▒░ ░░ ▒░▓  ░▒ ▒▓▒ ▒ ░░ ▒░▒░▒░ ░ ░▒ ▒  ░▒ ▒▓▒ ▒ ░
 ░ ▒  ▒   ▒   ▒▒ ░░ ░░   ░ ▒░ ▒ ░ ░ ░  ░░ ░ ▒  ░░ ░▒  ░ ░  ░ ▒ ▒░   ░  ▒   ░ ░▒  ░ ░
 ░ ░  ░   ░   ▒      ░   ░ ░  ▒ ░   ░     ░ ░   ░  ░  ░  ░ ░ ░ ▒  ░        ░  ░  ░
   ░          ░  ░         ░  ░     ░  ░    ░  ░      ░      ░ ░  ░ ░            ░
 ░                                                                ░
```

**Memscan v0.4.0** - Built by Danielscos
