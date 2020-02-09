mod gcode;
mod simple_machine;
mod window;
use std::env;
use std::sync::mpsc;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (toolstate_tx, toolstate_rx) = mpsc::channel::<simple_machine::SyncEntry>();
    let (config_tx, config_rx) = mpsc::channel::<simple_machine::ToolConfig>();

    let mut threads = match args.get(1) {
        Some(argument) => {
            let filepath = argument.to_string();

            simple_machine::start_machine(filepath, toolstate_tx, config_tx)
        }
        None => {
            println!("Unable to parse arguments: {:?}", &args);
            vec![]
        }
    };

    let gui_thread_handle = thread::spawn(move || {
        window::setup_window(toolstate_rx, config_rx);
    });

    gui_thread_handle.join().expect("Thread failed!");
    for thread_handle in threads {
        thread_handle.join().expect("Thread failed!");
    }
}
