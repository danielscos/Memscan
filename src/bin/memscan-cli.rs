// Memscan CLI - Command-line memory scanner with Real-Time Monitoring
// Usage: memscan-cli <command> [options]
// Built by the goat (danielscos)
//
//===============================================================================================
//===============================================================================================

//
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

//===============================================================================================
//===============================================================================================

use memscan::{
    memory_optimization::get_allocated_bytes,
    monitor::{DataType, MemoryMonitor},
    process::{Process, ProcessHandle, enumerate_processes},
    scanner::{
        scan_process_for_f32, scan_process_for_f64, scan_process_for_i32, scan_process_for_i64,
        scan_process_for_string,
    },
    utils::{display_system_info, loading_with_checks, suggest_fixes},
};
use std::env;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

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

// Global state for monitoring (thread-safe)
lazy_static::lazy_static! {
    static ref GLOBAL_MONITOR: Arc<Mutex<Option<MemoryMonitor>>> = Arc::new(Mutex::new(None));
    static ref GLOBAL_PROCESS_HANDLE: Arc<Mutex<Option<Arc<ProcessHandle>>>> = Arc::new(Mutex::new(None));
}

fn print_banner_and_initialize() -> Result<(), String> {
    print!("{}", DANIELSCOS_BANNER);

    match loading_with_checks() {
        Ok(system_info) => {
            display_system_info(&system_info);

            if !system_info.has_sudo || system_info.ptrace_scope.unwrap_or(0) > 1 {
                suggest_fixes(&system_info);
            }

            println!("\n Memscan ready for operation");
            Ok(())
        }
        Err(error) => {
            eprintln!("\n Initialization failed: {}", error);
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
    println!(" CLI started. Type 'help' for commands or 'exit' to quit.");

    let stdin = io::stdin();

    loop {
        // Show memory usage and monitor status in prompt
        let memory_mb = get_allocated_bytes() as f64 / 1024.0 / 1024.0;
        let monitor_status = {
            let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
            if let Some(ref monitor) = *monitor_guard {
                if monitor.is_running() { "🔴" } else { "⚪" }
            } else {
                "⚫"
            }
        };

        print!("memscan({:.1}MB){} > ", memory_mb, monitor_status);
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
                    // Clean shutdown - stop monitoring if running
                    cmd_stop_monitor();
                    println!("👋 Au Revoir");
                    break;
                }

                let string_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
                execute_command(&string_args);

                println!();
            }
            Err(e) => {
                eprintln!(" Error reading input: {}", e);
                break;
            }
        }
    }
}

fn execute_command(args: &[String]) {
    match args[0].as_str() {
        // Original commands
        "help" | "--help" | "-h" => print_usage(),
        "version" | "--version" | "-v" => print_version(),
        "list" => cmd_list_processes(),
        "scan" => cmd_scan(args),
        "info" => cmd_process_info(args),
        "dump" => cmd_dump_memory(args),
        "write" => cmd_write_memory(args),
        "modify" => cmd_modify_value(args),

        // New real-time monitoring commands
        "monitor" => cmd_start_monitor(args),
        "watch" => cmd_add_watch(args),
        "unwatch" => cmd_remove_watch(args),
        "changes" => cmd_show_changes(args),
        "status" => cmd_monitor_status(),
        "stop" | "stop-monitor" => cmd_stop_monitor(),
        "targets" => cmd_list_targets(),
        "auto-monitor" => cmd_auto_scan_and_monitor(args),
        "live" => cmd_live_monitor(),
        "notifications" => cmd_toggle_notifications(args),
        "interval" => cmd_set_interval(args),
        "debug" => cmd_debug_monitoring(args),

        _ => {
            println!(" Unknown command: '{}'", args[0]);
            println!(" Type 'help' to see available commands");
        }
    }
}

fn print_version() {
    println!("Memscan v0.4.0 - High Performance Memory Scanner with Real-Time Monitoring");
}

fn print_usage() {
    println!(" MEMSCAN CLI - High Performance Memory Scanner");
    println!();
    println!("BASIC COMMANDS:");
    println!("  help                                 Show this help message");
    println!("  version                              Show version information");
    println!("  list                                 List running processes");
    println!("  scan <PID> <VALUE> <TYPE>           Scan for value in process memory");
    println!("  info <PID>                          Show process memory information");
    println!("  dump <PID> <ADDRESS> <SIZE>         Dump memory region as hex");
    println!("  write <PID> <ADDRESS> <VALUE> <TYPE> Write value to memory");
    println!("  modify <PID> <ADDRESS> <VALUE> <TYPE> Safe modify with validation");
    println!();
    println!("REAL TIME MONITORING:");
    println!("  monitor <PID> [interval_ms]         Start real-time monitoring (default: 100ms)");
    println!("  watch <ADDRESS> <TYPE> <NAME>       Add memory address to watch list");
    println!("  unwatch <ADDRESS>                   Remove address from monitoring");
    println!("  changes [count]                     Show recent memory changes");
    println!("  targets                             List all monitored addresses");
    println!("  status                              Show monitoring status and statistics");
    println!("  stop                                Stop monitoring");
    println!("  auto-monitor <VALUE> <TYPE>         Scan current process and auto-add targets");
    println!(
        "  live                                Show live memory changes (press Ctrl+C to exit)"
    );
    println!("  notifications <on|off>              Enable/disable automatic change notifications");
    println!(
        "  interval <ms>                       Set monitoring update interval (default: 100ms)"
    );
    println!("  debug <on|off>                      Enable debug output for troubleshooting");
    println!();
    println!("DATA TYPES:");
    println!("  i32, i64, f32, f64, string, string:<size>");
    println!();
    println!("EXAMPLES:");
    println!("   Basic Usage:");
    println!("    list                               # List processes");
    println!("    scan 1234 100 i32                 # Find integer 100 in process 1234");
    println!("    info 1234                         # Show memory layout");
    println!();
    println!("   Real-Time Monitoring:");
    println!("    monitor 1234 50                   # Start monitoring every 50ms");
    println!("    watch 0x7fff12345678 i32 health   # Watch health variable");
    println!("    watch 0x7fff87654321 string:32 name # Watch player name");
    println!("    changes 10                        # Show last 10 changes");
    println!("    auto-monitor 100 i32               # Auto-find and monitor current process");
    println!("    live                              # Watch changes in real-time");
    println!("    notifications on                  # Enable auto-notifications");
    println!(
        "    interval 200                      # Set 200ms update interval (slower = more stable)"
    );
    println!("    debug on                          # Show detailed timing info");
    println!();
    println!("  ⚡ Advanced:");
    println!("    write 1234 0x7fff12345678 999 i32 # Write value to address");
    println!("    dump 1234 0x7fff12345678 64       # Hex dump 64 bytes");
    println!();
    println!("MONITOR STATUS INDICATORS:");
    println!("  ⚫ No monitor running    🔴 Monitor active    ⚪ Monitor stopped");
    println!();
    println!(" Pro Tips:");
    println!("  - Use 'auto-monitor' to quickly find and monitor values");
    println!("  - Monitor shows live changes as they happen");
    println!("  - Memory usage is displayed in the prompt");
    println!("  - Use Ctrl+C in interactive mode to stop gracefully");
    println!();
    println!(" TROUBLESHOOTING:");
    println!("  - Seeing garbled strings? Try 'interval 200' for slower monitoring");
    println!("  - Getting partial changes? Use 'debug on' to see timing details");
    println!("  - Fast intervals (< 100ms) may catch writes in progress");
}

