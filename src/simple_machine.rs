use std::sync::mpsc;
use std::thread;
mod gcode;

fn fixedresolution_add(value: f32, increment: i32, resolution: i32) -> f32 {
    value + (increment as f32 / resolution as f32)
}

fn fixedresolution_equal(value: f32, comp: f32, resolution: i32) -> bool {
    let a = (value * resolution as f32) as i32;
    let b = (comp * resolution as f32) as i32;

    a == b
}

fn fixedresolution_step(current: &mut f32, next: f32, resolution: i32) -> Option<i32> {
    if !fixedresolution_equal(*current, next, resolution) {
        let direction = if *current < next { 1 } else { -1 };
        *current = fixedresolution_add(*current, direction, resolution);
        Some(direction)
    } else {
        None
    }
}

#[derive(Debug)]
enum Command {
    StepperX,
    StepperY,
    StepperZ,
    StepperE,
    Feedrate,
}

#[derive(Debug)]
pub struct CommandEntry {
    command: Command,
    value: i32,
    rate: f32,
}

pub fn start_machine(filepath: String) {
    let (tx, rx) = mpsc::channel::<CommandEntry>();

    let mut machine = SimpleMachine::new(filepath, tx);

    let thread_handle = thread::spawn(move || {
        let mut return_code = 0;
        while return_code == 0 {
            return_code = machine.process();
        }
    });

    for _ in 0..10 {
        println!("WorkItem: {:?}", rx.recv());
    }

    thread_handle.join().expect("Thread failed!");
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToolState {
    x: f32,
    y: f32,
    z: f32,
    e: f32,
    feedrate: f32,
    steps_per_unit_x: i32,
    steps_per_unit_y: i32,
    steps_per_unit_z: i32,
    steps_per_unit_e: i32,
}

pub struct SimpleMachine {
    program: gcode::GCodeProgram,
    pc: i32,
    step: i32,
    queue: mpsc::Sender<CommandEntry>,
    toolstate: ToolState,
}

impl SimpleMachine {
    pub fn new(filepath: String, queue: mpsc::Sender<CommandEntry>) -> SimpleMachine {
        SimpleMachine {
            program: gcode::parse(filepath),
            pc: 0,
            step: 1,
            queue: queue,
            toolstate: ToolState {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                e: 0.0,
                feedrate: 1000.0,
                steps_per_unit_x: 100,
                steps_per_unit_y: 100,
                steps_per_unit_z: 100,
                steps_per_unit_e: 100,
            },
        }
    }

    fn process(&mut self) -> i32 {
        match self.program.get(&self.pc) {
            Some(entry) => {
                match &entry[0].command {
                    // Movement
                    'G' => match &entry[0].major {
                        0 => self.movement_rapid(&entry),
                        1 => self.movement_interpolated(&entry),
                        _ => println!("Unsupported move: {:?}", entry),
                    },
                    'O' => {
                        println!("Set name of section");
                    }
                    _ => println!("Unsupported"),
                }
                self.pc += self.step;
                0
            }
            None => 1,
        }
    }

    fn movement_rapid(&self, parameters: &gcode::GCodeBlock) {
        println!("Rapid movement");
        let mut next = self.toolstate.clone();

        for parameter in parameters.iter().skip(1) {
            match parameter.command {
                'X' => next.x = parameter.major as f32 + parameter.minor,
                'Y' => next.y = parameter.major as f32 + parameter.minor,
                'Z' => next.z = parameter.major as f32 + parameter.minor,
                'E' => next.e = parameter.major as f32 + parameter.minor,
                'F' => next.feedrate = parameter.major as f32 + parameter.minor,
                _ => println!("Unsupported parameter, {:?}", parameter),
            }
        }

        let mut current = self.toolstate.clone();

        if current.feedrate != next.feedrate {
            current.feedrate = next.feedrate;
            self.add_to_queue(CommandEntry {
                command: Command::Feedrate,
                value: 0,
                rate: next.feedrate,
            });
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.x, next.x, next.steps_per_unit_x) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperX,
                    value: direction,
                    rate: next.feedrate,
                }),
                None => break,
            };
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.y, next.y, next.steps_per_unit_x) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperY,
                    value: direction,
                    rate: next.feedrate,
                }),
                None => break,
            };
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.z, next.z, next.steps_per_unit_x) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperZ,
                    value: direction,
                    rate: next.feedrate,
                }),
                None => break,
            };
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.e, next.e, next.steps_per_unit_x) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperE,
                    value: direction,
                    rate: next.feedrate,
                }),
                None => break,
            };
        }
    }

    fn movement_interpolated(&self, parameters: &gcode::GCodeBlock) {
        println!("Interpolated movement");
        let mut next = self.toolstate.clone();

        for parameter in parameters.iter().skip(1) {
            match parameter.command {
                'X' => next.x = parameter.major as f32 + parameter.minor,
                'Y' => next.y = parameter.major as f32 + parameter.minor,
                'Z' => next.z = parameter.major as f32 + parameter.minor,
                'E' => next.e = parameter.major as f32 + parameter.minor,
                'F' => next.feedrate = parameter.major as f32 + parameter.minor,
                _ => println!("Unsupported parameter, {:?}", parameter),
            }
        }

        let mut current = self.toolstate.clone();
        if current.feedrate != next.feedrate {
            current.feedrate = next.feedrate;
            self.add_to_queue(CommandEntry {
                command: Command::Feedrate,
                value: 0,
                rate: next.feedrate,
            });
        }

        loop {
            let mut added_to_queue = false;

            // Queue stepper instructions
            added_to_queue =
                match fixedresolution_step(&mut current.x, next.x, next.steps_per_unit_x) {
                    Some(direction) => self.add_to_queue(CommandEntry {
                        command: Command::StepperX,
                        value: direction,
                        rate: next.feedrate,
                    }),
                    None => added_to_queue,
                };

            added_to_queue =
                match fixedresolution_step(&mut current.y, next.y, next.steps_per_unit_x) {
                    Some(direction) => self.add_to_queue(CommandEntry {
                        command: Command::StepperY,
                        value: direction,
                        rate: next.feedrate,
                    }),
                    None => added_to_queue,
                };

            added_to_queue =
                match fixedresolution_step(&mut current.z, next.z, next.steps_per_unit_x) {
                    Some(direction) => self.add_to_queue(CommandEntry {
                        command: Command::StepperZ,
                        value: direction,
                        rate: next.feedrate,
                    }),
                    None => added_to_queue,
                };

            added_to_queue =
                match fixedresolution_step(&mut current.e, next.e, next.steps_per_unit_x) {
                    Some(direction) => self.add_to_queue(CommandEntry {
                        command: Command::StepperE,
                        value: direction,
                        rate: next.feedrate,
                    }),
                    None => added_to_queue,
                };

            if !added_to_queue {
                break;
            }
        }
    }

    fn add_to_queue(&self, entry: CommandEntry) -> bool {
        self.queue.send(entry).expect("Sent failed!");
        true
    }
}
