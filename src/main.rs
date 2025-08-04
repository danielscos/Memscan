// Memscan v0.1.0 high performance, compact, memory scanner tool. built by the goat (Danielscos)
// Main app

use fltk::{
    app, browser::Browser, button::Button, frame::Frame, input::Input, prelude::*, window::Window,
};
use std::{cell::RefCell, rc::Rc};

mod memory;
mod memory_optimization;
mod process;
mod scanner;
mod utils;

use memory_optimization::{get_allocated_bytes, track_allocation};
use process::{Process, enumerate_processes};

// app state
struct AppState {
    selected_process: Option<Process>,
    scan_results: Vec<String>,
    is_attached: bool,
}

impl AppState {
    fn new() -> Self {
        track_allocation(std::mem::size_of::<Self>());
        Self {
            selected_process: None,
            scan_results: Vec::new(),
            is_attached: false,
        }
    }

    fn set_selected_process(&mut self, process: Process) {
        self.selected_process = Some(process);
        self.is_attached = false; // reset attachment status
    }

    fn attach_to_process(&mut self) -> Result<(), String> {
        if let Some(ref mut process) = self.selected_process {
            match process.open() {
                Ok(_) => {
                    self.is_attached = true;
                    Ok(())
                }
                Err(e) => Err(format!("failed to attach to: {}", e)),
            }
        } else {
            Err("No process selected".to_string())
        }
    }

    fn get_memory_usage(&self) -> String {
        let allocated = get_allocated_bytes();
        format!("Memscan Memory Usage: {:.1} KB", allocated as f64 / 1024.0)
    }
}

