// Memscan CLI - Command-line memory scanner
// Usage: memscan-cli <command> [options]
// Built by the goat (danielscos)
//
//=======================================================
//=======================================================

//         ▓█████▄  ▄▄▄       ███▄    █  ██▓▓█████  ██▓      ██████  ▒█████   ▄████▄    ██████
//         ▒██▀ ██▌▒████▄     ██ ▀█   █ ▓██▒▓█   ▀ ▓██▒    ▒██    ▒ ▒██▒  ██▒▒██▀ ▀█  ▒██    ▒
//         ░██   █▌▒██  ▀█▄  ▓██  ▀█ ██▒▒██▒▒███   ▒██░    ░ ▓██▄   ▒██░  ██▒▒▓█    ▄ ░ ▓██▄
//         ░▓█▄   ▌░██▄▄▄▄██ ▓██▒  ▐▌██▒░██░▒▓█  ▄ ▒██░      ▒   ██▒▒██   ██░▒▓▓▄ ▄██▒  ▒   ██▒
//         ░▒████▓  ▓█   ▓██▒▒██░   ▓██░░██░░▒████▒░██████▒▒██████▒▒░ ████▓▒░▒ ▓███▀ ░▒██████▒▒
//         ▒▒▓  ▒  ▒▒   ▓▒█░░ ▒░   ▒ ▒ ░▓  ░░ ▒░ ░░ ▒░▓  ░▒ ▒▓▒ ▒ ░░ ▒░▒░▒░ ░ ░▒ ▒  ░▒ ▒▓▒ ▒ ░
//         ░ ▒  ▒   ▒   ▒▒ ░░ ░░   ░ ▒░ ▒ ░ ░ ░  ░░ ░ ▒  ░░ ░▒  ░ ░  ░ ▒ ▒░   ░  ▒   ░ ░▒  ░ ░
//         ░ ░  ░   ░   ▒      ░   ░ ░  ▒ ░   ░     ░ ░   ░  ░  ░  ░ ░ ░ ▒  ░        ░  ░  ░
//           ░          ░  ░         ░  ░     ░  ░    ░  ░      ░      ░ ░  ░ ░            ░
//         ░                                                                ░

//=======================================================
//=======================================================

use memscan::{
    memory_optimization::get_allocated_bytes,
    process::{Process, enumerate_processes},
    scanner::{
        scan_process_for_f32, scan_process_for_f64, scan_process_for_i32, scan_process_for_i64,
        scan_process_for_string,
    },
    utils::{display_system_info, loading_with_checks, suggest_fixes},
};
use std::env;
use std::io::{self, Write};

