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

#[derive(Debug, PartialEq, Clone)]
pub struct ToolState {
    x: f32,
    y: f32,
    z: f32,
    e: f32,
    steps_per_unit_x: i32,
    steps_per_unit_y: i32,
    steps_per_unit_z: i32,
    steps_per_unit_e: i32,
    feedrate: i32,
}

pub struct SimpleMachine {
    program: gcode::GCodeProgram,
    pc: i32,
    step: i32,
    toolstate: ToolState,
}

#[derive(Debug)]
enum Command {
    StepperX,
    StepperY,
    StepperZ,
    StepperE,
    Feedrate,
}

impl SimpleMachine {
    pub fn new(filepath: String) -> SimpleMachine {
        SimpleMachine {
            program: gcode::parse(filepath),
            pc: 0,
            step: 1,
            toolstate: ToolState {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                e: 0.0,
                steps_per_unit_x: 100,
                steps_per_unit_y: 100,
                steps_per_unit_z: 100,
                steps_per_unit_e: 100,
                feedrate: 1000,
            },
        }
    }

    pub fn process(&mut self) -> i32 {
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
                'F' => next.feedrate = parameter.major,
                _ => println!("Unsupported parameter, {:?}", parameter),
            }
        }

        let mut current = self.toolstate.clone();

        if current.feedrate != next.feedrate {
            current.feedrate = next.feedrate;
            self.add_to_queue((Command::Feedrate, current.feedrate, 0));
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.x, next.x, next.steps_per_unit_x) {
                Some(direction) => self.add_to_queue((Command::StepperX, direction, next.feedrate)),
                None => break,
            };
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.y, next.y, next.steps_per_unit_x) {
                Some(direction) => self.add_to_queue((Command::StepperX, direction, next.feedrate)),
                None => break,
            };
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.z, next.z, next.steps_per_unit_x) {
                Some(direction) => self.add_to_queue((Command::StepperX, direction, next.feedrate)),
                None => break,
            };
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.e, next.e, next.steps_per_unit_x) {
                Some(direction) => self.add_to_queue((Command::StepperX, direction, next.feedrate)),
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
                'F' => next.feedrate = parameter.major,
                _ => println!("Unsupported parameter, {:?}", parameter),
            }
        }

        let mut current = self.toolstate.clone();
        if current.feedrate != next.feedrate {
            current.feedrate = next.feedrate;
            self.add_to_queue((Command::Feedrate, current.feedrate, 0));
        }

        loop {
            let mut added_to_queue = false;

            // Queue stepper instructions
            added_to_queue =
                match fixedresolution_step(&mut current.x, next.x, next.steps_per_unit_x) {
                    Some(direction) => {
                        self.add_to_queue((Command::StepperX, direction, next.feedrate))
                    }
                    None => added_to_queue,
                };

            added_to_queue =
                match fixedresolution_step(&mut current.y, next.y, next.steps_per_unit_x) {
                    Some(direction) => {
                        self.add_to_queue((Command::StepperY, direction, next.feedrate))
                    }
                    None => added_to_queue,
                };

            added_to_queue =
                match fixedresolution_step(&mut current.z, next.z, next.steps_per_unit_x) {
                    Some(direction) => {
                        self.add_to_queue((Command::StepperZ, direction, next.feedrate))
                    }
                    None => added_to_queue,
                };

            added_to_queue =
                match fixedresolution_step(&mut current.e, next.e, next.steps_per_unit_x) {
                    Some(direction) => {
                        self.add_to_queue((Command::StepperE, direction, next.feedrate))
                    }
                    None => added_to_queue,
                };

            if !added_to_queue {
                break;
            }
        }
    }

    fn add_to_queue(&self, entry: (Command, i32, i32)) -> bool {
        true
    }
}
