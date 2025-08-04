// Memory reading and writing >:3
//
//▓█████▄  ▄▄▄       ███▄    █  ██▓▓█████  ██▓      ██████  ▒█████   ▄████▄    ██████
//▒██▀ ██▌▒████▄     ██ ▀█   █ ▓██▒▓█   ▀ ▓██▒    ▒██    ▒ ▒██▒  ██▒▒██▀ ▀█  ▒██    ▒
//░██   █▌▒██  ▀█▄  ▓██  ▀█ ██▒▒██▒▒███   ▒██░    ░ ▓██▄   ▒██░  ██▒▒▓█    ▄ ░ ▓██▄
//░▓█▄   ▌░██▄▄▄▄██ ▓██▒  ▐▌██▒░██░▒▓█  ▄ ▒██░      ▒   ██▒▒██   ██░▒▓▓▄ ▄██▒  ▒   ██▒
//░▒████▓  ▓█   ▓██▒▒██░   ▓██░░██░░▒████▒░██████▒▒██████▒▒░ ████▓▒░▒ ▓███▀ ░▒██████▒▒
// ▒▒▓  ▒  ▒▒   ▓▒█░░ ▒░   ▒ ▒ ░▓  ░░ ▒░ ░░ ▒░▓  ░▒ ▒▓▒ ▒ ░░ ▒░▒░▒░ ░ ░▒ ▒  ░▒ ▒▓▒ ▒ ░
// ░ ▒  ▒   ▒   ▒▒ ░░ ░░   ░ ▒░ ▒ ░ ░ ░  ░░ ░ ▒  ░░ ░▒  ░ ░  ░ ▒ ▒░   ░  ▒   ░ ░▒  ░ ░
// ░ ░  ░   ░   ▒      ░   ░ ░  ▒ ░   ░     ░ ░   ░  ░  ░  ░ ░ ░ ▒  ░        ░  ░  ░
//   ░          ░  ░         ░  ░     ░  ░    ░  ░      ░      ░ ░  ░ ░            ░
// ░                                                                ░
// built by the goat (danielcos)

use nix::sys::uio::{RemoteIoVec, process_vm_readv};
use nix::unistd::Pid;
use std::io::{self, IoSliceMut};

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start_address: usize,
    pub size: usize,
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

#[derive(Debug, Clone)]
pub struct MemoryReader {
    pid: Pid,
}

impl MemoryReader {
    pub fn new(pid: u32) -> Self {
        Self {
            pid: Pid::from_raw(pid as i32),
        }
    }

    // read memory from target process
    pub fn read_memory(&self, address: usize, size: usize) -> Result<Vec<u8>, io::Error> {
        let mut buffer = vec![0u8; size];

        let mut ioslice = IoSliceMut::new(&mut buffer);
        let local_iov = std::slice::from_mut(&mut ioslice);

        let remote_iov = [RemoteIoVec {
            base: address,
            len: size,
        }];

        // read from target proess
        match process_vm_readv(self.pid, local_iov, &remote_iov) {
            Ok(bytes_read) => {
                buffer.truncate(bytes_read);
                Ok(buffer)
            }
            Err(e) => Err(io::Error::from(e)),
        }
    }
}

pub fn get_memory_regions(pid: u32) -> Result<Vec<MemoryRegion>, io::Error> {
    let maps_path = format!("/proc/{}/maps", pid);
    let maps_content = std::fs::read_to_string(maps_path)?;

    let mut regions = Vec::new();

    for line in maps_content.lines() {
        if let Some(region) = parse_maps_line(line) {
            regions.push(region);
        }
    }

    Ok(regions)
}

fn parse_maps_line(line: &str) -> Option<MemoryRegion> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let addr_parts: Vec<&str> = parts[0].split('-').collect();
    if addr_parts.len() != 2 {
        return None;
    }

    let start = usize::from_str_radix(addr_parts[0], 16).ok()?;
    let end = usize::from_str_radix(addr_parts[1], 16).ok()?;

    // parse permissions (e.g., "rwxp")
    let perms = parts[1];
    let readable = perms.chars().nth(0) == Some('r');
    let writable = perms.chars().nth(1) == Some('w');
    let executable = perms.chars().nth(2) == Some('x');

    Some(MemoryRegion {
        start_address: start,
        size: end - start,
        readable,
        writable,
        executable,
    })
}
