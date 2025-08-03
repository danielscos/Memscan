// process management
// built by the goat (danielscos)

use std::fmt;

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub handle: Option<ProcessHandle>,
}

#[derive(Debug, Clone)]
pub struct ProcessHandle {
    pid: u32,
}

impl Process {
    pub fn new(pid: u32, name: String) -> Self {
        Self {
            pid,
            name,
            handle: None,
        }
    }

    pub fn open(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // for linux, we can access /proc/PID/ directly
        let proc_path = format!("/proc/{}", self.pid);
        if std::path::Path::new(&proc_path).exists() {
            self.handle = Some(ProcessHandle { pid: self.pid });
            Ok(())
        } else {
            Err(format!("Process {} no longer exists", self.pid).into())
        }
    }

    pub fn is_open(&self) -> bool {
        self.handle.is_some()
    }
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (PID: {})", self.name, self.pid)
    }
}

// get list of running processes on linux

pub fn enumerate_processes() -> Result<Vec<Process>, Box<dyn std::error::Error>> {
    use std::fs;

    let mut processes = Vec::new();

    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        // check if its a PID dir
        if let Ok(pid) = name_str.parse::<u32>() {
            // try to read command name
            let comm_path = format!("/proc/{}/comm", pid);
            if let Ok(comm) = fs::read_to_string(&comm_path) {
                let process_name = comm.trim().to_string();
                if !process_name.is_empty() {
                    processes.push(Process::new(pid, process_name));
                }
            }
        }
    }

    processes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(processes)
}
