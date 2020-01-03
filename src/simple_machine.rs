mod gcode;

pub struct SimpleMachine {
    program: gcode::GCodeProgram,
    pc: i32,
    step: i32,
    x: (i32, i32),
    y: (i32, i32),
    z: (i32, i32),
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
        }
    }

    pub fn process(&mut self) -> i32 {
        match self.program.get(&self.pc) {
            Some(gcode) => {
                println!("pc:{} gcode:{:?}", self.pc, gcode);
                self.pc += self.step;
                0
            }
            None => 1,
        }
    }
}
