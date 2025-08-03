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

    // write memory to target process
    pub fn write_memory(&self, address: usize, data: &[u8]) -> Result<usize, io::Error> {
        let ioslice = IoSlice::new(data);
        let local_iov = std::slice::from_ref(&ioslice);

        // create remote IoVec (target process memory)
        let remote_iov = [RemoteIoVec {
            base: address,
            len: data.len(),
        }];

        // write to target process
        match process_vm_writev(self.pid, local_iov, &remote_iov) {
            Ok(bytes_written) => Ok(bytes_written),
            Err(e) => Err(io::Error::from(e)),
        }
    }

    // read a specific type from memory (i32, f64)
    pub fn read_value<T: Copy>(&self, address: usize) -> Result<T, io::Error> {
        let size = std::mem::size_of::<T>();
        let bytes = self.read_memory(address, size)?;

        if bytes.len() < size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes read",
            ));
        }

        let value = unsafe { std::ptr::read(bytes.as_ptr() as *const T) };

        Ok(value)
    }

    // write a specific type to memory
    pub fn write_value<T: Copy>(&self, address: usize, value: &T) -> Result<(), io::Error> {
        let bytes = unsafe {
            std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>())
        };

        self.write_memory(address, bytes)?;
        Ok(())
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

// filter memory regions by criteria
pub fn filter_readable_regions(regions: &[MemoryRegion]) -> Vec<&MemoryRegion> {
    regions.iter().filter(|r| r.readable).collect()
}

// filter memory regions by size
pub fn filter_regions_by_size(regions: &[MemoryRegion], min_size: usize) -> Vec<&MemoryRegion> {
    regions.iter().filter(|r| r.size >= min_size).collect()
}

// get writable regions only (for memory modification)
pub fn get_writable_regions(regions: &[MemoryRegion]) -> Vec<&MemoryRegion> {
    regions.iter().filter(|r| r.writable).collect()
}
