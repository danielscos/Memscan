# Memscan Usage Guide

**High-performance, cross-platform memory scanner built with Rust**

## Quick Start

Memscan provides a command-line interface for memory scanning.

### 💻 CLI Mode
```bash
# Interactive mode
./target/release/memscan-cli

# One-shot commands
./target/release/memscan-cli list
./target/release/memscan-cli scan <PID> <VALUE> <TYPE>

# Get help
./target/release/memscan-cli --help
```

## Installation & Building

### Prerequisites
```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config

# Arch Linux  
sudo pacman -S base-devel pkgconf

# Fedora
sudo dnf install pkgconfig
```

### Build
```bash
git clone https://github.com/danielscos/memscan.git
cd memscan
cargo build --release
```

## CLI Usage

### Interactive Mode

Start interactive mode for multiple commands:
```bash
./target/release/memscan-cli

# Shows banner and system checks, then:
memscan> list
memscan> scan 1234 42 i32
memscan> info 1234
memscan> exit
```

### Step-by-Step Walkthrough

1. **Start the test target** (for testing):
   ```bash
   ./test_target
   ```

2. **Launch Memscan CLI**:
   ```bash
   ./target/release/memscan-cli  # Interactive mode
   # Or with privileges: ./run_memscan.sh
   ```

3. **List Processes**:
   ```bash
   memscan> list
   ```

4. **Scan for Values**:
   ```bash
   memscan> scan <PID> 12345 i32
   ```

5. **View Results**:
   - Results show memory addresses containing your value
   - Multiple results are normal (variables are copied in memory)

### CLI Features
- **Interactive shell**: Run multiple commands without restarting
- **One-shot mode**: Single commands for scripting
- **Multiple data types**: i32, i64, f32, f64, strings
- **Real-time memory usage**: Stays under 100MB
- **System checks**: Automatic privilege and security verification

## Command Reference

### Commands Overview

| Command | Description | Example |
|---------|-------------|---------|
| `list` | List running processes | `memscan-cli list` |
| `scan` | Scan process memory | `memscan-cli scan 1234 42 i32` |
| `info` | Show process info | `memscan-cli info 1234` |
| `dump` | Dump raw memory | `memscan-cli dump 1234 0x7fff123 256` |

### Data Types

| Type | Description | Example |
|------|-------------|---------|
| `i32` | 32-bit signed integer | `12345` |
| `i64` | 64-bit signed integer | `9876543210` |
| `f32` | 32-bit float | `42.5` |
| `f64` | 64-bit float | `1337.1337` |
| `string` | ASCII string | `testplayer` |

### CLI Examples

#### Basic Process Scanning
```bash
# List all processes
./target/release/memscan-cli list

# Find test_target process and note its PID
./target/release/memscan-cli list | grep test_target

# Scan for integer value
./target/release/memscan-cli scan 43077 12345 i32

# Scan for string value  
./target/release/memscan-cli scan 43077 testplayer string

# Scan for float value
./target/release/memscan-cli scan 43077 42.5 f32
```

#### Process Information
```bash
# Get detailed process info
./target/release/memscan-cli info 43077

# Example output:
# Process Name: test_target
# Total Memory: 2048 KB
# Memory Regions: 25
#   Readable: 15
#   Writable: 8  
#   Executable: 5
```

#### Memory Dumping
```bash
# Dump 256 bytes from address
./target/release/memscan-cli dump 43077 0x7fff12345678 256

# Dump with decimal address
./target/release/memscan-cli dump 43077 140734799804024 256
```

#### Using with Privileges
```bash
# CLI with automatic privilege handling
./run_memscan.sh --cli list
./run_memscan.sh --cli scan 43077 12345 i32

# Manual privilege escalation
sudo -E ./target/release/memscan-cli scan 43077 12345 i32
```

## Test Target Usage

The included `test_target` program provides known values for testing:

### Start Test Target
```bash
./test_target
```

### Known Test Values
| Value | Type | Description |
|-------|------|-------------|
| `12345` | i32 | secret_number |
| `100` | i32 | health_points |
| `999` | i32 | score |
| `9876543210` | i64 | big_number |
| `1000000` | i64 | coins |
| `42.5` | f32 | player_x |
| `15.75` | f32 | speed |
| `1337.1337` | f64 | balance |
| `9999.9999` | f64 | experience |
| `testplayer` | string | username |
| `sword` | string | weapon |
| `dungeon` | string | location |

### Testing Workflow
```bash
# Terminal 1: Start test target
./test_target

# Terminal 2: Scan for known values
./target/release/memscan-cli scan $(pgrep test_target) 12345 i32
./target/release/memscan-cli scan $(pgrep test_target) testplayer string
./target/release/memscan-cli scan $(pgrep test_target) 42.5 f32

# Test value changes (press Enter in test_target)
./target/release/memscan-cli scan $(pgrep test_target) newplayer string
```

## Privilege Management

### Understanding ptrace_scope

Linux systems use `ptrace_scope` to control memory access:

- **0**: No restrictions (ideal for scanning)
- **1**: Restricted to child processes  
- **2**: Admin-only ptrace
- **3**: No ptrace allowed

### Check Current Setting
```bash
cat /proc/sys/kernel/yama/ptrace_scope
```

### Privilege Solutions

