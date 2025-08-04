// Memscan CLI - Command-line memory scanner
// Usage: memscan-cli <command> [options]
// Built by the goat (Danielscos)

use memscan::{
    memory_optimization::get_allocated_bytes,
    process::{Process, enumerate_processes},
    scanner::{
        scan_process_for_f32, scan_process_for_f64, scan_process_for_i32, scan_process_for_i64,
        scan_process_for_string,
    },
};
use std::env;
use std::io::{self, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "list" => cmd_list_processes(),
        "scan" => cmd_scan(&args[2..]),
        "info" => cmd_process_info(&args[2..]),
        "dump" => cmd_dump_memory(&args[2..]),
        "help" | "--help" | "-h" => print_usage(),
        "version" | "--version" | "-v" => print_version(),
        _ => {
            eprintln!("❌ Unknown command: {}", args[1]);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_version() {
    println!("🔍 Memscan CLI v0.1.0");
    println!("High-performance, cross-platform memory scanner");
    println!("Built by Danielscos 🐐");
    println!("Memory usage: {} bytes", get_allocated_bytes());
}

fn print_usage() {
    println!("🔍 Memscan CLI v0.1.0 - High-performance memory scanner");
    println!();
    println!("USAGE:");
    println!("    memscan-cli <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    list                     List all running processes");
    println!("    scan <PID> <VALUE> <TYPE> Scan process memory for value");
    println!("    info <PID>               Show process memory information");
    println!("    dump <PID> <ADDR> <SIZE> Dump raw memory (hex)");
    println!("    help                     Show this help message");
    println!("    version                  Show version information");
    println!();
    println!("SCAN TYPES:");
    println!("    i32        32-bit signed integer");
    println!("    i64        64-bit signed integer");
    println!("    f32        32-bit floating point");
    println!("    f64        64-bit floating point");
    println!("    string     ASCII string");
    println!();
    println!("EXAMPLES:");
    println!("    memscan-cli list");
    println!("    memscan-cli scan 1234 42 i32");
    println!("    memscan-cli scan 1234 'testplayer' string");
    println!("    memscan-cli info 1234");
    println!("    memscan-cli dump 1234 0x7fff12345678 256");
    println!();
    println!("NOTES:");
    println!("    • Use './run_memscan.sh' wrapper for automatic privilege handling");
    println!("    • String values with spaces should be quoted");
    println!("    • Addresses can be in hex (0x...) or decimal format");
    println!();
    println!("Memory usage: {} bytes", get_allocated_bytes());
}

fn cmd_list_processes() {
    println!("🔍 Enumerating processes...");
    print!("Loading");
    io::stdout().flush().unwrap();

    // Show loading animation
    for _ in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        print!(".");
        io::stdout().flush().unwrap();
    }
    println!();

    match enumerate_processes() {
        Ok(processes) => {
            println!("📋 Found {} processes:\n", processes.len());
            println!("{:<8} {}", "PID", "NAME");
            println!("{}", "-".repeat(60));

            for (i, process) in processes.iter().enumerate() {
                if i >= 50 {
                    break;
                }
                println!("{:<8} {}", process.pid, process.name);
            }

            if processes.len() > 50 {
                println!("... and {} more processes", processes.len() - 50);
                println!("💡 Tip: Use GUI version for full interactive list");
            }

            println!("\n🎯 To scan a process: memscan-cli scan <PID> <value> <type>");

            // Suggest test_target if found
            if let Some(test_proc) = processes.iter().find(|p| p.name.contains("test_target")) {
                println!("🧪 Test target found: PID {}", test_proc.pid);
                println!("   Try: memscan-cli scan {} 12345 i32", test_proc.pid);
                println!(
                    "   Try: memscan-cli scan {} testplayer string",
                    test_proc.pid
                );
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to enumerate processes: {}", e);
            eprintln!("💡 Try running with: ./run_memscan.sh");
            process::exit(1);
        }
    }
}

fn cmd_scan(args: &[String]) {
    if args.len() < 3 {
        eprintln!("❌ Usage: memscan-cli scan <PID> <VALUE> <TYPE>");
        eprintln!("   Types: i32, i64, f32, f64, string");
        eprintln!("   Example: memscan-cli scan 1234 42 i32");
        process::exit(1);
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("❌ Invalid PID: {}", args[0]);
            process::exit(1);
        }
    };

    let value = &args[1];
    let scan_type = &args[2];

    println!(
        "🔍 Scanning process {} for value '{}' (type: {})",
        pid, value, scan_type
    );

    // Create and open process
    let mut process = Process::new(pid, format!("PID-{}", pid));
    match process.open() {
        Ok(()) => println!("✅ Successfully attached to process {}", pid),
        Err(e) => {
            eprintln!("❌ Failed to attach to process {}: {}", pid, e);
            eprintln!("💡 Try running with: ./run_memscan.sh");
            process::exit(1);
        }
    }

    let handle = process.handle.as_ref().unwrap();

    // Show scanning progress
    print!("🔄 Scanning memory");
    io::stdout().flush().unwrap();

    let scan_result = match scan_type.as_str() {
        "i32" => scan_process_for_i32(handle, value),
        "i64" => scan_process_for_i64(handle, value),
        "f32" => scan_process_for_f32(handle, value),
        "f64" => scan_process_for_f64(handle, value),
        "string" => scan_process_for_string(handle, value),
        _ => {
            eprintln!("\n❌ Unknown scan type: {}", scan_type);
            eprintln!("   Valid types: i32, i64, f32, f64, string");
            process::exit(1);
        }
    };

    println!(); // New line after progress

    match scan_result {
        Ok(results) => {
            println!("🎯 Scan complete! Found {} matches", results.len());

            if results.is_empty() {
                println!("❌ No matches found for value '{}'", value);
                println!("💡 Tips:");
                println!("   • Make sure the target process contains this value");
                println!("   • Try different data types (i32, i64, f32, f64, string)");
                println!("   • For test_target, try: 12345, testplayer, sword, 42.5");
            } else {
                println!("\n📍 SCAN RESULTS:");
                println!("{:<18} {}", "ADDRESS", "DESCRIPTION");
                println!("{}", "-".repeat(50));

                for (i, result) in results.iter().enumerate() {
                    if i >= 20 {
                        // Limit output to first 20 results
                        println!("... and {} more matches", results.len() - 20);
                        break;
                    }
                    println!("0x{:016x} {} ({})", result.address, value, scan_type);
                }

                println!("\n✅ Success! Found value in process memory");

                if results.len() > 1 {
                    println!("💡 Multiple matches found. In a real scenario, you would:");
                    println!("   1. Change the value in the target program");
                    println!("   2. Scan again for the new value");
                    println!("   3. Find which address(es) updated");
                }

                if results.len() <= 5 {
                    println!(
                        "🎉 Excellent! Low number of matches - easy to identify the real variable"
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Scan failed: {}", e);
            eprintln!("💡 Common issues:");
            eprintln!("   • Process may have exited");
            eprintln!("   • Insufficient permissions (try ./run_memscan.sh)");
            eprintln!("   • Invalid value format for the specified type");
            process::exit(1);
        }
    }

    println!("\nMemory usage: {} bytes", get_allocated_bytes());
}

fn cmd_process_info(args: &[String]) {
    if args.is_empty() {
        eprintln!("❌ Usage: memscan-cli info <PID>");
        process::exit(1);
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("❌ Invalid PID: {}", args[0]);
            process::exit(1);
        }
    };

    println!("📊 Process Information for PID {}", pid);
    println!("{}", "=".repeat(50));

    // Try to get process name
    let proc_comm_path = format!("/proc/{}/comm", pid);
    let process_name = std::fs::read_to_string(&proc_comm_path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    println!("Process Name: {}", process_name);
    println!("Process ID:   {}", pid);

    // Try to get memory regions
    let mut process = Process::new(pid, process_name.clone());
    match process.open() {
        Ok(()) => {
            if let Some(handle) = &process.handle {
                match handle.get_memory_regions() {
                    Ok(regions) => {
                        let total_memory: usize = regions.iter().map(|r| r.size).sum();
                        let readable_regions: Vec<_> =
                            regions.iter().filter(|r| r.readable).collect();
                        let writable_regions: Vec<_> =
                            regions.iter().filter(|r| r.writable).collect();
                        let executable_regions: Vec<_> =
                            regions.iter().filter(|r| r.executable).collect();

                        println!("Total Memory:     {} KB", total_memory / 1024);
                        println!("Memory Regions:   {}", regions.len());
                        println!("  Readable:       {}", readable_regions.len());
                        println!("  Writable:       {}", writable_regions.len());
                        println!("  Executable:     {}", executable_regions.len());

                        println!("\n📍 MEMORY LAYOUT (first 10 regions):");
                        println!(
                            "{:<18} {:<10} {:<8} {}",
                            "START", "SIZE (KB)", "PERMS", "TYPE"
                        );
                        println!("{}", "-".repeat(60));

                        for (i, region) in regions.iter().enumerate() {
                            if i >= 10 {
                                println!("... and {} more regions", regions.len() - 10);
                                break;
                            }

                            let perms = format!(
                                "{}{}{}",
                                if region.readable { "r" } else { "-" },
                                if region.writable { "w" } else { "-" },
                                if region.executable { "x" } else { "-" }
                            );

                            let region_type = if region.executable {
                                "CODE"
                            } else if region.writable {
                                "DATA"
                            } else if region.readable {
                                "READ"
                            } else {
                                "----"
                            };

                            println!(
                                "0x{:016x} {:<10} {:<8} {}",
                                region.start_address,
                                region.size / 1024,
                                perms,
                                region_type
                            );
                        }

                        match handle.get_scannable_regions() {
                            Ok(scannable) => {
                                let scannable_size: usize = scannable.iter().map(|r| r.size).sum();
                                println!("\n🎯 SCANNING INFO:");
                                println!("Scannable regions: {}", scannable.len());
                                println!("Scannable memory:  {} KB", scannable_size / 1024);
                            }
                            Err(e) => println!("⚠️  Could not get scannable regions: {}", e),
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to get memory regions: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to attach to process: {}", e);
            eprintln!("💡 Try running with: ./run_memscan.sh");
        }
    }
}

fn cmd_dump_memory(args: &[String]) {
    if args.len() < 3 {
        eprintln!("❌ Usage: memscan-cli dump <PID> <ADDRESS> <SIZE>");
        eprintln!("   ADDRESS can be hex (0x...) or decimal");
        eprintln!("   SIZE is in bytes");
        eprintln!("   Example: memscan-cli dump 1234 0x7fff12345678 256");
        process::exit(1);
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("❌ Invalid PID: {}", args[0]);
            process::exit(1);
        }
    };

    let address: usize = if args[1].starts_with("0x") || args[1].starts_with("0X") {
        usize::from_str_radix(&args[1][2..], 16)
    } else {
        args[1].parse()
    }
    .unwrap_or_else(|_| {
        eprintln!("❌ Invalid address: {}", args[1]);
        process::exit(1);
    });

    let size: usize = match args[2].parse() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("❌ Invalid size: {}", args[2]);
            process::exit(1);
        }
    };

    if size > 4096 {
        eprintln!("❌ Size too large (max 4096 bytes)");
        process::exit(1);
    }

    println!(
        "🔍 Memory dump for PID {} at 0x{:x} ({} bytes)",
        pid, address, size
    );

    let mut process = Process::new(pid, format!("PID-{}", pid));
    match process.open() {
        Ok(()) => {
            if let Some(handle) = &process.handle {
                match handle.read_memory(address, size) {
                    Ok(data) => {
                        println!("\n📄 HEX DUMP:");
                        print_hex_dump(&data, address);

                        println!("\n📝 ASCII VIEW:");
                        print_ascii_view(&data);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to read memory: {}", e);
                        eprintln!("💡 Address may be invalid or inaccessible");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to attach to process: {}", e);
            eprintln!("💡 Try running with: ./run_memscan.sh");
        }
    }
}

fn print_hex_dump(data: &[u8], base_address: usize) {
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("0x{:016x}: ", base_address + i * 16);

        // Print hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                print!(" ");
            }
            print!("{:02x} ", byte);
        }

        // Pad if necessary
        for j in chunk.len()..16 {
            if j == 8 {
                print!(" ");
            }
            print!("   ");
        }

        // Print ASCII
        print!(" |");
        for byte in chunk {
            let c = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            print!("{}", c);
        }
        println!("|");
    }
}

fn print_ascii_view(data: &[u8]) {
    let ascii_string: String = data
        .iter()
        .map(|&b| {
            if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            }
        })
        .collect();

    for (i, chunk) in ascii_string
        .chars()
        .collect::<Vec<_>>()
        .chunks(64)
        .enumerate()
    {
        println!("{:3}: {}", i, chunk.iter().collect::<String>());
    }
}
