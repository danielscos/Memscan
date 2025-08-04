// Memscan GUI - FLTK-based graphical memory scanner
// High-performance, cross-platform memory scanner built by the goat (Danielscos)

use fltk::{
    app, browser::Browser, button::Button, dialog, frame::Frame, input::Input, prelude::*,
    window::Window,
};
use std::{cell::RefCell, rc::Rc};

use memscan::{
    memory_optimization::{get_allocated_bytes, track_allocation},
    process::{Process, enumerate_processes},
    scanner::{
        ScanResult, scan_process_for_f32, scan_process_for_f64, scan_process_for_i32,
        scan_process_for_i64, scan_process_for_string,
    },
};

// app state
struct AppState {
    selected_process: Option<Process>,
    scan_results: Vec<ScanResult>,
    is_attached: bool,
    last_scan_value: String,
    all_processes: Vec<Process>,
    filtered_processes: Vec<Process>,
}

impl AppState {
    fn new() -> Self {
        track_allocation(std::mem::size_of::<Self>());
        Self {
            selected_process: None,
            scan_results: Vec::new(),
            is_attached: false,
            last_scan_value: String::new(),

            all_processes: Vec::new(),
            filtered_processes: Vec::new(),
        }
    }

    fn set_selected_process(&mut self, process: Process) {
        self.selected_process = Some(process);
        self.is_attached = false; // reset attachment status
        self.clear_scan_results(); // clear old results when switching processes
    }

    fn attach_to_process(&mut self) -> Result<(), String> {
        if let Some(ref mut process) = self.selected_process {
            // Check if we have sufficient privileges
            if !check_memory_access_privileges() {
                return self.handle_privilege_request();
            }

            // Proceed with attachment
            match process.open() {
                Ok(()) => {
                    self.is_attached = true;
                    Ok(())
                }
                Err(e) => Err(format!("Failed to attach to process: {}", e)),
            }
        } else {
            Err("No process selected".to_string())
        }
    }

    fn handle_privilege_request(&self) -> Result<(), String> {
        let ptrace_scope = get_ptrace_scope();

        let message = format!(
            "🔒 Privileges Required\n\n\
            Memscan needs elevated privileges to scan process memory.\n\
            Your system has ptrace_scope={}, which restricts memory access.\n\n\
            To run memscan with privileges:\n\
            sudo -E env DISPLAY=$DISPLAY XAUTHORITY=$XAUTHORITY ./target/release/memscan\n\n\
            Or use the helper script: ./run_with_sudo.sh",
            ptrace_scope
        );

        dialog::message_default(&message);
        Err("Please restart with elevated privileges".to_string())
    }

    fn clear_scan_results(&mut self) {
        self.scan_results.clear();
    }

    fn set_processes(&mut self, processes: Vec<Process>) {
        self.all_processes = processes;
        self.filtered_processes = self.all_processes.clone();
    }

    fn filter_processes(&mut self, filter: &str) {
        if filter.is_empty() {
            self.filtered_processes = self.all_processes.clone();
        } else {
            self.filtered_processes = self
                .all_processes
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&filter.to_lowercase()))
                .cloned()
                .collect();
        }
    }

    fn perform_scan_by_type(&mut self, value: &str, scan_type: usize) -> Result<usize, String> {
        self.clear_scan_results();
        self.last_scan_value = value.to_string();

        if let Some(ref process) = self.selected_process {
            if let Some(ref handle) = process.handle {
                let result = match scan_type {
                    0 => scan_process_for_i32(handle, value),
                    1 => scan_process_for_i64(handle, value),
                    2 => scan_process_for_f32(handle, value),
                    3 => scan_process_for_f64(handle, value),
                    4 => scan_process_for_string(handle, value),
                    _ => scan_process_for_i32(handle, value),
                };

                match result {
                    Ok(results) => {
                        let count = results.len();
                        self.scan_results = results;
                        Ok(count)
                    }
                    Err(e) => Err(format!("Scan failed: {}", e)),
                }
            } else {
                Err("Process not attached".to_string())
            }
        } else {
            Err("No process selected".to_string())
        }
    }

    fn get_memory_usage(&self) -> String {
        let bytes = get_allocated_bytes();
        let mb = bytes as f64 / (1024.0 * 1024.0);
        format!("Memory: {:.1} MB ({} bytes)", mb, bytes)
    }
}