#### Option 1: Helper Script (Recommended)
```bash
./run_memscan.sh        # GUI with temporary privileges
./run_memscan.sh --cli list  # CLI with temporary privileges
```

#### Option 2: Manual sudo
```bash
# CLI with sudo
sudo ./target/release/memscan-cli scan 1234 42 i32

# Interactive mode with sudo
sudo ./target/release/memscan-cli
```

#### Option 3: Temporary ptrace_scope Change
```bash
# Temporarily allow memory scanning
sudo sysctl kernel.yama.ptrace_scope=0

# Run memscan
./target/release/memscan-cli list

# Restore security (important!)
sudo sysctl kernel.yama.ptrace_scope=1
```

## Performance & Limits

### Memory Usage
- **Target**: <100MB RAM usage
- **Actual**: Typically 1-5MB for small scans
- **Monitoring**: Real-time usage displayed in CLI output

### Scan Limits
- **Max results**: 10,000 per scan (prevents memory overflow)
- **Chunk size**: 64KB memory reads for efficiency
- **Region filter**: Only scans readable regions >1KB

### Optimization Tips
```bash
# Use specific values to reduce results
memscan-cli scan 1234 12345 i32      # ✅ Good: specific value
memscan-cli scan 1234 0 i32          # ⚠️  Warning: too common

# Use appropriate data types
memscan-cli scan 1234 42.5 f32       # ✅ Good: correct type
memscan-cli scan 1234 42.5 i32       # ❌ Bad: wrong type

# Filter processes before scanning
memscan-cli list | grep game         # ✅ Good: find target first
```

## Troubleshooting

### Common Issues

#### "Permission denied" when attaching
```bash
# Solution 1: Use helper script
./run_memscan.sh

# Solution 2: Check ptrace_scope
cat /proc/sys/kernel/yama/ptrace_scope

# Solution 3: Run with sudo
sudo ./target/release/memscan-cli scan 1234 42 i32
```

#### "No results found" during scanning
```bash
# Check if target process exists
ps aux | grep target_name

# Verify target process has the value
./test_target  # Known good values for testing

# Try different data types
memscan-cli scan 1234 42 i32    # Try integer
memscan-cli scan 1234 42 f32    # Try float
```

#### CLI permission issues
```bash
# Use the helper script for automatic privileges
./run_memscan.sh

# Or run with sudo directly
sudo ./target/release/memscan-cli
```

#### Build fails with missing dependencies
```bash
# Install required packages
# Ubuntu/Debian:
sudo apt install build-essential pkg-config

# Arch Linux:
sudo pacman -S base-devel pkgconf

# Fedora:
sudo dnf install pkgconfig
```

### Debug Mode
```bash
# Enable debug output
RUST_LOG=debug ./target/release/memscan-cli scan 1234 42 i32

# Verbose build
cargo build --release --verbose
```

## Advanced Usage

### Scripting with CLI
```bash
#!/bin/bash
# scan_script.sh - Automated scanning example

TARGET_PROCESS="test_target"
PID=$(pgrep $TARGET_PROCESS)

if [ -z "$PID" ]; then
    echo "Process $TARGET_PROCESS not found"
    exit 1
fi

echo "Scanning process $TARGET_PROCESS (PID: $PID)"

# Scan for multiple values
VALUES=("12345" "100" "999")
for value in "${VALUES[@]}"; do
    echo "Scanning for: $value"
    ./target/release/memscan-cli scan $PID $value i32
    echo "---"
done
```

### Memory Analysis
```bash
# Get process memory layout
./target/release/memscan-cli info 1234

# Dump specific memory regions
./target/release/memscan-cli dump 1234 0x7fff12345678 1024

# Find executable code sections
./target/release/memscan-cli info 1234 | grep "x.*CODE"
```

### Value Change Detection
```bash
# 1. Scan for initial value
./target/release/memscan-cli scan 1234 100 i32

# 2. Change value in target program
# (modify the variable somehow)

# 3. Scan for new value
./target/release/memscan-cli scan 1234 75 i32

# 4. Compare results to find "real" address
```

## Next Steps

### Learning Path
1. **Start with CLI**: Learn the command-line interface
2. **Try interactive mode**: Use persistent shell for efficiency
3. **Use test_target**: Practice with known values
4. **Real applications**: Scan actual programs
5. **Advanced features**: Memory dumping, analysis

### Planned Features (Roadmap)
- **v0.2.0**: Enhanced memory reading
- **v0.3.0**: Value filtering and advanced scanning
- **v0.4.0**: Memory writing capabilities
- **v0.5.0**: Windows and macOS support
- **v1.0.0**: Full cross-platform release

### Contributing
```bash
# Development setup
git clone https://github.com/danielscos/memscan.git
cd memscan
cargo test
cargo run --bin memscan-cli

# See CONTRIBUTING.md for guidelines
```

## Security & Legal Notice

⚠️ **Important**: Only use Memscan on:
- Applications you own or develop
- Systems you have explicit permission to analyze  
- Educational and research purposes
- Security testing with proper authorization

**Never use on**:
- Other users' applications without permission
- Production systems without authorization
- Anti-cheat protected games
- Systems you don't own

---

**For more information**:
- [Installation Guide](INSTALL.md)
- [GitHub Issues](https://github.com/danielscos/memscan/issues)
- [GitHub Discussions](https://github.com/danielscos/memscan/discussions)

Built with ❤️ by Danielscos 🐐