fn cmd_list_processes() {
    println!(" Enumerating processes...");

    match enumerate_processes() {
        Ok(processes) => {
            if processes.is_empty() {
                println!("  No processes found or insufficient privileges");
                println!(" Try running with sudo");
                return;
            }

            println!(" Found {} accessible processes:", processes.len());
            println!("{:-<60}", "");

            for (i, process) in processes.iter().enumerate() {
                if i < 30 {
                    // Show first 30 processes
                    println!("  {:3}. {} (PID: {})", i + 1, process.name, process.pid);
                } else if i == 30 {
                    println!("  ... and {} more processes", processes.len() - 30);
                    break;
                }
            }

            println!("{:-<60}", "");
            println!(" Use 'scan <PID> <VALUE> <TYPE>' to search process memory");
            println!(" Use 'monitor <PID>' to start real-time monitoring");
        }
        Err(e) => {
            println!(" Failed to enumerate processes: {}", e);
            println!(" Try running with elevated privileges");
        }
    }
}

fn cmd_scan(args: &[String]) {
    if args.len() < 4 {
        println!(" Usage: scan <PID> <VALUE> <TYPE>");
        println!(" Types: i32, i64, f32, f64, string");
        println!(" Example: scan 1234 100 i32");
        return;
    }

    let pid: u32 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            println!(" Invalid PID: {}", args[1]);
            return;
        }
    };

    let value_str = &args[2];
    let data_type = &args[3];

    println!(
        " Scanning process {} for value '{}' of type {}",
        pid, value_str, data_type
    );

    // Get process
    let mut process = match get_process_by_pid(pid) {
        Some(p) => p,
        None => {
            println!(" Process {} not found", pid);
            println!(" Use 'list' to see available processes");
            return;
        }
    };

    // Open process
    if let Err(e) = process.open() {
        println!(" Failed to open process {}: {}", pid, e);
        println!(" Try running with sudo");
        return;
    }

    let handle = match process.handle {
        Some(ref h) => h,
        None => {
            println!(" Failed to get process handle");
            return;
        }
    };

    // Perform scan based on type
    let scan_result = match data_type.as_str() {
        "i32" => scan_process_for_i32(handle, value_str),
        "i64" => scan_process_for_i64(handle, value_str),
        "f32" => scan_process_for_f32(handle, value_str),
        "f64" => scan_process_for_f64(handle, value_str),
        "string" => scan_process_for_string(handle, value_str),
        _ => {
            println!(" Invalid type: {}", data_type);
            println!(" Valid types: i32, i64, f32, f64, string");
            return;
        }
    };

    match scan_result {
        Ok(results) => {
            if results.is_empty() {
                println!(" No matches found");
                println!(" Try a different value or data type");
            } else {
                println!(" Found {} matches:", results.len());
                println!("{:-<50}", "");

                for (i, result) in results.iter().enumerate() {
                    if i < 20 {
                        // Show first 20 results
                        println!("  {:2}. 0x{:x}", i + 1, result.address);
                    } else if i == 20 {
                        println!("  ... and {} more matches", results.len() - 20);
                        break;
                    }
                }

                println!("{:-<50}", "");
                println!(" Use 'write <PID> <ADDRESS> <VALUE> <TYPE>' to modify");
                println!(
                    " Use 'monitor <PID>' then 'watch <ADDRESS> <TYPE> <NAME>' for real-time tracking"
                );
            }
        }
        Err(e) => {
            println!(" Scan failed: {}", e);
        }
    }
}

