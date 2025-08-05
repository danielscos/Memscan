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

use nix::sys::uio::{RemoteIoVec, process_vm_readv, process_vm_writev};
use nix::unistd::Pid;
use std::io::{self, IoSlice, IoSliceMut};

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

    pub fn write_memory(&self, address: usize, data: &[u8]) -> Result<usize, io::Error> {
        let ioslice = IoSlice::new(data);
        let local_iov = std::slice::from_ref(&ioslice);

        let remote_iov = [RemoteIoVec {
            base: address,
            len: data.len(),
        }];

        match process_vm_writev(self.pid, local_iov, &remote_iov) {
            Ok(bytes_written) => Ok(bytes_written),
            Err(e) => Err(io::Error::from(e)),
        }
    }

    pub fn write_i32(&self, address: usize, value: i32) -> Result<usize, io::Error> {
        let bytes = value.to_le_bytes();
        self.write_memory(address, &bytes)
    }

    pub fn write_i64(&self, address: usize, value: i64) -> Result<usize, io::Error> {
        let bytes = value.to_le_bytes();
        self.write_memory(address, &bytes)
    }

    pub fn write_f32(&self, address: usize, value: f32) -> Result<usize, io::Error> {
        let bytes = value.to_le_bytes();
        self.write_memory(address, &bytes)
    }

    pub fn write_f64(&self, address: usize, value: f64) -> Result<usize, io::Error> {
        let bytes = value.to_le_bytes();
        self.write_memory(address, &bytes)
    }

    pub fn write_string(&self, address: usize, value: &str) -> Result<usize, io::Error> {
        let bytes = value.as_bytes();
        self.write_memory(address, bytes)
    }

    pub fn write_cstring(&self, address: usize, value: &str) -> Result<usize, io::Error> {
        let mut bytes = value.as_bytes().to_vec();
        bytes.push(0);
        self.write_memory(address, &bytes)
    }

    pub fn read_modify_write<F>(
        &self,
        address: usize,
        size: usize,
        modifier: F,
    ) -> Result<Vec<u8>, io::Error>
    where
        F: FnOnce(&mut [u8]),
    {
        let mut data = self.read_memory(address, size)?;

        modifier(&mut data);

        self.write_memory(address, &data)?;

        Ok(data)
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