// Helper functions for privilege management
fn check_memory_access_privileges() -> bool {
    let is_root = unsafe { libc::geteuid() } == 0;
    let ptrace_scope = get_ptrace_scope();
    is_root || ptrace_scope == 0
}

fn get_ptrace_scope() -> i32 {
    std::fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope")
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(-1)
}

fn main() {
    println!("Starting Memscan v0.1.0");

    // Check privilege status
    let is_root = unsafe { libc::geteuid() } == 0;
    let ptrace_scope = get_ptrace_scope();

    if is_root {
        println!("✅ Running with elevated privileges");
    } else if ptrace_scope > 0 {
        println!(
            "ℹ️  ptrace_scope={}, may need elevation for memory scanning",
            ptrace_scope
        );
    }

    // init fltk
    let app = app::App::default();

    // create window - make it taller for better layout
    let mut wind = Window::new(100, 100, 900, 700, "Memscan - Memory Scanner v0.1.0");
    wind.set_color(fltk::enums::Color::White);

    // create ui elements
    let mut title_frame = Frame::new(
        10,
        10,
        830,
        30,
        "Memscan - Memory Scanner (with optimizations)",
    );
    title_frame.set_label_size(16);
    title_frame.set_label_color(fltk::enums::Color::DarkBlue);

    let mut memory_frame = Frame::new(10, 45, 780, 25, "");
    memory_frame.set_label_size(11);
    memory_frame.set_label_color(fltk::enums::Color::DarkGreen);

    // process selection
    let mut process_section = Frame::new(10, 80, 300, 25, "Select target process:");
    process_section.set_label_size(14);
    process_section.set_label_color(fltk::enums::Color::Black);
    process_section.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    // process search functionality
    let mut search_label = Frame::new(10, 105, 150, 20, "Search processes:");
    search_label.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);
    search_label.set_label_size(12);

    let mut search_input = Input::new(10, 125, 300, 25, "");
    let mut search_hint = Frame::new(
        10,
        150,
        400,
        15,
        "Type to filter (e.g., 'test_target', 'chrome', 'firefox')",
    );
    search_hint.set_label_size(10);
    search_hint.set_label_color(fltk::enums::Color::Dark3);
    search_hint.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    let mut clear_search_btn = Button::new(320, 125, 80, 25, "Clear");

    let mut process_list = Browser::new(10, 170, 600, 185, "");
    process_list.set_type(fltk::browser::BrowserType::Hold);

    let mut list_processes_btn = Button::new(620, 170, 120, 35, "List Processes");
    let mut attach_btn = Button::new(620, 215, 120, 35, "Attach");
    attach_btn.deactivate();

    let mut scan_section = Frame::new(10, 365, 300, 25, "Memory scanning:");
    scan_section.set_label_size(14);
    scan_section.set_label_color(fltk::enums::Color::Black);
    scan_section.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    let mut value_label = Frame::new(10, 395, 150, 20, "Value to find:");
    value_label.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    let mut value_input = Input::new(10, 420, 150, 30, "");
    value_input.set_value("0");

    let mut scan_btn = Button::new(170, 420, 100, 30, "Start Scan");
    scan_btn.deactivate(); // enable after attachment

    let scan_type_btn = Button::new(280, 395, 150, 25, "Type: i32 (4 bytes)");
    let scan_types = vec![
        "i32 (4 bytes)",
        "i64 (8 bytes)",
        "f32 (4 bytes)",
        "f64 (8 bytes)",
        "String (ASCII)",
    ];
    let scan_type_index = Rc::new(RefCell::new(0));

    {
        let scan_type_index = scan_type_index.clone();
        let scan_types = scan_types.clone();
        let mut scan_type_btn = scan_type_btn.clone();

        scan_type_btn.set_callback(move |btn| {
            let mut index = scan_type_index.borrow_mut();
            *index = (*index + 1) % scan_types.len();
            btn.set_label(&format!("Type: {}", scan_types[*index]));
            println!("Scan type changed to: {}", scan_types[*index]);
            btn.redraw();
        });
    }

    // Add a scan status frame
    let mut scan_status_frame = Frame::new(10, 455, 600, 20, "Ready to scan");
    scan_status_frame.set_label_size(11);
    scan_status_frame.set_label_color(fltk::enums::Color::DarkBlue);
    scan_status_frame.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    // results section with better layout
    let mut results_section = Frame::new(10, 480, 300, 25, "Scan Results:");
    results_section.set_label_size(14);
    results_section.set_label_color(fltk::enums::Color::Black);
    results_section.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    // Make results list bigger with column headers
    let mut results_list = Browser::new(10, 510, 600, 120, "");
    results_list.add("Address          | Value");
    results_list.add("-----------------|-------");

    // Clear results button
    let mut clear_results_btn = Button::new(620, 510, 100, 30, "Clear Results");

    // status and controls (moved down to accommodate larger results area)
    let mut status_frame = Frame::new(10, 640, 600, 25, "ready - click 'list processes' to begin");
    status_frame.set_label_size(10);
    status_frame.set_label_color(fltk::enums::Color::DarkRed);
    status_frame.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    let mut quit_btn = Button::new(620, 640, 80, 25, "Quit");

    let state = Rc::new(RefCell::new(AppState::new()));

    // Memory usage display update
    {
        let state = state.clone();
        let mut memory_frame = memory_frame.clone();
        memory_frame.set_label(&state.borrow().get_memory_usage());
    }

    // list processes button
    {
        let mut process_list = process_list.clone();
        let mut status_frame = status_frame.clone();
        let state = state.clone();
        let mut memory_frame = memory_frame.clone();

        list_processes_btn.set_callback(move |_| {
            println!("Listing processes..."); //debug

            match enumerate_processes() {
                Ok(processes) => {
                    process_list.clear();

                    // Store processes in state for filtering
                    state.borrow_mut().set_processes(processes);

                    // Display initial filtered list (all processes)
                    for process in &state.borrow().filtered_processes {
                        process_list.add(&format!("{}", process));
                    }

                    status_frame.set_label(&format!(
                        "Found {} processes - select one to attach to",
                        state.borrow().all_processes.len()
                    ));

                    // update memory usage
                    memory_frame.set_label(&state.borrow().get_memory_usage());
                }
                Err(e) => {
                    status_frame.set_label(&format!("Error listing processes: {}", e));
                    println!("Error: {}", e);
                }
            }

            status_frame.redraw();
            memory_frame.redraw();
        });
    }

    // Search input callback for real-time filtering
    {
        let mut process_list = process_list.clone();
        let mut status_frame = status_frame.clone();
        let state = state.clone();

        search_input.set_callback(move |input| {
            let filter_text = input.value();

            // Only filter if we have processes loaded
            if state.borrow().all_processes.is_empty() {
                return;
            }

            // Filter processes based on search text
            state.borrow_mut().filter_processes(&filter_text);

            // Update process list display
            process_list.clear();
            for process in &state.borrow().filtered_processes {
                process_list.add(&format!("{}", process));
            }

            // Update status with filtered count
            let total = state.borrow().all_processes.len();
            let filtered = state.borrow().filtered_processes.len();

            if filter_text.is_empty() {
                status_frame.set_label(&format!(
                    "Showing all {} processes - select one to attach to",
                    total
                ));
            } else {
                status_frame.set_label(&format!(
                    "🔍 {} of {} processes match '{}' - {} hidden",
                    filtered,
                    total,
                    filter_text,
                    total - filtered
                ));
            }

            process_list.redraw();
            status_frame.redraw();
        });
    }

    // Clear search button callback
    {
        let mut search_input = search_input.clone();
        let mut process_list = process_list.clone();
        let mut status_frame = status_frame.clone();
        let state = state.clone();

        clear_search_btn.set_callback(move |_| {
            search_input.set_value("");

            // Only clear if we have processes loaded
            if state.borrow().all_processes.is_empty() {
                return;
            }

            // Reset filter to show all processes
            state.borrow_mut().filter_processes("");

            // Update process list display
            process_list.clear();
            for process in &state.borrow().filtered_processes {
                process_list.add(&format!("{}", process));
            }

            // Update status
            let total = state.borrow().all_processes.len();
            status_frame.set_label(&format!(
                "✅ Showing all {} processes - search cleared",
                total
            ));

            process_list.redraw();
            status_frame.redraw();
        });
    }

    {
        let state = state.clone();
        let mut attach_btn = attach_btn.clone();
        let mut status_frame = status_frame.clone();

        process_list.set_callback(move |browser| {
            let selection = browser.value();
            if selection > 0 {
                if let Some(text) = browser.text(selection) {
                    println!("Selected: {}", text);

                    if let Some(pid_start) = text.rfind("PID: ") {
                        if let Some(pid_end) = text.rfind(")") {
                            let pid_str = &text[pid_start + 5..pid_end];
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                let name_end = text.rfind(" (PID:").unwrap_or(text.len());
                                let name = text[..name_end].to_string();

                                let process = Process::new(pid, name);
                                state.borrow_mut().set_selected_process(process);

                                attach_btn.activate();
                                status_frame
                                    .set_label(&format!("Selected: {} - click 'attach'", text));
                                status_frame.redraw();
                            }
                        }
                    }
                }
            }
        });
    }

    // attach button
    {
        let state = state.clone();
        let mut status_frame = status_frame.clone();
        let mut scan_btn = scan_btn.clone();

        attach_btn.set_callback(move |_| {
            println!("Attach button clicked"); //debug

            let has_process = state.borrow().selected_process.is_some();

            if has_process {
                // Show initial status
                status_frame.set_label("🔒 Checking permissions...");
                status_frame.redraw();

                let result = {
                    let mut state_mut = state.borrow_mut();
                    state_mut.attach_to_process()
                };

                match result {
                    Ok(_) => {
                        if let Some(ref process) = state.borrow().selected_process {
                            status_frame.set_label(&format!(
                                "✅ Attached to {} - ready to scan memory",
                                process
                            ));
                            scan_btn.activate(); // enable scanning
                            println!("Successfully attached to {}", process); //debug
                        }
                    }
                    Err(e) => {
                        if e.contains("privileges") || e.contains("sudo") {
                            status_frame.set_label("🔐 Elevated privileges required - see dialog");
                        } else {
                            status_frame.set_label(&format!("❌ {}", e));
                        }
                        println!("attach failed: {}", e);
                    }
                }
            } else {
                status_frame.set_label("No process selected");
                println!("no process selected")
            }

            status_frame.redraw();
        });
    }

    {
        let state = state.clone();
        let mut status_frame = status_frame.clone();
        let mut scan_status_frame = scan_status_frame.clone();
        let mut results_list = results_list.clone();
        let value_input = value_input.clone();
        let scan_type_index = scan_type_index.clone();

        scan_btn.set_callback(move |btn| {
            let search_value = value_input.value();
            let selected_type = *scan_type_index.borrow();

            // Validation
            if search_value.is_empty() {
                status_frame.set_label("❌ Please enter a value to search for");
                status_frame.redraw();
                return;
            }

            // Check if we're attached
            if !state.borrow().is_attached {
                status_frame.set_label("❌ Not attached to any process");
                status_frame.redraw();
                return;
            }

            // Smart suggestions for common problematic values
            if search_value == "0" {
                status_frame.set_label("⚠️ Searching for '0' may find too many results. Try a unique value instead.");
                status_frame.redraw();
                scan_status_frame.set_label("💡 Tip: Use unique values like 12345, 999, or 42 for better results");
                scan_status_frame.redraw();
                app::awake();
                std::thread::sleep(std::time::Duration::from_millis(2000));
            }

            // All data types are now supported!

            // Disable scan button during scanning
            btn.deactivate();
            status_frame.set_label("🔄 Scanning in progress...");
            status_frame.redraw();
            scan_status_frame.set_label("🔍 Searching through process memory...");
            scan_status_frame.redraw();
            app::awake(); // Update GUI immediately

            // Perform the scan using unified method
            let scan_result = state
                .borrow_mut()
                .perform_scan_by_type(&search_value, selected_type);
            match scan_result {
                Ok(count) => {
                    // Clear previous results
                    results_list.clear();
                    results_list.add("Address          | Value");
                    results_list.add("-----------------|-------");

                    // Get results in a separate scope to avoid borrowing conflicts
                    let (results, display_count) = {
                        let state_ref = state.borrow();
                        let results = &state_ref.scan_results;
                        let display_count = results.len().min(100);
                        (results.clone(), display_count)
                    };

                    for result in results.iter().take(display_count) {
                        let type_display = match selected_type {
                            0 => format!("{} (i32)", search_value),
                            1 => format!("{} (i64)", search_value),
                            2 => format!("{} (f32)", search_value),
                            3 => format!("{} (f64)", search_value),
                            4 => format!("'{}' (string)", search_value),
                            _ => search_value.to_string(),
                        };
                        results_list.add(&format!("0x{:016X} | {}", result.address, type_display));
                    }

                    if results.len() > display_count {
                        results_list.add(&format!(
                            "... and {} more results (showing first {})",
                            results.len() - display_count,
                            display_count
                        ));
                    }

                    // Update status with intelligent feedback
                    let type_name = match selected_type {
                        0 => "i32",
                        1 => "i64",
                        2 => "f32",
                        3 => "f64",
                        4 => "string",
                        _ => "unknown",
                    };

                    if count >= 10000 {
                        status_frame.set_label(&format!(
                            "⚠️ Found {}+ {} matches (limit reached)",
                            count, type_name
                        ));
                        scan_status_frame.set_label(&format!(
                            "Too many results! Try a more unique value. Common values like 0, 1, -1 appear frequently."
                        ));
                    } else if count > 1000 {
                        status_frame.set_label(&format!(
                            "⚠️ Found {} {} matches (many results)",
                            count, type_name
                        ));
                        scan_status_frame.set_label(&format!(
                            "Many results found. Consider using a more specific value to narrow down."
                        ));
                    } else if count > 100 {
                        status_frame.set_label(&format!(
                            "✅ Found {} {} matches (good)",
                            count, type_name
                        ));
                        scan_status_frame.set_label(&format!(
                            "Found {} addresses containing {} value '{}'",
                            count, type_name, search_value
                        ));
                    } else if count > 0 {
                        status_frame.set_label(&format!(
                            "✅ Found {} {} matches (excellent)",
                            count, type_name
                        ));
                        scan_status_frame.set_label(&format!(
                            "Perfect! Found {} addresses containing {} value '{}'",
                            count, type_name, search_value
                        ));
                    } else {
                        status_frame.set_label(&format!(
                            "❌ No {} matches found",
                            type_name
                        ));
                        scan_status_frame.set_label(&format!(
                            "Value '{}' not found. Try a different value or data type.",
                            search_value
                        ));
                    }

                    println!("✅ Scan completed successfully: {} results", count);
                }
                Err(e) => {
                    status_frame.set_label(&format!("❌ Scan failed: {}", e));
                    scan_status_frame.set_label("Scan failed - check status for details");
                    println!("❌ Scan error: {}", e);
                }
            }

            // Re-enable scan button
            btn.activate();
            status_frame.redraw();
            scan_status_frame.redraw();
            results_list.redraw();
        });
    }

    // Clear results button callback
    {
        let state = state.clone();
        let mut results_list = results_list.clone();
        let mut scan_status_frame = scan_status_frame.clone();

        clear_results_btn.set_callback(move |_| {
            state.borrow_mut().clear_scan_results();
            results_list.clear();
            results_list.add("Address          | Value");
            results_list.add("-----------------|-------");
            scan_status_frame.set_label("Ready to scan");
            scan_status_frame.redraw();
            results_list.redraw();
            println!("🗑️ Scan results cleared");
        });
    }

    quit_btn.set_callback(|_| {
        println!("Quit button clicked!"); // debug
        app::quit();
    });

    wind.end();
    wind.show();

    println!(
        "Starting app with {} bytes allocated",
        get_allocated_bytes()
    );

    app.run().unwrap();
}