fn main() {
    println!("Starting Memscan v0.1.0");
    // init fltk
    let app = app::App::default();

    // create window
    let mut wind = Window::new(100, 100, 900, 650, "Memscan - Memory Scanner v0.1.0");
    wind.set_color(fltk::enums::Color::White);

    // create ui elements
    let mut title_frame = Frame::new(
        10,
        10,
        830,
        30,
        "Memscan - Memory Scanner (with optimizarions)",
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

    let mut process_list = Browser::new(10, 110, 600, 250, "");
    process_list.set_type(fltk::browser::BrowserType::Hold);

    let mut list_processes_btn = Button::new(620, 110, 120, 35, "List Processes");
    let mut attach_btn = Button::new(620, 155, 120, 35, "Attach");
    attach_btn.deactivate();

    let mut scan_section = Frame::new(10, 370, 300, 25, "Memory scanning:");
    scan_section.set_label_size(14);
    scan_section.set_label_color(fltk::enums::Color::Black);
    scan_section.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    let mut value_label = Frame::new(10, 405, 150, 20, "Value to find:");
    value_label.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    let mut value_input = Input::new(10, 430, 150, 30, "");
    value_input.set_value("0");

    let mut scan_btn = Button::new(170, 430, 100, 30, "Start Scan");
    scan_btn.deactivate(); // enable after attachment

    let mut scan_type_btn = Button::new(280, 405, 150, 25, "Type: i32 (4 bytes)");
    let scan_types = vec!["i32 (4 bytes)", "i64 (8 bytes)", "f32 (4 bytes)", "String"];
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

    // results
    let mut results_section = Frame::new(10, 440, 300, 25, "Scan Results:");
    results_section.set_label_size(14);
    results_section.set_label_color(fltk::enums::Color::Black);
    results_section.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    let results_list = Browser::new(10, 470, 600, 80, "");

    // status and controls
    let mut status_frame = Frame::new(10, 560, 600, 25, "ready - click 'list processes' to begin");
    status_frame.set_label_size(10);
    status_frame.set_label_color(fltk::enums::Color::DarkRed);
    status_frame.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);

    let mut quit_btn = Button::new(620, 560, 80, 25, "Quit");

    let state = Rc::new(RefCell::new(AppState::new()));

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
                    for process in processes {
                        process_list.add(&format!("{}", process));
                    }
                    status_frame.set_label(&format!(
                        "Found {} processes - select one to attach to",
                        process_list.size()
                    ));

                    // update memory usage
                    memory_frame.set_label(&state.borrow().get_memory_usage());
                }
                Err(e) => {
                    status_frame.set_label(&format!("Error listing processes: {}", e));
                }
            }

            process_list.redraw();
            status_frame.redraw();
            memory_frame.redraw();
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
                let result = {
                    let mut state_mut = state.borrow_mut();
                    state_mut.attach_to_process()
                };

                match result {
                    Ok(_) => {
                        if let Some(ref process) = state.borrow().selected_process {
                            status_frame.set_label(&format!(
                                "attached to {} - ready to scan memory",
                                process
                            ));
                            scan_btn.activate(); // enable scanning
                            println!("Successfully attached to {}", process); //debug
                        }
                    }
                    Err(e) => {
                        status_frame.set_label(&format!("{}", e));
                        println!("attach failed: {}", e);
                    }
                }
            } else {
                status_frame.set_label("No process selected");
                print!("no process selected")
            }

            status_frame.redraw();
        });
    }

    {
        let state = state.clone();
        let mut status_frame = status_frame.clone();
        let mut results_list = results_list.clone();
        let value_input = value_input.clone();
        let scan_type_index = scan_type_index.clone();

        scan_btn.set_callback(move |_| {
            let search_value = value_input.value();
            let selected_type = *scan_type_index.borrow();

            if search_value.is_empty() {
                status_frame.set_label("Please enter a value to search for");
                status_frame.redraw();
                return;
            }

            if let Some(ref process) = state.borrow().selected_process {
                if let Some(ref handle) = process.handle {
                    match handle.get_scannable_regions() {
                        Ok(regions) => {
                            results_list.clear();
                            let mut found_count = 0;

                            // Use the selected type in your search
                            match selected_type {
                                0 => {
                                    // i32
                                    if let Ok(target_value) = search_value.parse::<i32>() {
                                        status_frame.set_label(&format!(
                                            "Searching for i32 value: {}",
                                            target_value
                                        ));
                                        for region in regions.iter().take(5) {
                                            let chunk_size = 4096;
                                            for addr in (region.start_address
                                                ..region.start_address
                                                    + region.size.min(chunk_size))
                                                .step_by(4)
                                            {
                                                if let Ok(value) = handle.read_i32(addr) {
                                                    if value == target_value {
                                                        results_list.add(&format!(
                                                            "Found {} at 0x{:X}",
                                                            value, addr
                                                        ));
                                                        found_count += 1;
                                                        if found_count >= 10 {
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            if found_count >= 10 {
                                                break;
                                            }
                                        }
                                    }
                                }
                                1 => {
                                    // i64
                                    if let Ok(target_value) = search_value.parse::<i64>() {
                                        status_frame.set_label(&format!(
                                            "Searching for i64 value: {}",
                                            target_value
                                        ));
                                        results_list.add("i64 search not implemented yet");
                                        found_count = 1;
                                    }
                                }
                                2 => {
                                    // f32
                                    if let Ok(target_value) = search_value.parse::<f32>() {
                                        status_frame.set_label(&format!(
                                            "Searching for f32 value: {}",
                                            target_value
                                        ));
                                        results_list.add("f32 search not implemented yet");
                                        found_count = 1;
                                    }
                                }
                                _ => {
                                    // String
                                    status_frame.set_label("String search not implemented yet");
                                    results_list.add("String search coming soon");
                                    found_count = 1;
                                }
                            }

                            status_frame.set_label(&format!(
                                "Search complete. Found {} matches",
                                found_count
                            ));
                        }
                        Err(e) => {
                            status_frame.set_label(&format!("Search failed: {}", e));
                        }
                    }
                }
            }

            status_frame.redraw();
            results_list.redraw();
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