fn cmd_process_info(args: &[String]) {
    if args.len() < 2 {
        println!(" Usage: info <PID>");
        println!(" Example: info 1234");
        return;
    }

    let pid: u32 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            println!(" Invalid PID: {}", args[1]);
            return;
        }
    };

    println!(" Gathering information for process {}...", pid);

    let mut process = match get_process_by_pid(pid) {
        Some(p) => p,
        None => {
            println!(" Process {} not found", pid);
            return;
        }
    };

    if let Err(e) = process.open() {
        println!(" Failed to open process: {}", e);
        return;
    }

    let handle = match process.handle {
        Some(ref h) => h,
        None => {
            println!(" Failed to get process handle");
            return;
        }
    };

    println!(" Process Information:");
    println!("{:=<50}", "");
    println!("Name: {}", process.name);
    println!("PID:  {}", process.pid);

    match handle.get_memory_regions() {
        Ok(regions) => {
            let total_regions = regions.len();
            let scannable_regions: Vec<_> = regions
                .iter()
                .filter(|r| r.readable && r.size > 1024)
                .collect();
            let total_memory: usize = regions.iter().map(|r| r.size).sum();

            println!("Memory Regions: {}", total_regions);
            println!("Scannable:      {}", scannable_regions.len());
            println!(
                "Total Memory:   {:.2} MB",
                total_memory as f64 / 1024.0 / 1024.0
            );

            println!("\n📍 Memory Layout (first 10 regions):");
            println!("{:-<70}", "");
            println!(
                "{:<16} {:>10} {:>10} {:>10}",
                "Address", "Size", "Perms", "Type"
            );
            println!("{:-<70}", "");

            for (i, region) in regions.iter().enumerate() {
                if i >= 10 {
                    println!("... and {} more regions", total_regions - 10);
                    break;
                }

                let perms = format!(
                    "{}{}{}",
                    if region.readable { "r" } else { "-" },
                    if region.writable { "w" } else { "-" },
                    if region.executable { "x" } else { "-" }
                );

                let size_str = if region.size > 1024 * 1024 {
                    format!("{:.1}MB", region.size as f64 / 1024.0 / 1024.0)
                } else if region.size > 1024 {
                    format!("{:.1}KB", region.size as f64 / 1024.0)
                } else {
                    format!("{}B", region.size)
                };

                println!(
                    "0x{:012x} {:>10} {:>10} {:>10}",
                    region.start_address,
                    size_str,
                    perms,
                    if region.readable && region.size > 1024 {
                        "Scannable"
                    } else {
                        ""
                    }
                );
            }
        }
        Err(e) => {
            println!(" Failed to get memory regions: {}", e);
        }
    }
}

fn cmd_dump_memory(args: &[String]) {
    if args.len() < 4 {
        println!(" Usage: dump <PID> <ADDRESS> <SIZE>");
        println!(" Example: dump 1234 0x7fff12345678 64");
        return;
    }

    let pid: u32 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            println!(" Invalid PID: {}", args[1]);
            return;
        }
    };

    let address_str = args[2].trim_start_matches("0x");
    let address = match usize::from_str_radix(address_str, 16) {
        Ok(addr) => addr,
        Err(_) => {
            println!(" Invalid address: {}", args[2]);
            return;
        }
    };

    let size: usize = match args[3].parse() {
        Ok(s) => s,
        Err(_) => {
            println!(" Invalid size: {}", args[3]);
            return;
        }
    };

    if size > 4096 {
        println!("  Size limited to 4096 bytes for display purposes");
        return;
    }

    let mut process = match get_process_by_pid(pid) {
        Some(p) => p,
        None => {
            println!(" Process {} not found", pid);
            return;
        }
    };

    if let Err(e) = process.open() {
        println!(" Failed to open process: {}", e);
        return;
    }

    let handle = match process.handle {
        Some(ref h) => h,
        None => {
            println!(" Failed to get process handle");
            return;
        }
    };

    println!(" Reading {} bytes from 0x{:x}...", size, address);

    match handle.read_memory(address, size) {
        Ok(data) => {
            println!(" Memory dump:");
            print_hex_dump(&data, address);
            print_ascii_view(&data);
        }
        Err(e) => {
            println!(" Failed to read memory: {}", e);
        }
    }
}

fn cmd_write_memory(args: &[String]) {
    if args.len() < 5 {
        println!(" Usage: write <PID> <ADDRESS> <VALUE> <TYPE>");
        println!(" Types: i32, i64, f32, f64, string");
        println!(" Example: write 1234 0x7fff12345678 999 i32");
        return;
    }

    let pid: u32 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            println!(" Invalid PID: {}", args[1]);
            return;
        }
    };

    let address_str = args[2].trim_start_matches("0x");
    let address = match usize::from_str_radix(address_str, 16) {
        Ok(addr) => addr,
        Err(_) => {
            println!(" Invalid address: {}", args[2]);
            return;
        }
    };

    let value_str = &args[3];
    let data_type = &args[4];

    let mut process = match get_process_by_pid(pid) {
        Some(p) => p,
        None => {
            println!(" Process {} not found", pid);
            return;
        }
    };

    if let Err(e) = process.open() {
        println!(" Failed to open process: {}", e);
        return;
    }

    let handle = match process.handle {
        Some(ref h) => h,
        None => {
            println!(" Failed to get process handle");
            return;
        }
    };

    println!(
        "✏️  Writing '{}' as {} to 0x{:x}...",
        value_str, data_type, address
    );

    let write_result = match data_type.as_str() {
        "i32" => match value_str.parse::<i32>() {
            Ok(val) => handle.write_i32(address, val),
            Err(_) => {
                println!(" Invalid i32 value: {}", value_str);
                return;
            }
        },
        "i64" => match value_str.parse::<i64>() {
            Ok(val) => handle.write_i64(address, val),
            Err(_) => {
                println!(" Invalid i64 value: {}", value_str);
                return;
            }
        },
        "f32" => match value_str.parse::<f32>() {
            Ok(val) => handle.write_f32(address, val),
            Err(_) => {
                println!(" Invalid f32 value: {}", value_str);
                return;
            }
        },
        "f64" => match value_str.parse::<f64>() {
            Ok(val) => handle.write_f64(address, val),
            Err(_) => {
                println!(" Invalid f64 value: {}", value_str);
                return;
            }
        },
        "string" => {
            // For strings, we need to handle proper null termination and clearing
            // First read the current value to determine how much to clear
            match handle.read_memory(address, 256) {
                Ok(current_data) => {
                    // Find the current string length (up to null terminator)
                    let current_len = current_data.iter().position(|&b| b == 0).unwrap_or(256);

                    // Create new data with null termination and clearing
                    let new_bytes = value_str.as_bytes();
                    let new_len = new_bytes.len();

                    if new_len < current_len {
                        // New string is shorter, need to clear the remaining bytes
                        let mut write_data = vec![0u8; current_len + 1];
                        write_data[..new_len].copy_from_slice(new_bytes);
                        // write_data already has null bytes for the rest
                        handle.write_memory(address, &write_data)
                    } else {
                        // New string is same length or longer, just write with null terminator
                        let mut write_data = new_bytes.to_vec();
                        write_data.push(0);
                        handle.write_memory(address, &write_data)
                    }
                }
                Err(_) => {
                    // Fallback to simple null-terminated write
                    let mut write_data = value_str.as_bytes().to_vec();
                    write_data.push(0);
                    handle.write_memory(address, &write_data)
                }
            }
        }
        _ => {
            println!(" Invalid type: {}", data_type);
            return;
        }
    };

    match write_result {
        Ok(bytes_written) => {
            println!(" Successfully wrote {} bytes", bytes_written);
        }
        Err(e) => {
            println!(" Write failed: {}", e);
        }
    }
}

