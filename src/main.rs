use fltk::{app, button::Button, frame::Frame, prelude::*, valuator::Counter, window::Window};
use std::alloc::alloc;
use std::os::linux::raw::stat;
use std::rc::Rc;
use std::{cell::RefCell, fmt::format};

mod memory_utils;
use memory_utils::{ObjectPool, get_allocated_bytes, track_allocation};

// app state with minimal memory footprint
struct AppState {
    counter: i32,
    total_allocations: usize,
}

impl AppState {
    fn new() -> Self {
        track_allocation(std::mem::size_of::<Self>());
        Self {
            counter: 0,
            total_allocations: 0,
        }
    }

    fn increment(&mut self) {
        self.counter += 1;
        self.total_allocations += 1;
        // track a small alloc for demo
        track_allocation(32);
    }

    fn reset(&mut self) {
        self.counter = 0;
    }

    fn get_memory_info(&self) -> (usize, usize, usize) {
        let allocated = get_allocated_bytes();
        let app_size = std::mem::size_of::<Self>();
        let estimated_ui = self.total_allocations * 32; // rough estimate
        (allocated, app_size, estimated_ui)
    }
}

fn main() {
    // init fltk
    let app = app::App::default();

    // create window
    let mut wind = Window::new(100, 100, 500, 400, "Memory Optimized FLTK App");
    wind.set_color(fltk::enums::Color::White);

    // create ui elements
    let mut title_frame = Frame::new(10, 10, 480, 30, "Memoru Optimized FLTK app");
    title_frame.set_label_size(16);
    title_frame.set_label_color(fltk::enums::Color::Black);

    let mut counter_frame = Frame::new(10, 50, 480, 30, "Counter: 0");
    counter_frame.set_label_size(14);
    counter_frame.set_label_color(fltk::enums::Color::DarkBlue);

    let mut memory_frame = Frame::new(10, 90, 430, 60, "Memory usafe will appear here");
    memory_frame.set_label_size(11);
    memory_frame.set_label_color(fltk::enums::Color::DarkGreen);

    let mut status_frame = Frame::new(10, 160, 480, 30, "Ready");
    status_frame.set_label_size(10);
    status_frame.set_label_color(fltk::enums::Color::DarkRed);

    let mut increment_btn = Button::new(10, 200, 100, 40, "Increment");
    let mut memory_btn = Button::new(120, 200, 120, 40, "Memory");
    let mut reset_btn = Button::new(250, 200, 80, 40, "Reset");
    let mut stress_btn = Button::new(340, 200, 80, 40, "Stress Test");
    let mut quit_btn = Button::new(10, 250, 80, 40, "Quit");

    // shared app state
    let state = Rc::new(RefCell::new(AppState::new()));

    {
        let state = state.clone();
        let mut counter_frame = counter_frame.clone();
        let mut status_frame = status_frame.clone();

        increment_btn.set_callback(move |_| {
            println!("Increment button clicked!"); // debug

            // update state
            state.borrow_mut().increment();
            let count = state.borrow().counter;

            // update ui
            counter_frame.set_label(&format!("Counter {}", count));
            status_frame.set_label(&format!("Last action: Incremented to {}", count));

            // force redraw
            counter_frame.redraw();
            status_frame.redraw();
        });
    }

    // memory check button callback

    {
        let state = state.clone();
        let mut memory_frame = memory_frame.clone();
        let mut status_frame = status_frame.clone();

        memory_btn.set_callback(move |_| {
            println!("Memory button clicked!");

            let (allocated, app_size, estimated_ui) = state.borrow().get_memory_info();

            let memory_text = format!(
                "allocated: {} bytes ({:.1} KB)\nApp struvt: {} bytes\nUI alloc: ~{} bytes",
                allocated,
                allocated as f64 / 1024.0,
                app_size,
                estimated_ui
            );

            memory_frame.set_label(&memory_text);
            status_frame.set_label("Memory usage updated");

            memory_frame.redraw();
            status_frame.redraw();
        });
    }

    // reset button callback

    {
        let state = state.clone();
        let mut counter_frame = counter_frame.clone();
        let mut status_frame = status_frame.clone();

        reset_btn.set_callback(move |_| {
            println!("Reset button clicked!");

            state.borrow_mut().reset();
            counter_frame.set_label("Counter: 0");
            status_frame.set_label("Counter reset");

            counter_frame.redraw();
            status_frame.redraw();
        });
    }

    // stress test button
    {
        let state = state.clone();
        let mut status_frame = status_frame.clone();

        stress_btn.set_callback(move |_| {
            println!("Stress test started!"); // debug

            for i in 0..100 {
                let _v: Vec<u8> = vec![0; 1024]; // 1kb each
                state.borrow_mut().total_allocations += 1;
            }

            status_frame.set_label("Stress test completed (100 x 1kn allocations)");
            status_frame.redraw();
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
