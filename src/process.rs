// process management
// built by the goat (danielscos)

use crate::memory::{MemoryReader, MemoryRegion};
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
    memory_reader: Option<MemoryReader>,
}

impl ProcessHandle {
    pub fn read_memory(&self, address: usize, size: usize) -> Result<Vec<u8>, std::io::Error> {
        if let Some(ref reader) = self.memory_reader {
            reader.read_memory(address, size)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Process not attached",
            ))
        }
    }

    pub fn get_memory_regions(&self) -> Result<Vec<MemoryRegion>, std::io::Error> {
        crate::memory::get_memory_regions(self.pid)
    }

    pub fn get_scannable_regions(&self) -> Result<Vec<MemoryRegion>, std::io::Error> {
        let all_regions = self.get_memory_regions()?;

        // filter for readable regions larger than 1kb
        let scannable: Vec<MemoryRegion> = all_regions
            .into_iter()
            .filter(|r| r.readable && r.size > 1024)
            .collect();

        Ok(scannable)
    }
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
            let memory_reader = MemoryReader::new(self.pid);
            self.handle = Some(ProcessHandle {
                pid: self.pid,
                memory_reader: Some(memory_reader),
            });
            Ok(())
        } else {
            Err(format!("Process {} no longer exists", self.pid).into())
        }
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
    let _current_uid = unsafe { libc::getuid() };

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
                if process_name.is_empty() || process_name.starts_with('[') {
                    continue;
                }

                let status_path = format!("/proc/{}/status", pid);
                if std::path::Path::new(&status_path).exists() {
                    let maps_path = format!("/proc/{}/maps", pid);
                    if std::path::Path::new(&maps_path).exists() {
                        processes.push(Process::new(pid, process_name));
                    }
                }
            }
        }
    }

    processes.sort_by(|a, b| a.name.cmp(&b.name));
    processes.truncate(500); // Increased limit to show more processes

    Ok(processes)
}