const DANIELSCOS_BANNER: &str = r#"
`7MMM.     ,MMF'
  MMMb    dPMM
  M YM   ,M MM  .gP"Ya `7MMpMMMb.pMMMb.  ,pP"Ybd  ,p6"bo   ,6"Yb.  `7MMpMMMb.
  M  Mb  M' MM ,M'   Yb  MM    MM    MM  8I   `" 6M'  OO  8)   MM    MM    MM
  M  YM.P'  MM 8M""""""  MM    MM    MM  `YMMMa. 8M        ,pm9MM    MM    MM
  M  `YM'   MM YM.    ,  MM    MM    MM  L.   I8 YM.    , 8M   MM    MM    MM
.JML. `'  .JMML.`Mbmmd'.JMML  JMML  JMML.M9mmmP'  YMbmd'  `Moo9^Yo..JMML  JMML.


                    High Performance Memory Scanner v0.4.0
                             Built by danielscos
                    "#;

fn print_banner_and_initialize() -> Result<(), String> {
    print!("{}", DANIELSCOS_BANNER);

    match loading_with_checks() {
        Ok(system_info) => {
            display_system_info(&system_info);

            if !system_info.has_sudo || system_info.ptrace_scope.unwrap_or(0) > 1 {
                suggest_fixes(&system_info);
            }

            println!("\n    Memscan ready for operation!\n");
            Ok(())
        }
        Err(error) => {
            eprintln!("\n   Initialization failed: {}", error);
            eprintln!(" Try running with sudo");
            Err(error)
        }
    }
}

fn main() {
    if let Err(_) = print_banner_and_initialize() {
        std::process::exit(1);
    }

    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        execute_command(&args[1..]);
    } else {
        run_interactive_mode();
    }
}

fn run_interactive_mode() {
    println!("   CLI started. Type 'help' for commands or 'exit' to quit.");

    let stdin = io::stdin();

    loop {
        print!("memscan> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(_) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                let args: Vec<&str> = input.split_whitespace().collect();
                if args.is_empty() {
                    continue;
                }

                if matches!(args[0], "exit" | "quit" | "q") {
                    println!("Au Revoir!");
                    break;
                }

                let string_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

                execute_command(&string_args);

                println!();
            }
            Err(e) => {
                eprintln!("     Error reading input: {}", e);
                break;
            }
        }
    }
}

fn execute_command(args: &[String]) {
    if args.is_empty() {
        print_usage();
        return;
    }

    match args[0].as_str() {
        "list" => cmd_list_processes(),
        "scan" => {
            if args.len() >= 4 {
                cmd_scan(&args[1..]);
            } else {
                eprintln!("   Usage: scan <PID> <VALUE> <TYPE>");
                eprintln!("   Types: i32, i64, f32, f64, string");
                eprintln!("   Example: scan 1234 42 i32");
            }
        }
        "info" => {
            if args.len() >= 2 {
                cmd_process_info(&args[1..]);
            } else {
                eprintln!("   Usage: info <PID>");
                eprintln!("   Example: info 1234");
            }
        }
        "dump" => {
            if args.len() >= 4 {
                cmd_dump_memory(&args[1..]);
            } else {
                eprintln!("   Usage: dump <PID> <ADDRESS> <SIZE>");
                eprintln!("   ADDRESS can be hex (0x...) or decimal");
                eprintln!("   SIZE is in bytes");
                eprintln!("   Example: dump 1234 0x7fff12345678 256");
            }
        }
        "write" => {
            if args.len() >= 4 {
                cmd_write_memory(&args[1..]);
            } else {
                eprintln!("   Usage: write <PID> <ADDRESS> <HEX_DATA>");
                eprintln!("   OR:     write <PID> <ADDRESS> --string <TEXT>");
                eprintln!();
                eprintln!("   HEX DATA Examples:");
                eprintln!(
                    "     write 1234 0x7fff12345678 \"48656c6c6f\"        # Write 'Hello' as hex"
                );
                eprintln!(
                    "     write 1234 0x7fff12345678 \"64000000\"          # Write i32 value 100"
                );
                eprintln!(
                    "     write 1234 0x7fff12345678 \"0000c842\"          # Write f32 value 100.0"
                );
                eprintln!();
                eprintln!("   STRING Examples:");
                eprintln!(
                    "     write 1234 0x7fff12345678 --string \"Hello\"    # Write string directly"
                );
                eprintln!(
                    "     write 1234 0x7fff12345678 --string \"newname\"  # Write new string"
                );
                eprintln!();
                eprintln!("       TIP: Use 'modify' command instead for typed values:");
                eprintln!(
                    "     modify 1234 0x7fff12345678 100 i32              # Easier for numbers"
                );
                eprintln!(
                    "     modify 1234 0x7fff12345678 \"newname\" string    # Easier for strings"
                );
            }
        }
        "modify" => {
            if args.len() >= 5 {
                cmd_modify_value(&args[1..]);
            } else {
                eprintln!("   Usage: modify <PID> <ADDRESS> <VALUE> <TYPE>");
                eprintln!("   Types: i32, i64, f32, f64, string");
                eprintln!("   Example: modify 1234 0x7fff123456 999 i32");
                eprintln!("   Example: modify 1234 0x7fff123456 \"newname\" string");
            }
        }
        "help" | "--help" | "-h" => print_usage(),
        "version" | "--version" | "-v" => print_version(),
        "clear" | "cls" => {
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush().unwrap();
        }
        "banner" => {
            print!("{}", DANIELSCOS_BANNER);
        }
        _ => {
            eprintln!("   Unknown command: '{}'", args[0]);
            eprintln!("   Type 'help' for available commands");
        }
    }
}

fn print_version() {
    println!("Memory usage: {} bytes", get_allocated_bytes());
}

fn print_usage() {
    println!();
    println!("USAGE:");
    println!("    Interactive: memscan-cli");
    println!("    One-shot:    memscan-cli <COMMAND> [OPTIONS]");
    println!();
    println!("READ COMMANDS:");
    println!("    list                        List all running processes");
    println!("    scan <PID> <VALUE> <TYPE>   Scan process memory for value");
    println!("    info <PID>                  Show process memory information");
    println!("    dump <PID> <ADDR> <SIZE>    Dump raw memory (hex)");
    println!();
    println!("WRITE COMMANDS:");
    println!("    write <PID> <ADDR> <HEX_DATA>       Write raw hex data to memory");
    println!("    write <PID> <ADDR> --string <TEXT>  Write string to memory");
    println!("    modify <PID> <ADDR> <VAL> <TYPE>    Modify typed value (recommended)");
    println!();
    println!("INTERACTIVE COMMANDS:");
    println!("    help                        Show this help message");
    println!("    version                     Show version information");
    println!("    clear                       Clear screen");
    println!("    banner                      Show banner again");
    println!("    exit                        Quit interactive mode");
    println!();
    println!("DATA TYPES:");
    println!("    i32        32-bit signed integer");
    println!("    i64        64-bit signed integer");
    println!("    f32        32-bit floating point");
    println!("    f64        64-bit floating point");
    println!("    string     ASCII string");
    println!();
    println!("EXAMPLES:");
    println!("    # Interactive mode");
    println!("    $ ./memscan-cli");
    println!("    memscan> list");
    println!("    memscan> scan 1234 42 i32");
    println!("    memscan> info 1234");
    println!("    memscan> exit");
    println!();
    println!("    # One-shot mode (for scripting)");
    println!("    $ ./memscan-cli list");
    println!("    $ ./memscan-cli scan 1234 42 i32");
    println!();
    println!("NOTES:");
    println!("    • For GUI, run './run_memscan.sh' wrapper for automatic privilege handling");
    println!("    • Always run this tool with sudo");
    println!("    • Addresses can be in hex (0x...) or decimal format");
    println!("    • Write operations require writable memory regions");
    println!("    • Use 'info' command to find writable memory regions");
    println!();
    println!("Memory usage: {} bytes", get_allocated_bytes());
}

fn cmd_list_processes() {
    println!("    Enumerating processes...");
    print!("Loading");
    io::stdout().flush().unwrap();

    // loading anims
    for _ in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        print!(".");
        io::stdout().flush().unwrap();
    }
    println!();

    match enumerate_processes() {
        Ok(processes) => {
            println!("Found {} processes:\n", processes.len());
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
                println!("Tip: Use GUI version for full interactive list");
            }

            println!("\nTo scan a process: scan <PID> <value> <type>");

            // Suggest test_target if found
            if let Some(test_proc) = processes.iter().find(|p| p.name.contains("test_target")) {
                println!("   Test target found: PID {}", test_proc.pid);
                println!("   Try: scan {} 12345 i32", test_proc.pid);
                println!("   Try: scan {} testplayer string", test_proc.pid);
            }
        }
        Err(e) => {
            eprintln!("   Failed to enumerate processes: {}", e);
            eprintln!("   Try running with sudo");
        }
    }
}

fn cmd_scan(args: &[String]) {
    if args.len() < 3 {
        eprintln!("   Usage: scan <PID> <VALUE> <TYPE>");
        eprintln!("   Types: i32, i64, f32, f64, string");
        eprintln!("   Example: scan 1234 42 i32");
        return;
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("   Invalid PID: {}", args[0]);
            return;
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
        Ok(()) => println!("     Successfully attached to process {}", pid),
        Err(e) => {
            eprintln!("   Failed to attach to process {}: {}", pid, e);
            eprintln!("   Try running with sudo");
            return;
        }
    }

    let handle = process.handle.as_ref().unwrap();

    // Show scanning progress
    print!("     Scanning memory");
    io::stdout().flush().unwrap();

    let scan_result = match scan_type.as_str() {
        "i32" => scan_process_for_i32(handle, value),
        "i64" => scan_process_for_i64(handle, value),
        "f32" => scan_process_for_f32(handle, value),
        "f64" => scan_process_for_f64(handle, value),
        "string" => scan_process_for_string(handle, value),
        _ => {
            eprintln!("\n    Unknown scan type: {}", scan_type);
            eprintln!("   Valid types: i32, i64, f32, f64, string");
            return;
        }
    };

    println!(); // New line after progress

    match scan_result {
        Ok(results) => {
            println!("    Scan complete! Found {} matches", results.len());

            if results.is_empty() {
                println!("   No matches found for value '{}'", value);
                println!("   Tips:");
                println!("   • Make sure the target process contains this value");
                println!("   • Try different data types (i32, i64, f32, f64, string)");
                println!("   • For test_target, try: 12345, testplayer, sword, 42.5");
            } else {
                println!("\n SCAN RESULTS:");
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

                println!("\n     Success! Found value in process memory");

                if results.len() > 1 {
                    println!("   Multiple matches found. In a real scenario, you would:");
                    println!("   1. Change the value in the target program");
                    println!("   2. Scan again for the new value");
                    println!("   3. Find which address(es) updated");
                }

                if results.len() <= 5 {
                    println!(
                        "Excellent! Low number of matches - easy to identify the real variable"
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("   Scan failed: {}", e);
            eprintln!("   Common issues:");
            eprintln!("   • Process may have exited");
            eprintln!("   • Insufficient permissions (try running with sudo)");
            eprintln!("   • Invalid value format for the specified type");
        }
    }

    println!("\nMemory usage: {} bytes", get_allocated_bytes());
}

fn cmd_process_info(args: &[String]) {
    if args.is_empty() {
        eprintln!("  Usage: info <PID>");
        return;
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("  Invalid PID: {}", args[0]);
            return;
        }
    };

    println!("   Process Information for PID {}", pid);
    println!("{}", "=".repeat(50));

    let proc_comm_path = format!("/proc/{}/comm", pid);
    let process_name = std::fs::read_to_string(&proc_comm_path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    println!("Process Name: {}", process_name);
    println!("Process ID:   {}", pid);

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

                        println!("\n MEMORY LAYOUT (first 10 regions):");
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
                                println!("\n SCANNING INFO:");
                                println!("Scannable regions: {}", scannable.len());
                                println!("Scannable memory:  {} KB", scannable_size / 1024);
                            }
                            Err(e) => println!("     Could not get scannable regions: {}", e),
                        }
                    }
                    Err(e) => {
                        eprintln!("  Failed to get memory regions: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("   Failed to attach to process: {}", e);
            eprintln!("   Try running with sudo");
        }
    }
}

fn cmd_dump_memory(args: &[String]) {
    if args.len() < 3 {
        eprintln!("   Usage: dump <PID> <ADDRESS> <SIZE>");
        eprintln!("   ADDRESS can be hex (0x...) or decimal");
        eprintln!("   SIZE is in bytes");
        eprintln!("   Example: dump 1234 0x7fff12345678 256");
        return;
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("  Invalid PID: {}", args[0]);
            return;
        }
    };

    let address: usize = if args[1].starts_with("0x") || args[1].starts_with("0X") {
        usize::from_str_radix(&args[1][2..], 16)
    } else {
        args[1].parse()
    }
    .unwrap_or_else(|_| {
        eprintln!("  Invalid address: {}", args[1]);
        0
    });

    if address == 0 && args[1] != "0" && args[1] != "0x0" {
        return;
    }

    let size: usize = match args[2].parse() {
        Ok(s) => s,
        Err(_) => {
            eprintln!("  Invalid size: {}", args[2]);
            return;
        }
    };

    if size > 4096 {
        eprintln!("  Size too large (max 4096 bytes)");
        return;
    }

    println!(
        "    Memory dump for PID {} at 0x{:x} ({} bytes)",
        pid, address, size
    );

    let mut process = Process::new(pid, format!("PID-{}", pid));
    match process.open() {
        Ok(()) => {
            if let Some(handle) = &process.handle {
                match handle.read_memory(address, size) {
                    Ok(data) => {
                        println!("\n  HEX DUMP:");
                        print_hex_dump(&data, address);

                        println!("\n   ASCII VIEW:");
                        print_ascii_view(&data);
                    }
                    Err(e) => {
                        eprintln!("  Failed to read memory: {}", e);
                        eprintln!("  Address may be invalid or inaccessible");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("   Failed to attach to process: {}", e);
            eprintln!("   Try running with sudo");
        }
    }
}

fn cmd_write_memory(args: &[String]) {
    if args.len() < 3 {
        eprintln!("   Usage: write <PID> <ADDRESS> <HEX_DATA>");
        eprintln!("   OR:     write <PID> <ADDRESS> --string <TEXT>");
        eprintln!();
        eprintln!("   HEX DATA Examples:");
        eprintln!("     write 1234 0x7fff12345678 \"48656c6c6f\"        # Write 'Hello' as hex");
        eprintln!("     write 1234 0x7fff12345678 \"64000000\"          # Write i32 value 100");
        eprintln!("     write 1234 0x7fff12345678 \"0000c842\"          # Write f32 value 100.0");
        eprintln!("     write 1234 0x7fff12345678 \"48 65 6c 6c 6f\"    # Hex with spaces");
        eprintln!();
        eprintln!("   STRING Examples:");
        eprintln!("     write 1234 0x7fff12345678 --string \"Hello\"    # Write string directly");
        eprintln!("     write 1234 0x7fff12345678 --string \"newname\"  # Write new string");
        eprintln!();
        eprintln!("      TIPS:");
        eprintln!("     • Hex values are in little-endian format");
        eprintln!("     • Use 'modify' command for easier typed value writing:");
        eprintln!("       modify 1234 0x7fff12345678 100 i32");
        eprintln!("       modify 1234 0x7fff12345678 \"newname\" string");
        eprintln!();
        eprintln!("      HEX CONVERSION REFERENCE:");
        eprintln!("     i32 100    = \"64000000\" (little-endian)");
        eprintln!("     i32 1000   = \"e8030000\"");
        eprintln!("     f32 100.0  = \"0000c842\"");
        eprintln!("     String     = ASCII hex (\"Hello\" = \"48656c6c6f\")");
        return;
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("   Invalid PID: {}", args[0]);
            return;
        }
    };

    let address: usize = if args[1].starts_with("0x") || args[1].starts_with("0X") {
        usize::from_str_radix(&args[1][2..], 16)
    } else {
        args[1].parse()
    }
    .unwrap_or_else(|_| {
        eprintln!("   Invalid address: {}", args[1]);
        0
    });

    if address == 0 && args[1] != "0" && args[1] != "0x0" {
        return;
    }

    let mut process = Process::new(pid, format!("PID-{}", pid));
    match process.open() {
        Ok(()) => {
            if let Some(handle) = &process.handle {
                if args.len() > 3 && args[2] == "--string" {
                    let text = &args[3];
                    match handle.write_string(address, text) {
                        Ok(bytes_written) => {
                            println!(
                                "    Successfully wrote {} bytes: \"{}\"",
                                bytes_written, text
                            );
                            println!("   Address: 0x{:x}", address);
                        }
                        Err(e) => {
                            eprintln!("   Failed to write string: {}", e);
                        }
                    }
                } else {
                    let hex_data = &args[2];
                    let hex_clean = hex_data.replace(" ", "").replace("\\x", "");

                    match hex::decode(&hex_clean) {
                        Ok(data) => match handle.write_memory(address, &data) {
                            Ok(bytes_written) => {
                                println!("   Successfully wrote {} bytes", bytes_written);
                                println!("   Address: 0x{:x}", address);
                                println!("   Data: {}", hex_data);
                            }
                            Err(e) => {
                                eprintln!("   Failed to write memory: {}", e);
                            }
                        },
                        Err(_) => {
                            eprintln!("   Invalid hex data: {}", hex_data);
                            eprintln!("   Use format: \"48656c6c6f\" or \"48 65 6c 6c 6f\"");
                            eprintln!();
                            eprintln!("   Examples:");
                            eprintln!("     \"48656c6c6f\"     # 'Hello' as hex");
                            eprintln!("     \"64000000\"       # i32 value 100 (little-endian)");
                            eprintln!("     \"0000c842\"       # f32 value 100.0 (little-endian)");
                            eprintln!();
                            eprintln!("      TIP: Use 'modify' command for easier value writing:");
                            eprintln!("     modify {} {} 100 i32", args[0], args[1]);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("   Failed to attach to process: {}", e);
            eprintln!("   Try running with sudo");
        }
    }
}

fn cmd_modify_value(args: &[String]) {
    if args.len() < 4 {
        eprintln!("   Usage: modify <PID> <ADDRESS> <VALUE> <TYPE>");
        eprintln!("   Types: i32, i64, f32, f64, string");
        eprintln!("   Example: modify 1234 0x7fff123456 999 i32");
        eprintln!("   Example: modify 1234 0x7fff123456 \"newname\" string");
        return;
    }

    let pid: u32 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("   Invalid PID: {}", args[0]);
            return;
        }
    };

    let address: usize = if args[1].starts_with("0x") || args[1].starts_with("0X") {
        usize::from_str_radix(&args[1][2..], 16)
    } else {
        args[1].parse()
    }
    .unwrap_or_else(|_| {
        eprintln!("   Invalid address: {}", args[1]);
        0
    });

    if address == 0 && args[1] != "0" && args[1] != "0x0" {
        return;
    }

    let value = &args[2];
    let value_type = &args[3];

    let mut process = Process::new(pid, format!("PID-{}", pid));
    match process.open() {
        Ok(()) => {
            if let Some(handle) = &process.handle {
                let result = match value_type.as_str() {
                    "i32" => match value.parse::<i32>() {
                        Ok(val) => handle.write_i32(address, val).map(|_| format!("{}", val)),
                        Err(_) => {
                            eprintln!("   Invalid i32 value: {}", value);
                            return;
                        }
                    },
                    "i64" => match value.parse::<i64>() {
                        Ok(val) => handle.write_i64(address, val).map(|_| format!("{}", val)),
                        Err(_) => {
                            eprintln!("   Invalid i64 value: {}", value);
                            return;
                        }
                    },
                    "f32" => match value.parse::<f32>() {
                        Ok(val) => handle.write_f32(address, val).map(|_| format!("{}", val)),
                        Err(_) => {
                            eprintln!("   Invalid f32 value: {}", value);
                            return;
                        }
                    },
                    "f64" => match value.parse::<f64>() {
                        Ok(val) => handle.write_f64(address, val).map(|_| format!("{}", val)),
                        Err(_) => {
                            eprintln!("   Invalid f64 value: {}", value);
                            return;
                        }
                    },
                    "string" => handle
                        .write_string(address, value)
                        .map(|_| format!("\"{}\"", value)),
                    _ => {
                        eprintln!("   Unknown type: {}", value_type);
                        eprintln!("   Valid types: i32, i64, f32, f64, string");
                        return;
                    }
                };

                match result {
                    Ok(formatted_value) => {
                        println!("    Successfully modified memory");
                        println!("   Address: 0x{:x}", address);
                        println!("   New value: {} ({})", formatted_value, value_type);
                    }
                    Err(e) => {
                        eprintln!("   Failed to modify memory: {}", e);
                        eprintln!("   Make sure the memory region is writable");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("   Failed to attach to process: {}", e);
            eprintln!("   Try running with sudo");
        }
    }
}

fn print_hex_dump(data: &[u8], base_address: usize) {
    for (i, chunk) in data.chunks(16).enumerate() {
        print!("0x{:016x}: ", base_address + i * 16);

        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                print!(" ");
            }
            print!("{:02x} ", byte);
        }

        for j in chunk.len()..16 {
            if j == 8 {
                print!(" ");
            }
            print!("   ");
        }

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