fn cmd_modify_value(args: &[String]) {
    if args.len() < 5 {
        println!(" Usage: modify <PID> <ADDRESS> <VALUE> <TYPE>");
        println!(" This command verifies the write was successful");
        return;
    }

    let pid: u32 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            println!(" Invalid PID: {}", args[1]);
            return;
        }
    };

    let address_str = args[2].trim_start_matches("0x");
    let address = match usize::from_str_radix(address_str, 16) {
        Ok(addr) => addr,
        Err(_) => {
            println!(" Invalid address: {}", args[2]);
            return;
        }
    };

    let value_str = &args[3];
    let data_type = &args[4];

    let mut process = match get_process_by_pid(pid) {
        Some(p) => p,
        None => {
            println!(" Process {} not found", pid);
            return;
        }
    };

    if let Err(e) = process.open() {
        println!(" Failed to open process: {}", e);
        return;
    }

    let handle = match process.handle {
        Some(ref h) => h,
        None => {
            println!(" Failed to get process handle");
            return;
        }
    };

    // First check if memory is writable
    match handle.is_writable(address) {
        Ok(true) => {}
        Ok(false) => {
            println!("  Warning: Memory region may not be writable");
        }
        Err(e) => {
            println!("  Warning: Could not verify memory permissions: {}", e);
        }
    }

    // Read original value
    let size = match data_type.as_str() {
        "i32" | "f32" => 4,
        "i64" | "f64" => 8,
        "string" => value_str.len(),
        _ => {
            println!(" Invalid type: {}", data_type);
            return;
        }
    };

    let original_data = match handle.read_memory(address, size) {
        Ok(data) => data,
        Err(e) => {
            println!(" Failed to read original value: {}", e);
            return;
        }
    };

    println!("📖 Original value: {:02x?}", original_data);

    // Write new value
    let write_result = match data_type.as_str() {
        "i32" => match value_str.parse::<i32>() {
            Ok(val) => handle.write_i32(address, val),
            Err(_) => {
                println!(" Invalid i32 value: {}", value_str);
                return;
            }
        },
        "i64" => match value_str.parse::<i64>() {
            Ok(val) => handle.write_i64(address, val),
            Err(_) => {
                println!(" Invalid i64 value: {}", value_str);
                return;
            }
        },
        "f32" => match value_str.parse::<f32>() {
            Ok(val) => handle.write_f32(address, val),
            Err(_) => {
                println!(" Invalid f32 value: {}", value_str);
                return;
            }
        },
        "f64" => match value_str.parse::<f64>() {
            Ok(val) => handle.write_f64(address, val),
            Err(_) => {
                println!(" Invalid f64 value: {}", value_str);
                return;
            }
        },
        "string" => {
            // For strings, we need to handle proper null termination and clearing
            // First read the current value to determine how much to clear
            match handle.read_memory(address, 256) {
                Ok(current_data) => {
                    // Find the current string length (up to null terminator)
                    let current_len = current_data.iter().position(|&b| b == 0).unwrap_or(256);

                    // Create new data with null termination and clearing
                    let new_bytes = value_str.as_bytes();
                    let new_len = new_bytes.len();

                    if new_len < current_len {
                        // New string is shorter, need to clear the remaining bytes
                        let mut write_data = vec![0u8; current_len + 1];
                        write_data[..new_len].copy_from_slice(new_bytes);
                        // write_data already has null bytes for the rest
                        handle.write_memory(address, &write_data)
                    } else {
                        // New string is same length or longer, just write with null terminator
                        let mut write_data = new_bytes.to_vec();
                        write_data.push(0);
                        handle.write_memory(address, &write_data)
                    }
                }
                Err(_) => {
                    // Fallback to simple null-terminated write
                    let mut write_data = value_str.as_bytes().to_vec();
                    write_data.push(0);
                    handle.write_memory(address, &write_data)
                }
            }
        }
        _ => {
            println!(" Invalid type: {}", data_type);
            return;
        }
    };

    match write_result {
        Ok(bytes_written) => {
            println!("✏️  Wrote {} bytes", bytes_written);

            // Verify the write
            match handle.read_memory(address, size) {
                Ok(new_data) => {
                    println!("📖 New value: {:02x?}", new_data);
                    if new_data != original_data {
                        println!(" Value successfully modified");
                    } else {
                        println!("  Warning: Value appears unchanged after write");
                    }
                }
                Err(e) => {
                    println!("  Could not verify write: {}", e);
                }
            }
        }
        Err(e) => {
            println!(" Write failed: {}", e);
        }
    }
}

