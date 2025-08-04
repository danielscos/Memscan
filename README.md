# Memscan

**High-performance, cross-platform memory scanner built with Rust and FLTK**

Memscan is a lightweight, efficient utility for scanning and analyzing the memory of live processes. Designed for reverse engineering, game modification, and security analysis, it maintains a strict memory footprint under 100MB while providing powerful memory inspection capabilities.

## Features

- **Cross-Platform Support**: Works on Linux, macOS, and Windows
- **Memory-Optimized**: Custom allocator ensures <100MB RAM usage
- **Real-Time Process Scanning**: Live process enumeration and attachment
- **High-Performance Memory Access**: Platform-specific optimized memory APIs
- **Lightweight GUI**: Built with FLTK for minimal resource usage
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
- **Rust** 1.70+ (2024 edition)
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

# Run with automatic privilege handling
./run_memscan.sh
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
# Recommended: Use the helper script (handles privileges automatically)
./run_memscan.sh

# Alternative: Run directly (may need privileges for memory scanning)
cargo run
```

### Memory Scanner Workflow
1. **Start test target**: `./test_target` (in separate terminal)
2. **Launch Memscan**: `./run_memscan.sh` (confirms privilege adjustment)
3. **Click "List Processes"** to enumerate running processes
4. **Select target process** from the list (e.g., "test_target")
5. **Click "Attach"** to connect to the process
6. **Enter value to search** in the input field
7. **Select data type** (i32, i64, f32, f64, String)
8. **Click "Start Scan"** to find memory addresses

### Example: Finding a Value
1. Start test target: `./test_target`
2. Launch memscan: `./run_memscan.sh`
3. Attach to "test_target" process
4. Enter "12345" in the search field (known test value)
5. Select "i32 (4 bytes)" as data type
6. Click "Start Scan" - should find 1-2 matches
7. Try other test values: "testplayer", "sword", "42.5"

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
├── Cargo.toml                  # Dependencies & build config
├── README.md                   # This file
└── INSTALL.md                  # Detailed installation guide
```

## Security & Legal Notice

Memscan should only be used on:
- ✅ Applications you own or develop
- ✅ Systems you have explicit permission to analyze
- ✅ Educational and research purposes
- ✅ Security testing with proper authorization

## Development Status

| Feature | Linux | macOS | Windows | Status |
|---------|-------|-------|---------|---------|
| Process Enumeration | ✅ | ❌️ | ❌️ | Implemented |
| Process Attachment | ✅ | ❌️ | ❌️ | Implemented |
| Memory Reading | ✅ | ❌️ | ❌️ | Implemented |
| Value Scanning | ✅ | ❌️ | ❌️ | Implemented |
| Memory Writing | ❌️ | ❌️ | ❌️ | In Development |

## Troubleshooting

### Common Issues

**"Permission denied" when attaching to process:**
- Use the provided script: `./run_memscan.sh`
- This temporarily adjusts system security settings safely
- Or run with: `sudo -E env DISPLAY=$DISPLAY XAUTHORITY=$XAUTHORITY ./target/release/memscan`

**"No results found" during scanning:**
- Ensure the test target is running: `./test_target`
- Try scanning for known values: 12345, "testplayer", "sword"
- Check that you're attached to the correct process

**Build fails with FLTK errors:**
- See [INSTALL.md](INSTALL.md) for platform-specific dependencies
- Ensure you have a C++ compiler installed

**GUI doesn't open with sudo:**
- Use `./run_memscan.sh` instead (better approach)
- This runs as regular user with temporary security adjustment

For more troubleshooting, see [INSTALL.md](INSTALL.md#troubleshooting).

## Roadmap

- [ ] **v0.2.0**: Complete memory reading implementation
- [ ] **v0.3.0**: Add value filtering and scanning algorithms
- [ ] **v0.4.0**: Memory writing capabilities
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

**Memscan v0.1.0** - Built by Danielscos
*High-performance memory scanning for the modern age*
