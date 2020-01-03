use std::env;
mod simple_machine;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1) {
        Some(argument) => {
            let filepath = argument.to_string();

            let mut simulator = simple_machine::SimpleMachine::new(filepath);

            let mut return_code = 0;
            while return_code == 0 {
                return_code = simulator.process();
            }
        }
        None => {
            println!("Unable to parse arguments: {:?}", &args);
        }
    }
}