fn cmd_start_monitor(args: &[String]) {
    if args.len() < 2 {
        println!(" Usage: monitor <PID> [update_interval_ms]");
        println!(" Example: monitor 1234 100  # Monitor every 100ms");
        println!(" Quick start: Use 'auto-monitor <PID> <VALUE> <TYPE>' for instant results");
        return;
    }

    let pid: u32 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            println!(" Invalid PID: {}", args[1]);
            return;
        }
    };

    let interval_ms = if args.len() > 2 {
        args[2].parse().unwrap_or(200)
    } else {
        200 // Default 200ms updates (more stable)
    };

    // Validate interval and warn about race conditions
    if interval_ms < 50 {
        println!("  Warning: Fast intervals may cause race conditions");
        println!("   You might see partial/garbled string changes");
        println!("   Recommended: 100ms+ for stable monitoring");
        println!("   Use 'interval 200' to change after starting");
    } else if interval_ms >= 100 {
        println!(" Using stable interval: {}ms", interval_ms);
    }

    // Get process handle
    let mut process = match get_process_by_pid(pid) {
        Some(p) => p,
        None => {
            println!(" Process {} not found", pid);
            println!(" Use 'list' to see available processes");
            return;
        }
    };

    println!(" Attaching to process {} ({})...", pid, process.name);

    if let Err(e) = process.open() {
        println!(" Failed to open process {}: {}", pid, e);
        println!(" Try running with sudo or ./run_memscan.sh");
        return;
    }

    let handle = match process.handle {
        Some(h) => Arc::new(h),
        None => {
            println!(" Failed to get process handle");
            return;
        }
    };

    // Stop existing monitor if running
    cmd_stop_monitor();

    // Create and start new monitor
    let monitor = MemoryMonitor::new(interval_ms);

    if let Err(e) = monitor.start_monitoring(Arc::clone(&handle)) {
        println!(" Failed to start monitoring: {}", e);
        return;
    }

    // Store in global state
    {
        let mut monitor_guard = GLOBAL_MONITOR.lock().unwrap();
        *monitor_guard = Some(monitor);
    }
    {
        let mut handle_guard = GLOBAL_PROCESS_HANDLE.lock().unwrap();
        *handle_guard = Some(handle);
    }

    println!(" Real-time monitoring started");
    println!(" Process: {} (PID: {})", process.name, pid);
    println!();
    println!(" NEXT STEPS:");
    println!("  1️⃣  Add watch targets: 'watch <address> <type> <name>'");
    println!("  2️⃣  Changes to the watch targets will appear automatically.");
    println!();
    println!(" QUICK OPTIONS:");
    println!(
        "  • Use 'auto-monitor <VALUE> <TYPE>' to automatically find the addresses and auto-watch them"
    );
    println!("  • Live notifications are automatically enabled");
    println!();
    println!(" Example: watch 0x7fff12345678 i32 health");
    println!(" If you see garbled strings, try: interval 300");

    // Automatically enable live notifications for better UX
    {
        let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
        if let Some(ref monitor) = *monitor_guard {
            monitor.enable_live_notifications();
        }
    }
}

fn cmd_add_watch(args: &[String]) {
    if args.len() < 4 {
        println!(" Usage: watch <address> <type> <name>");
        println!(" Types: i32, i64, f32, f64, string:<size>");
        println!(" Examples:");
        println!("   watch 0x7fff12345678 i32 health");
        println!("   watch 0x7fff87654321 string:32 username");
        println!("   watch 0x7fff99999999 f32 player_x");
        return;
    }

    let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
    let monitor = match monitor_guard.as_ref() {
        Some(m) => {
            if !m.is_running() {
                println!(" Monitor exists but is not running");
                println!(" Start monitoring first with 'monitor <PID>'");
                return;
            }
            m
        }
        None => {
            println!(" No monitor running");
            println!(" Start monitoring first with 'monitor <PID>' command");
            return;
        }
    };

    // Parse address (support both 0x and plain hex)
    let address_str = args[1].trim_start_matches("0x");
    let address = match usize::from_str_radix(address_str, 16) {
        Ok(addr) => addr,
        Err(_) => {
            println!(" Invalid address: {}", args[1]);
            println!(" Use format: 0x7fff12345678 or 7fff12345678");
            return;
        }
    };

    // Parse data type
    let data_type = match args[2].as_str() {
        "i32" => DataType::I32,
        "i64" => DataType::I64,
        "f32" => DataType::F32,
        "f64" => DataType::F64,
        "string" => DataType::String(256), // Default string size
        s if s.starts_with("string:") => {
            let size_str = &s[7..];
            match size_str.parse::<usize>() {
                Ok(size) if size > 0 && size <= 1024 => DataType::String(size),
                Ok(_) => {
                    println!(" String size must be between 1 and 1024 bytes");
                    return;
                }
                Err(_) => {
                    println!(" Invalid string size: {}", size_str);
                    return;
                }
            }
        }
        _ => {
            println!(" Invalid type: '{}'", args[2]);
            println!(" Valid types: i32, i64, f32, f64, string, string:<size>");
            return;
        }
    };

    let name = args[3].to_string();

    // Add target to monitor
    monitor.add_target(address, data_type, name.clone());
    println!(" Added '{}' to watch list", name);
    println!("📍 Address: 0x{:x}", address);
    println!(" Type: {}", args[2]);
    println!("🔴 Live notifications active - changes will appear automatically");
}

fn cmd_remove_watch(args: &[String]) {
    if args.len() < 2 {
        println!(" Usage: unwatch <address>");
        println!(" Example: unwatch 0x7fff12345678");
        return;
    }

    let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
    let monitor = match monitor_guard.as_ref() {
        Some(m) => m,
        None => {
            println!(" No monitor running");
            return;
        }
    };

    let address_str = args[1].trim_start_matches("0x");
    let address = match usize::from_str_radix(address_str, 16) {
        Ok(addr) => addr,
        Err(_) => {
            println!(" Invalid address: {}", args[1]);
            return;
        }
    };

    monitor.remove_target(address);
    println!(" Removed address 0x{:x} from watch list", address);
}

