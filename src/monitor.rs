// real time memory monitoring with live notifications
// built by:
// =================================================================================================
// =================================================================================================

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

//=================================================================================================
// =================================================================================================

use crate::process::ProcessHandle;
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct MonitorTarget {
    pub address: usize,
    pub data_type: DataType,
    pub name: String,
    pub last_value: Option<Vec<u8>>,
    pub change_count: u64,
    pub last_changed: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum DataType {
    I32,
    I64,
    F32,
    F64,
    String(usize),
}

#[derive(Debug, Clone)]
pub struct MonitorChange {
    pub address: usize,
    pub name: String,
    pub old_value: Vec<u8>,
    pub new_value: Vec<u8>,
    pub timestamp: Instant,
    pub data_type: DataType,
}

pub struct MemoryMonitor {
    targets: Arc<Mutex<HashMap<usize, MonitorTarget>>>,
    changes: Arc<Mutex<Vec<MonitorChange>>>,
    running: Arc<AtomicBool>,
    update_interval: Duration,
    max_changes_history: usize,
    live_mode: Arc<AtomicBool>,
}

impl MemoryMonitor {
    pub fn new(update_interval_ms: u64) -> Self {
        Self {
            targets: Arc::new(Mutex::new(HashMap::new())),
            changes: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
            update_interval: Duration::from_millis(update_interval_ms),
            max_changes_history: 1000,
            live_mode: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn add_target(&self, address: usize, data_type: DataType, name: String) {
        let target = MonitorTarget {
            address,
            data_type,
            name: name.clone(),
            last_value: None,
            change_count: 0,
            last_changed: None,
        };

        let mut targets = self.targets.lock().unwrap();
        targets.insert(address, target);
        println!("  Added monitoring target: {} at 0x{:x}", name, address);
    }

    pub fn remove_target(&self, address: usize) {
        let mut targets = self.targets.lock().unwrap();
        if let Some(target) = targets.remove(&address) {
            println!(
                "  Removed monitoring target: {} at 0x{:x}",
                target.name, address
            );
        }
    }

    pub fn start_monitoring(&self, process_handle: Arc<ProcessHandle>) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Err("Monitor is already running".to_string());
        }

        self.running.store(true, Ordering::Relaxed);
        println!(" Starting real time monitoring");

        let targets_clone = Arc::clone(&self.targets);
        let changes_clone = Arc::clone(&self.changes);
        let running_clone = Arc::clone(&self.running);
        let live_mode_clone = Arc::clone(&self.live_mode);
        let update_interval = self.update_interval;
        let max_changes = self.max_changes_history;

        // Start main monitoring thread
        thread::spawn(move || {
            Self::monitor_loop(
                process_handle,
                targets_clone,
                changes_clone,
                running_clone,
                live_mode_clone,
                update_interval,
                max_changes,
            )
        });

        Ok(())
    }

    pub fn stop_monitoring(&self) {
        self.running.store(false, Ordering::Relaxed);
        self.live_mode.store(false, Ordering::Relaxed);
        println!("⏹️  Stopping memory monitoring");
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn enable_live_notifications(&self) {
        self.live_mode.store(true, Ordering::Relaxed);
        println!("🔴 Live notifications enabled - changes will appear automatically");
    }

    pub fn disable_live_notifications(&self) {
        self.live_mode.store(false, Ordering::Relaxed);
        println!("⚫ Live notifications disabled");
    }

    pub fn is_live_mode(&self) -> bool {
        self.live_mode.load(Ordering::Relaxed)
    }

    pub fn get_recent_changes(&self, count: usize) -> Vec<MonitorChange> {
        let changes = self.changes.lock().unwrap();
        let start_idx = changes.len().saturating_sub(count);
        changes[start_idx..].to_vec()
    }

    pub fn get_targets_status(&self) -> Vec<(usize, String, u64, Option<Instant>)> {
        let targets = self.targets.lock().unwrap();
        targets
            .values()
            .map(|t| (t.address, t.name.clone(), t.change_count, t.last_changed))
            .collect()
    }

    fn display_change_notification(change: &MonitorChange, live_mode: &Arc<AtomicBool>) {
        if !live_mode.load(Ordering::Relaxed) {
            return;
        }

        let elapsed = change.timestamp.elapsed();
        println!(
            "\n🔄 CHANGE DETECTED: {} (0x{:x}) - {:.1}s ago",
            change.name,
            change.address,
            elapsed.as_secs_f64()
        );

        // Display based on the actual data type stored with the target
        match change.data_type {
            DataType::I32 => {
                let old_i32 =
                    i32::from_le_bytes(change.old_value[..4].try_into().unwrap_or([0; 4]));
                let new_i32 =
                    i32::from_le_bytes(change.new_value[..4].try_into().unwrap_or([0; 4]));
                println!("   📊 {} → {} (i32)", old_i32, new_i32);
            }
            DataType::I64 => {
                let old_i64 =
                    i64::from_le_bytes(change.old_value[..8].try_into().unwrap_or([0; 8]));
                let new_i64 =
                    i64::from_le_bytes(change.new_value[..8].try_into().unwrap_or([0; 8]));
                println!("   📊 {} → {} (i64)", old_i64, new_i64);
            }
            DataType::F32 => {
                let old_f32 =
                    f32::from_le_bytes(change.old_value[..4].try_into().unwrap_or([0; 4]));
                let new_f32 =
                    f32::from_le_bytes(change.new_value[..4].try_into().unwrap_or([0; 4]));
                println!("   📊 {:.3} → {:.3} (f32)", old_f32, new_f32);
            }
            DataType::F64 => {
                let old_f64 =
                    f64::from_le_bytes(change.old_value[..8].try_into().unwrap_or([0; 8]));
                let new_f64 =
                    f64::from_le_bytes(change.new_value[..8].try_into().unwrap_or([0; 8]));
                println!("   📊 {:.6} → {:.6} (f64)", old_f64, new_f64);
            }
            DataType::String(_) => {
                // Display as string
                Self::display_string_change(change);
            }
        }
        print!("memscan> "); // Re-display prompt
        use std::io::Write;
        std::io::stdout().flush().unwrap();
    }

    fn display_string_change(change: &MonitorChange) {
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
            println!("   📊 '{}' → '{}'", old_clean, new_clean);

            // Show byte lengths if different
            if old_str_end != new_str_end {
                println!("   📏 Length: {} → {} bytes", old_str_end, new_str_end);
            }
        } else {
            // Fallback to hex if no printable characters
            println!("   📊 {:02x?} → {:02x?}", old_bytes, new_bytes);
        }
    }

    fn monitor_loop(
        process_handle: Arc<ProcessHandle>,
        targets: Arc<Mutex<HashMap<usize, MonitorTarget>>>,
        changes: Arc<Mutex<Vec<MonitorChange>>>,
        running: Arc<AtomicBool>,
        live_mode: Arc<AtomicBool>,
        update_interval: Duration,
        max_changes: usize,
    ) {
        while running.load(Ordering::Relaxed) {
            let start_time = Instant::now();

            // read all monitored addresses
            {
                let mut targets_guard = targets.lock().unwrap();
                let addresses: Vec<usize> = targets_guard.keys().cloned().collect();

                for address in addresses {
                    if let Some(target) = targets_guard.get_mut(&address) {
                        // Read with stability checking to avoid race conditions
                        let stable_value = Self::read_with_stability_check(
                            &process_handle,
                            address,
                            &target.data_type,
                        );

                        if let Some(current_value) = stable_value {
                            if let Some(ref last_value) = target.last_value {
                                // For strings, compare only meaningful content
                                let values_different = if let DataType::String(_) = target.data_type
                                {
                                    Self::string_content_different(last_value, &current_value)
                                } else {
                                    current_value != *last_value
                                };

                                if values_different {
                                    let change = MonitorChange {
                                        address,
                                        name: target.name.clone(),
                                        old_value: last_value.clone(),
                                        new_value: current_value.clone(),
                                        timestamp: Instant::now(),
                                        data_type: target.data_type.clone(),
                                    };

                                    target.change_count += 1;
                                    target.last_changed = Some(change.timestamp);

                                    // Display change notification if live mode is enabled
                                    Self::display_change_notification(&change, &live_mode);

                                    {
                                        let mut changes_guard = changes.lock().unwrap();
                                        changes_guard.push(change.clone());

                                        if changes_guard.len() > max_changes {
                                            changes_guard.remove(0);
                                        }
                                    }
                                }
                            }
                            target.last_value = Some(current_value);
                        }
                    }
                }
            }

            let elapsed = start_time.elapsed();
            if elapsed < update_interval {
                thread::sleep(update_interval - elapsed);
            }
        }

        println!("  Memory monitoring stopped");
    }

    fn get_data_type_size(data_type: &DataType) -> usize {
        match data_type {
            DataType::I32 => 4,
            DataType::I64 => 8,
            DataType::F32 => 4,
            DataType::F64 => 8,
            DataType::String(_) => 256, // Max read size for strings
        }
    }

    fn read_adaptive_string(
        process_handle: &Arc<ProcessHandle>,
        address: usize,
        max_size: usize,
    ) -> (usize, Vec<u8>) {
        // Read up to max_size bytes
        if let Ok(data) = process_handle.read_memory(address, max_size) {
            // Find null terminator
            if let Some(null_pos) = data.iter().position(|&b| b == 0) {
                // Return string content + null terminator
                let actual_size = null_pos + 1;
                (actual_size, data[..actual_size].to_vec())
            } else {
                // No null terminator found, return all data
                (max_size, data)
            }
        } else {
            // Failed to read, return empty
            (0, vec![])
        }
    }

    fn read_with_stability_check(
        process_handle: &Arc<ProcessHandle>,
        address: usize,
        data_type: &DataType,
    ) -> Option<Vec<u8>> {
        // Read the memory multiple times to ensure stability
        let (first_read, second_read) = match data_type {
            DataType::String(_) => {
                let (_, first) = Self::read_adaptive_string(process_handle, address, 256);
                // Small delay to let any writes complete
                std::thread::sleep(std::time::Duration::from_millis(1));
                let (_, second) = Self::read_adaptive_string(process_handle, address, 256);
                (first, second)
            }
            _ => {
                let size = Self::get_data_type_size(data_type);
                let first = process_handle.read_memory(address, size).ok()?;
                // Small delay to let any writes complete
                std::thread::sleep(std::time::Duration::from_millis(1));
                let second = process_handle.read_memory(address, size).ok()?;
                (first, second)
            }
        };

        // For strings, compare meaningful content; for others, exact match
        let values_stable = match data_type {
            DataType::String(_) => !Self::string_content_different(&first_read, &second_read),
            _ => first_read == second_read,
        };

        if values_stable {
            Some(second_read)
        } else {
            // Value is still changing, skip this update
            None
        }
    }

    fn string_content_different(old_value: &[u8], new_value: &[u8]) -> bool {
        // Find the actual string content (before null terminators)
        let old_end = old_value
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(old_value.len());
        let new_end = new_value
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(new_value.len());

        // Compare only the string content, not the full buffer
        old_value[..old_end] != new_value[..new_end]
    }
}
