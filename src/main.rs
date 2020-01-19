use std::env;
mod simple_machine;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(argument) => {
            let filepath = argument.to_string();

            simple_machine::start_machine(filepath)
        }
        None => {
            println!("Unable to parse arguments: {:?}", &args);
        }
    }
}