fn cmd_show_changes(args: &[String]) {
    let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
    let monitor = match monitor_guard.as_ref() {
        Some(m) => m,
        None => {
            println!(" No monitor running");
            return;
        }
    };

    let count = if args.len() > 1 {
        args[1].parse().unwrap_or(10)
    } else {
        10
    };

    let changes = monitor.get_recent_changes(count);

    if changes.is_empty() {
        println!(" No recent changes detected");
        println!(" Changes will appear here when monitored values change");
        return;
    }

    println!(" Recent Memory Changes (last {}):", changes.len());
    println!("{:=^80}", "");

    for (i, change) in changes.iter().enumerate() {
        let elapsed = change.timestamp.elapsed();
        println!(
            " #{} {} (0x{:x}) - {:.2}s ago",
            i + 1,
            change.name,
            change.address,
            elapsed.as_secs_f64()
        );

        // Display based on the actual data type stored with the target
        match change.data_type {
            memscan::monitor::DataType::I32 => {
                let old_i32 = i32::from_le_bytes(change.old_value[..4].try_into().unwrap());
                let new_i32 = i32::from_le_bytes(change.new_value[..4].try_into().unwrap());
                println!("    {} → {} (i32)", old_i32, new_i32);
            }
            memscan::monitor::DataType::I64 => {
                let old_i64 = i64::from_le_bytes(change.old_value[..8].try_into().unwrap());
                let new_i64 = i64::from_le_bytes(change.new_value[..8].try_into().unwrap());
                println!("    {} → {} (i64)", old_i64, new_i64);
            }
            memscan::monitor::DataType::F32 => {
                let old_f32 = f32::from_le_bytes(change.old_value[..4].try_into().unwrap());
                let new_f32 = f32::from_le_bytes(change.new_value[..4].try_into().unwrap());
                println!("    {:.3} → {:.3} (f32)", old_f32, new_f32);
            }
            memscan::monitor::DataType::F64 => {
                let old_f64 = f64::from_le_bytes(change.old_value[..8].try_into().unwrap());
                let new_f64 = f64::from_le_bytes(change.new_value[..8].try_into().unwrap());
                println!("    {:.6} → {:.6} (f64)", old_f64, new_f64);
            }
            memscan::monitor::DataType::String(_) => {
                // String or other data - show accurate string changes
                display_string_change_cli(change);
            }
        }
        println!();
    }
}

fn display_string_change_cli(change: &memscan::monitor::MonitorChange) {
    // Find the actual null-terminated strings
    let old_str_end = change
        .old_value
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(change.old_value.len());
    let new_str_end = change
        .new_value
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(change.new_value.len());

    let old_bytes = &change.old_value[..old_str_end];
    let new_bytes = &change.new_value[..new_str_end];

    // Try to create clean ASCII strings
    let old_clean: String = old_bytes
        .iter()
        .filter(|&&b| b.is_ascii_graphic() || b == b' ')
        .map(|&b| b as char)
        .collect();
    let new_clean: String = new_bytes
        .iter()
        .filter(|&&b| b.is_ascii_graphic() || b == b' ')
        .map(|&b| b as char)
        .collect();

    // Show the change
    if !old_clean.is_empty() || !new_clean.is_empty() {
        println!("    '{}' → '{}'", old_clean, new_clean);

        // Show byte lengths if different
        if old_str_end != new_str_end {
            println!("    Length: {} → {} bytes", old_str_end, new_str_end);
        }
    } else {
        // Fallback to hex if no printable characters
        println!("    {:02x?} → {:02x?}", old_bytes, new_bytes);
    }
}

fn cmd_monitor_status() {
    let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
    let monitor = match monitor_guard.as_ref() {
        Some(m) => m,
        None => {
            println!("⚫ No monitor initialized");
            println!(" Start monitoring with 'monitor <PID>' command");
            return;
        }
    };

    if !monitor.is_running() {
        println!("⚪ Monitor exists but is not running");
        println!(" Use 'monitor <PID>' to restart");
        return;
    }

    let targets = monitor.get_targets_status();
    let memory_usage = get_allocated_bytes() as f64 / 1024.0 / 1024.0;

    println!("🔴 Monitor Status: ACTIVE");
    println!("{:=^60}", "");
    println!(" Monitored Targets: {}", targets.len());
    println!();

    if targets.is_empty() {
        println!("  Monitor is running but no targets are being watched");
        println!(" No memory changes will be detected until you add targets.");
        println!();
        println!(" QUICK START OPTIONS:");
        println!("  • 'auto-monitor <VALUE> <TYPE>' - Auto-find and watch addresses");
        println!("  • 'watch <ADDRESS> <TYPE> <NAME>' - Watch specific address");
        println!();
        println!(" EXAMPLES:");
        println!("  auto-monitor 100 i32           # Find all instances of 100");
        println!("  watch 0x7fff12345678 i32 health # Watch specific address");
        println!("  notifications on               # Enable auto-notifications");
        println!();
        println!(" Then changes will appear automatically");
        return;
    }

    println!(" Monitoring Targets:");
    println!("{:-^60}", "");

    for (address, name, change_count, last_changed) in targets {
        let last_change_info = if let Some(timestamp) = last_changed {
            let elapsed = timestamp.elapsed();
            if elapsed.as_secs() < 60 {
                format!("{:.1}s ago", elapsed.as_secs_f64())
            } else {
                format!("{}m ago", elapsed.as_secs() / 60)
            }
        } else {
            "Never".to_string()
        };

        println!(" {} (0x{:x})", name, address);
        println!(
            "    {} changes | 🕒 Last: {}",
            change_count, last_change_info
        );
        println!();
    }
}

