mod gcode;

pub struct SimpleMachine {
    program: gcode::GCodeProgram,
    pc: i32,
    step: i32,
    x: (i32, i32),
    y: (i32, i32),
    z: (i32, i32),
    steps_per_unit_x: i32,
    steps_per_unit_y: i32,
    steps_per_unit_z: i32,
}

impl SimpleMachine {
    pub fn new(filepath: String) -> SimpleMachine {
        SimpleMachine {
            program: gcode::parse(filepath),
            pc: 0,
            step: 1,
            x: (0, 0),
            y: (0, 0),
            z: (0, 0),
            steps_per_unit_x: 100,
            steps_per_unit_y: 100,
            steps_per_unit_z: 100,
        }
    }

    pub fn process(&mut self) -> i32 {
        match self.program.get(&self.pc) {
            Some(entry) => {
                match &entry[0].command {
                    // Movement
                    'G' => match &entry[0].subcommand.0 {
                        0 => println!("Rapid movement, {:?}", entry),
                        1 => println!("Interpolated movement: {:?}", entry),
                        _ => println!("Unsupported: {:?}", entry),
                    },
                    'O' => {
                        println!("Set name of section");
                    }
                    _ => println!("Unsupported: {:?}", entry),
                }
                self.pc += self.step;
                0
            }
            None => 1,
        }
    }
}
