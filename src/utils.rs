// utility funcs
// built by the goat (danielcos)

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub has_sudo: bool,
    pub can_read_proc: bool,
    pub ptrace_scope: Option<u32>,
}

pub fn system_checks() -> Result<SystemInfo, String> {
    let mut info = SystemInfo {
        os: detect_os(),
        has_sudo: false,
        can_read_proc: false,
        ptrace_scope: None,
    };

    info.has_sudo = sudo_check();

    info.can_read_proc = check_proc_access();

    if info.os == "Linux" {
        info.ptrace_scope = get_ptrace_scope();
    }

    Ok(info)
}

pub fn detect_os() -> String {
    if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else if cfg!(target_os = "macos") {
        "macOS".to_string()
    } else if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else {
        "Unknown".to_string()
    }
}

pub fn sudo_check() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }

    #[cfg(windows)]
    {
        std::env::var("USERNAME")
            .map(|user| user.to_lowercase().contains("admin"))
            .unwrap_or(false)
    }
}

pub fn check_proc_access() -> bool {
    std::fs::read_dir("/proc").is_ok()
}

pub fn get_ptrace_scope() -> Option<u32> {
    std::fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope")
        .ok()?
        .trim()
        .parse()
        .ok()
}

pub fn loading_with_checks() -> Result<SystemInfo, String> {
    println!();
    println!();
    println!("                            Initializing Memscan...");
    println!();

    let loading_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let mut char_index = 0;

    type CheckFn = fn(&mut SystemInfo) -> Result<(), String>;

    let checks: &[(&str, CheckFn)] = &[
        ("Detecting operating system", detect_os_check),
        ("Checking privileges", check_privileges),
        ("Verifying proc access", verify_proc_access),
        ("Analyzing security settings", check_security_settings),
        ("Preparing memory scanner", final_prep),
    ];

    let mut system_info = SystemInfo {
        os: "Unknown".to_string(),
        has_sudo: false,
        can_read_proc: false,
        ptrace_scope: None,
    };

    for (description, check_fn) in checks.iter() {
        print!(
            "\r{} {}...",
            loading_chars[char_index % loading_chars.len()],
            description
        );
        io::stdout().flush().unwrap();

        check_fn(&mut system_info)?;

        for _ in 0..8 {
            thread::sleep(Duration::from_millis(100));
            char_index += 1;
            print!(
                "\r{} {}...",
                loading_chars[char_index % loading_chars.len()],
                description
            );
            io::stdout().flush().unwrap();
        }

        println!("\r {} complete", description);
    }

    Ok(system_info)
}

fn detect_os_check(info: &mut SystemInfo) -> Result<(), String> {
    info.os = detect_os();
    thread::sleep(Duration::from_millis(200));
    Ok(())
}

fn check_privileges(info: &mut SystemInfo) -> Result<(), String> {
    info.has_sudo = sudo_check();
    if !info.has_sudo {
        return Err("    Insufficient privileges".to_string());
    }
    thread::sleep(Duration::from_millis(300));
    Ok(())
}

fn verify_proc_access(info: &mut SystemInfo) -> Result<(), String> {
    info.can_read_proc = check_proc_access();
    if !info.can_read_proc {
        return Err("    Cannot access /proc filesystem".to_string());
    }
    thread::sleep(Duration::from_millis(250));
    Ok(())
}

fn check_security_settings(info: &mut SystemInfo) -> Result<(), String> {
    if info.os == "Linux" {
        info.ptrace_scope = get_ptrace_scope();
        if let Some(scope) = info.ptrace_scope {
            if scope > 1 {
                return Err(format!("    ptrace_scope too restrictive: {}", scope));
            }
        }
    }
    thread::sleep(Duration::from_millis(200));
    Ok(())
}

fn final_prep(_info: &mut SystemInfo) -> Result<(), String> {
    thread::sleep(Duration::from_millis(300));
    Ok(())
}

pub fn simple_loading(message: &str, duration_ms: u64) {
    let loading_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let steps = duration_ms / 100;

    for i in 0..steps {
        print!(
            "\r{} {}...",
            loading_chars[i as usize % loading_chars.len()],
            message
        );
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(100));
    }

    println!("\r {} complete", message);
}

pub fn display_system_info(info: &SystemInfo) {
    println!("\nSYSTEM INFORMATION:");
    println!("+---------------------------------------+");
    println!("| Operating System: {:18} |", info.os);

    let sudo_status = if info.has_sudo {
        "OK  Available"
    } else {
        "!!  Missing"
    };
    println!("| Sudo Privileges:  {:18} |", sudo_status);

    let proc_status = if info.can_read_proc {
        "OK  Available"
    } else {
        "!!  Blocked"
    };
    println!("| Proc Access:      {:18} |", proc_status);

    if let Some(scope) = info.ptrace_scope {
        let scope_status = match scope {
            0 => "OK  Disabled",
            1 => "!!  Restricted",
            _ => "!!  Blocked",
        };
        println!("| Ptrace Scope:     {:18} |", scope_status);
    }

    println!("+---------------------------------------+");
}

pub fn suggest_fixes(info: &SystemInfo) {
    if !info.has_sudo {
        println!("\n    PRIVILEGE ISSUE DETECTED:");
        println!("      Run with sudo or ./run_memscan.sh");
    }

    if let Some(scope) = info.ptrace_scope {
        if scope > 1 {
            println!("\n    PTRACE RESTRICTION DETECTED:");
            println!("      Run with sudo or ./run_memscan.sh")
        }
    }

    if !info.can_read_proc {
        println!("\n    PROC ACCESS ISSUE:");
        println!("  Check if /proc is mounted and accessible");
    }
}