fn cmd_stop_monitor() {
    let mut monitor_guard = GLOBAL_MONITOR.lock().unwrap();

    if let Some(ref monitor) = *monitor_guard {
        monitor.stop_monitoring();
        println!("  Monitor stopped");
    }

    *monitor_guard = None;

    // Clear process handle too
    {
        let mut handle_guard = GLOBAL_PROCESS_HANDLE.lock().unwrap();
        *handle_guard = None;
    }
}

fn cmd_list_targets() {
    let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
    let monitor = match monitor_guard.as_ref() {
        Some(m) => m,
        None => {
            println!(" No monitor running");
            return;
        }
    };

    let targets = monitor.get_targets_status();

    if targets.is_empty() {
        println!(" No targets are being monitored");
        println!(" Use 'watch <address> <type> <name>' to add targets");
        return;
    }

    println!(" Monitored Targets ({}):", targets.len());
    println!("{:-^70}", "");

    for (address, name, change_count, _) in targets {
        println!("📍 {} at 0x{:x} ({} changes)", name, address, change_count);
    }
}

fn cmd_auto_scan_and_monitor(args: &[String]) {
    if args.len() < 3 {
        println!(" Usage: auto-monitor <VALUE> <TYPE>");
        println!(" This scans the currently monitored process and auto-adds found addresses");
        println!(" Example: auto-monitor 100 i32");
        println!("  Note: Start monitoring with 'monitor <PID>' first");
        return;
    }

    // Check if monitor is running and get handle
    let handle = {
        let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
        let handle_guard = GLOBAL_PROCESS_HANDLE.lock().unwrap();

        match (monitor_guard.as_ref(), handle_guard.as_ref()) {
            (Some(m), Some(h)) if m.is_running() => Arc::clone(h),
            _ => {
                println!(" No active monitoring session");
                println!(" Start monitoring first: 'monitor <PID>'");
                return;
            }
        }
    };

    let value_str = &args[1];
    let data_type = &args[2];

    println!(
        " Auto-scanning for '{}' as {} and setting up monitoring...",
        value_str, data_type
    );

    // Perform scan using the monitored process handle
    let scan_result = match data_type.as_str() {
        "i32" => scan_process_for_i32(&handle, value_str),
        "i64" => scan_process_for_i64(&handle, value_str),
        "f32" => scan_process_for_f32(&handle, value_str),
        "f64" => scan_process_for_f64(&handle, value_str),
        "string" => scan_process_for_string(&handle, value_str),
        _ => {
            println!(" Invalid type: {}", data_type);
            println!(" Valid types: i32, i64, f32, f64, string");
            return;
        }
    };

    match scan_result {
        Ok(results) => {
            if results.is_empty() {
                println!(" No matches found for '{}'", value_str);
                return;
            }

            {
                let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
                if let Some(ref monitor) = *monitor_guard {
                    let add_count = std::cmp::min(results.len(), 10); // Limit to first 10 matches

                    println!(
                        " Found {} matches, adding first {} to monitoring:",
                        results.len(),
                        add_count
                    );

                    let data_type_enum = match data_type.as_str() {
                        "i32" => DataType::I32,
                        "i64" => DataType::I64,
                        "f32" => DataType::F32,
                        "f64" => DataType::F64,
                        "string" => DataType::String(256),
                        _ => return,
                    };

                    for (i, result) in results.iter().take(add_count).enumerate() {
                        let name = format!("{}_{}", value_str, i + 1);
                        monitor.add_target(result.address, data_type_enum.clone(), name);
                        println!("  📍 Added: 0x{:x}", result.address);
                    }

                    if results.len() > 10 {
                        println!(
                            " {} additional matches found but not monitored (limit: 10)",
                            results.len() - 10
                        );
                        println!(" Use 'watch <address> <type> <name>' to add specific addresses");
                    }

                    println!();
                    println!(" Auto-monitoring setup complete");

                    // Automatically enable notifications for better UX
                    monitor.enable_live_notifications();

                    println!(" Use 'status' to see monitoring overview");
                    println!(" Use 'notifications off' to disable auto-notifications");
                }
            }
        }
        Err(e) => {
            println!(" Scan failed: {}", e);
        }
    }
}

