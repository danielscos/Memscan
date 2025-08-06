// Memscan v0.1.0 - Entry point with usage information
// Built by the goat (Danielscos)

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Print header
    println!("🔍 Memscan v0.1.0 - High-performance memory scanner");
    println!("   Built by Danielscos 🐐");
    println!();

    // Check for help flags
    if args.len() > 1 {
        match args[1].as_str() {
            "help" | "--help" | "-h" => {
                print_usage();
                return;
            }
            "version" | "--version" | "-v" => {
                println!("Memscan v0.1.0");
                println!("High-performance, cross-platform memory scanner");
                return;
            }
            _ => {}
        }
    }

    // Show usage and available binaries
    println!("📋 AVAILABLE INTERFACES:");
    println!();
    println!("🖥️  GUI Version (Recommended):");
    println!("   cargo run --bin memscan-gui");
    println!("   ./target/release/memscan-gui");
    println!("   ./run_memscan.sh  (with privilege handling)");
    println!();
    println!("    CLI Version (Terminal):");
    println!("   cargo run --bin memscan-cli --help");
    println!("   ./target/release/memscan-cli list");
    println!("   ./target/release/memscan-cli scan <PID> <VALUE> <TYPE>");
    println!();
    println!("🎯 Quick Examples:");
    println!("   memscan-cli list                    # List processes");
    println!("   memscan-cli scan 1234 42 i32       # Scan for integer");
    println!("   memscan-cli scan 1234 'hello' string # Scan for string");
    println!("   memscan-cli info 1234               # Process info");
    println!();
    println!("🚀 Starting GUI by default...");
    println!("   (Use --help for more options)");
    println!();

    // Check if GUI is available and start it
    match start_gui() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("❌ Failed to start GUI: {}", e);
            eprintln!("💡 Try CLI version instead:");
            eprintln!("   cargo run --bin memscan-cli --help");
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("USAGE:");
    println!("    memscan [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --help, -h       Show this help message");
    println!("    --version, -v    Show version information");
    println!();
    println!("BINARIES:");
    println!("    memscan-gui      Launch graphical interface (default)");
    println!("    memscan-cli      Command-line interface");
    println!();
    println!("EXAMPLES:");
    println!("    cargo run                           # Start GUI");
    println!("    cargo run --bin memscan-gui         # Start GUI explicitly");
    println!("    cargo run --bin memscan-cli list    # List processes in CLI");
    println!("    ./run_memscan.sh                    # GUI with privilege handling");
    println!();
    println!("For CLI usage: cargo run --bin memscan-cli --help");
}

fn start_gui() -> Result<(), Box<dyn std::error::Error>> {
    // Import and run the GUI main function
    // This is a bit of a hack since we can't directly call the other binary's main
    // In practice, users should use the specific binaries directly

    println!("💡 For best experience, use the specific binaries:");
    println!("   ./target/release/memscan-gui");
    println!("   ./target/release/memscan-cli");
    println!();
    println!("🔧 Building and starting GUI...");

    // Use std::process to run the GUI binary
    let status = process::Command::new("cargo")
        .args(&["run", "--bin", "memscan-gui"])
        .status()?;

    if !status.success() {
        return Err("GUI failed to start".into());
    }

    Ok(())
}
