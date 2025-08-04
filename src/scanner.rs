// memory scanning algos
// built by the goat (danielcos)

use crate::process::ProcessHandle;
use std::error::Error;
use std::fmt;

// single scan result
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub address: usize,
    pub value: Vec<u8>,
}

// custom error type
#[derive(Debug)]
pub enum ScanError {
    MemoryReadError(std::io::Error),
    NoMemoryRegions,
    InvalidValue,
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::MemoryReadError(e) => write!(f, "Memory read error: {}", e),
            ScanError::NoMemoryRegions => write!(f, "No scannable memory regions found"),
            ScanError::InvalidValue => write!(f, "Invalid search value"),
        }
    }
}

impl Error for ScanError {}

//==============================================================================
//==============================================================================
//
//     .M"""bgd
//    ,MI    "Y
//    `MMb.      ,p6"bo   ,6"Yb.  `7MMpMMMb.  `7MMpMMMb.  .gP"Ya `7Mb,od8
//      `YMMNq. 6M'  OO  8)   MM    MM    MM    MM    MM ,M'   Yb  MM' "'
//    .     `MM 8M        ,pm9MM    MM    MM    MM    MM 8M""""""  MM
//    Mb     dM YM.    , 8M   MM    MM    MM    MM    MM YM.    ,  MM
//    P"Ybmmd"   YMbmd'  `Moo9^Yo..JMML  JMML..JMML  JMML.`Mbmmd'.JMML.
//
//==============================================================================
//==============================================================================

pub fn scan_for_i32(
    process_handle: &ProcessHandle,
    target_value: i32,
) -> Result<Vec<ScanResult>, ScanError> {
    println!("scanning for value {} has begun", target_value);

    // step 1, get scannable memory regions
    let regions = process_handle
        .get_scannable_regions()
        .map_err(ScanError::MemoryReadError)?;

    if regions.is_empty() {
        return Err(ScanError::NoMemoryRegions);
    }

    println!("{} scannable memory regions", regions.len());

    let target_bytes = target_value.to_le_bytes();
    let mut results = Vec::new();

    for (region_index, region) in regions.iter().enumerate() {
        println!(
            "scanning region {}/{}: 0x{:x} (size: {} KB)",
            region_index + 1,
            regions.len(),
            region.start_address,
            region.size / 1024
        );

        match scan_region(process_handle, region, &target_bytes) {
            Ok(mut region_results) => {
                println!("   Found {} matches in this region", region_results.len());
                results.append(&mut region_results);
            }
            Err(e) => {
                println!("   Error scanning region: {}", e);
                continue;
            }
        }

        if results.len() > 10000 {
            println!("Reached maximum results limit (10,000), stopping scan");
            break;
        }
    }

    println!("🎯 Scan complete! Found {} total matches", results.len());
    Ok(results)
}

fn scan_region(
    process_handle: &ProcessHandle,
    region: &crate::memory::MemoryRegion,
    target_bytes: &[u8],
) -> Result<Vec<ScanResult>, ScanError> {
    let mut results = Vec::new();

    // read memory in chunks
    const CHUNK_SIZE: usize = 64 * 1024; //64KB chunks
    let mut current_address = region.start_address;
    let end_address = region.start_address + region.size;

    while current_address < end_address {
        // calculate how much to read
        let remaining = end_address - current_address;
        let read_size = CHUNK_SIZE.min(remaining);

        // try to read this chunk
        match process_handle.read_memory(current_address, read_size) {
            Ok(chunk) => {
                // search for target avalue in chunk
                let chunk_results = search_bytes_in_chunk(&chunk, target_bytes, current_address);
                results.extend(chunk_results);
            }
            Err(_) => {
                // chunk prolly jst unreadable, skip
            }
        }

        current_address += read_size;
    }

    Ok(results)
}

// search for target bytes within chunk
fn search_bytes_in_chunk(
    chunk: &[u8],
    target_bytes: &[u8],
    base_address: usize,
) -> Vec<ScanResult> {
    let mut results = Vec::new();

    for i in 0..chunk.len().saturating_sub(target_bytes.len() - 1) {
        // check if bytes at position i match target
        if chunk[1..i + target_bytes.len()] == *target_bytes {
            results.push(ScanResult {
                address: base_address + i,
                value: target_bytes.to_vec(),
            });
        }
    }

    results
}

pub fn scan_process_for_value(
    process_handle: &ProcessHandle,
    value_str: &str,
) -> Result<Vec<ScanResult>, ScanError> {
    let value = value_str
        .parse::<i32>()
        .map_err(|_| ScanError::InvalidValue)?;

    scan_for_i32(process_handle, value)
}