fn cmd_live_monitor() {
    let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
    let monitor = match monitor_guard.as_ref() {
        Some(m) => {
            if !m.is_running() {
                println!(" Monitor is not running");
                println!(" Start monitoring first with 'monitor <PID>'");
                return;
            }
            m
        }
        None => {
            println!(" No monitor running");
            println!(" Start monitoring first with 'monitor <PID>' command");
            return;
        }
    };

    let targets = monitor.get_targets_status();
    if targets.is_empty() {
        println!("  No targets being monitored");
        println!(
            " Add targets with 'watch <address> <type> <name>' or 'auto-monitor <value> <type>'"
        );
        return;
    }

    println!("🔴 LIVE MONITORING MODE");
    println!(
        " Monitoring {} targets - Press Ctrl+C to exit",
        targets.len()
    );
    println!("{:=^80}", "");

    let mut last_change_count = 0;

    loop {
        let changes = monitor.get_recent_changes(50);
        let new_changes = changes.len() - last_change_count;

        if new_changes > 0 {
            // Clear screen and show recent changes
            print!("\x1B[2J\x1B[1;1H"); // ANSI clear screen
            println!("🔴 LIVE MONITORING - {} targets active", targets.len());
            println!(" {} total changes detected", changes.len());
            println!("{:=^80}", "");

            // Show last 10 changes
            let recent_changes: Vec<_> = changes.iter().rev().take(10).collect();

            for (i, change) in recent_changes.iter().enumerate() {
                let elapsed = change.timestamp.elapsed();
                println!(
                    " {} (0x{:x}) - {:.1}s ago",
                    change.name,
                    change.address,
                    elapsed.as_secs_f64()
                );

                // Show value change based on size
                match change.old_value.len() {
                    4 => {
                        let old_i32 =
                            i32::from_le_bytes(change.old_value[..4].try_into().unwrap_or([0; 4]));
                        let new_i32 =
                            i32::from_le_bytes(change.new_value[..4].try_into().unwrap_or([0; 4]));
                        println!("    {} → {} (i32)", old_i32, new_i32);
                    }
                    8 => {
                        let old_i64 =
                            i64::from_le_bytes(change.old_value[..8].try_into().unwrap_or([0; 8]));
                        let new_i64 =
                            i64::from_le_bytes(change.new_value[..8].try_into().unwrap_or([0; 8]));
                        println!("    {} → {} (i64)", old_i64, new_i64);
                    }
                    _ => {
                        let old_str = String::from_utf8_lossy(&change.old_value);
                        let new_str = String::from_utf8_lossy(&change.new_value);
                        println!(
                            "    '{}' → '{}'",
                            old_str.trim_end_matches('\0'),
                            new_str.trim_end_matches('\0')
                        );
                    }
                }

                if i < recent_changes.len() - 1 {
                    println!();
                }
            }

            println!("{:=^80}", "");
            println!(" Live monitoring... Press Ctrl+C to exit");

            last_change_count = changes.len();
        }

        // Check if user pressed Ctrl+C or monitor stopped
        if !monitor.is_running() {
            println!("\n  Monitor stopped, exiting live mode");
            break;
        }

        // Sleep briefly to avoid overwhelming the display
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn cmd_toggle_notifications(args: &[String]) {
    let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
    let monitor = match monitor_guard.as_ref() {
        Some(m) => {
            if !m.is_running() {
                println!(" Monitor is not running");
                println!(" Start monitoring first with 'monitor <PID>'");
                return;
            }
            m
        }
        None => {
            println!(" No monitor running");
            println!(" Start monitoring first with 'monitor <PID>' command");
            return;
        }
    };

    if args.len() < 2 {
        // Show current status
        let status = if monitor.is_live_mode() { "ON" } else { "OFF" };
        println!(" Live notifications are currently: {}", status);
        println!(" Usage: notifications <on|off>");
        return;
    }

    match args[1].to_lowercase().as_str() {
        "on" | "enable" | "true" | "1" => {
            monitor.enable_live_notifications();
        }
        "off" | "disable" | "false" | "0" => {
            monitor.disable_live_notifications();
            println!("⚫ Live notifications disabled");
            println!(" Use 'changes' command to view changes manually");
        }
        _ => {
            println!(" Invalid option: '{}'", args[1]);
            println!(" Use: notifications <on|off>");
        }
    }
}

fn cmd_set_interval(args: &[String]) {
    let monitor_guard = GLOBAL_MONITOR.lock().unwrap();
    let monitor = match monitor_guard.as_ref() {
        Some(m) => {
            if !m.is_running() {
                println!(" Monitor is not running");
                println!(" Start monitoring first with 'monitor <PID>'");
                return;
            }
            m
        }
        None => {
            println!(" No monitor running");
            return;
        }
    };

    if args.len() < 2 {
        println!(" Usage: interval <milliseconds>");
        println!(" Example: interval 50  # 50ms updates");
        println!(" Current interval: Check with 'status'");
        println!("  Note: Requires restarting monitor to take effect");
        return;
    }

    match args[1].parse::<u64>() {
        Ok(ms) if ms >= 10 && ms <= 5000 => {
            println!(" Interval set to {}ms", ms);
            println!("  Restart monitoring for changes to take effect");
            if ms < 100 {
                println!(" Fast intervals may cause race conditions");
                println!(" You might see partial/garbled string changes");
                println!(" Try 200ms+ if you see strange results");
            } else {
                println!(" Good interval for stable monitoring");
            }
        }
        Ok(ms) => {
            println!(" Interval must be between 10ms and 5000ms, got {}ms", ms);
        }
        Err(_) => {
            println!(" Invalid interval: '{}'", args[1]);
            println!(" Use a number in milliseconds");
        }
    }
}

fn cmd_debug_monitoring(args: &[String]) {
    if args.len() < 2 {
        println!(" Usage: debug <on|off>");
        println!(" Enables debug output for troubleshooting");
        return;
    }

    match args[1].to_lowercase().as_str() {
        "on" | "enable" | "true" | "1" => {
            println!(" Debug monitoring enabled");
            println!(" You'll see detailed timing and read information");
            println!(" Use 'debug off' to disable");
        }
        "off" | "disable" | "false" | "0" => {
            println!(" Debug monitoring disabled");
        }
        _ => {
            println!(" Invalid option: '{}'", args[1]);
            println!(" Use: debug <on|off>");
        }
    }
}

// Helper function to get process by PID
fn get_process_by_pid(pid: u32) -> Option<Process> {
    match enumerate_processes() {
        Ok(processes) => processes.into_iter().find(|p| p.pid == pid),
        Err(e) => {
            eprintln!(" Error enumerating processes: {}", e);
            None
        }
    }
}

fn print_hex_dump(data: &[u8], base_address: usize) {
    println!("{:-<60}", "");
    println!("Offset      00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f");
    println!("{:-<60}", "");

    for (i, chunk) in data.chunks(16).enumerate() {
        print!("{:08x}  ", base_address + i * 16);

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

        print!("  ");

        // Print ASCII representation
        for &byte in chunk {
            if byte.is_ascii_graphic() || byte == b' ' {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }

        println!();
    }
    println!("{:-<60}", "");
}

fn print_ascii_view(data: &[u8]) {
    println!("ASCII view:");
    let ascii_data: String = data
        .iter()
        .map(|&byte| {
            if byte.is_ascii_graphic() || byte == b' ' {
                byte as char
            } else {
                '.'
            }
        })
        .collect();

    println!("\"{}\"", ascii_data);
}
