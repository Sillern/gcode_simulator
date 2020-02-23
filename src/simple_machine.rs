use crate::gcode;
use std::f32;
use std::f32::consts::PI;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Copy)]
struct FixedResolution {
    raw_value: i64,
    resolution: i32,
}

impl FixedResolution {
    pub fn new(value: f32, resolution: i32) -> Self {
        Self {
            raw_value: (value * resolution as f32) as i64,
            resolution: resolution,
        }
    }
    pub fn repr(&self) -> f32 {
        return self.raw_value as f32 / self.resolution as f32;
    }

    pub fn increment(&self, direction: i32) -> FixedResolution {
        return FixedResolution {
            raw_value: self.raw_value + direction as i64,
            resolution: self.resolution,
        };
    }

    pub fn add(&self, value: FixedResolution) -> FixedResolution {
        return FixedResolution {
            raw_value: self.raw_value + value.raw_value,
            resolution: self.resolution,
        };
    }

    pub fn subtract(&self, value: FixedResolution) -> FixedResolution {
        return FixedResolution {
            raw_value: self.raw_value - value.raw_value,
            resolution: self.resolution,
        };
    }

    pub fn multiply_raw(&self, value: f32) -> FixedResolution {
        return FixedResolution {
            raw_value: ((self.raw_value as f32) * value) as i64,
            resolution: self.resolution,
        };
    }

    pub fn equal(&self, comp: &FixedResolution) -> bool {
        self.raw_value == comp.raw_value
    }

    pub fn less_than(&self, comp: &FixedResolution) -> bool {
        self.raw_value < comp.raw_value
    }

    pub fn get_direction(&self, next: &FixedResolution) -> Option<i32> {
        if self.equal(next) {
            None
        } else if self.less_than(next) {
            Some(1)
        } else {
            Some(-1)
        }
    }
}

#[derive(Debug)]
enum Command {
    StepperX,
    StepperY,
    StepperZ,
    StepperE,
    Feedrate,
    Done,
    Quit,
}

#[derive(Debug)]
pub struct CommandEntry {
    command: Command,
    value: f32,
}

#[derive(Debug, Clone)]
pub struct SyncEntry {
    steps_x: i32,
    steps_y: i32,
    steps_z: i32,
    steps_e: i32,
    rate: f32,
}

impl SyncEntry {
    fn new() -> Self {
        SyncEntry {
            steps_x: 0,
            steps_y: 0,
            steps_z: 0,
            steps_e: 0,
            rate: 0.0,
        }
    }
}

pub fn start_machine(
    filepath: String,
    toolstate: mpsc::Sender<SyncEntry>,
    config_sync: mpsc::Sender<ToolConfig>,
) -> Vec<thread::JoinHandle<()>> {
    let (tx, rx) = mpsc::channel::<CommandEntry>();
    let (sync_tx, sync_rx) = mpsc::channel::<SyncEntry>();

    let machine_thread_handle = thread::spawn(move || {
        let mut machine = SimpleMachine::new(filepath, tx.clone(), sync_rx, config_sync.clone());

        let mut return_code = 0;
        while return_code == 0 {
            return_code = machine.process();
        }

        tx.send(CommandEntry {
            command: Command::Quit,
            value: 0.0,
        })
        .expect("Sent failed!");
    });

    let stepper_thread_handle = thread::spawn(move || {
        let mut syncentry = SyncEntry::new();
        let mut gui_syncentry = SyncEntry::new();
        let mut return_code = 0;
        let mut counter = 0;
        while return_code == 0 {
            match rx.recv() {
                Ok(entry) => {
                    let sigmoid_value = if entry.value < 0.0 {
                        -1
                    } else if entry.value > 0.0 {
                        1
                    } else {
                        0
                    };

                    match &entry.command {
                        Command::StepperX => {
                            syncentry.steps_x += sigmoid_value;
                            gui_syncentry.steps_x += sigmoid_value;
                        }
                        Command::StepperY => {
                            syncentry.steps_y += sigmoid_value;
                            gui_syncentry.steps_y += sigmoid_value;
                        }
                        Command::StepperZ => {
                            syncentry.steps_z += sigmoid_value;
                            gui_syncentry.steps_z += sigmoid_value;
                        }
                        Command::StepperE => {
                            syncentry.steps_e += sigmoid_value;
                            gui_syncentry.steps_e += sigmoid_value;
                        }
                        Command::Feedrate => {
                            syncentry.rate = entry.value;
                            gui_syncentry.rate = entry.value;
                        }
                        Command::Done => {
                            sync_tx.send(syncentry.clone()).expect("Sent failed!");

                            // Reset the counters
                            syncentry.steps_x = 0;
                            syncentry.steps_y = 0;
                            syncentry.steps_z = 0;
                            syncentry.steps_e = 0;
                        }
                        Command::Quit => {
                            return_code = 1;
                        }
                    };

                    toolstate.send(gui_syncentry.clone()).expect("Sent failed!");

                    // Reset the counters
                    gui_syncentry.steps_x = 0;
                    gui_syncentry.steps_y = 0;
                    gui_syncentry.steps_z = 0;
                    gui_syncentry.steps_e = 0;

                    counter += 1;
                }
                Err(something) => {
                    println!("Unable to fetch work item: {:?}", something);
                    return_code = 1;
                }
            }
        }
    });

    return vec![machine_thread_handle, stepper_thread_handle];
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToolState {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub e: f32,
    pub feedrate: f32,
}
impl ToolState {
    pub fn new() -> Self {
        ToolState {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            e: 0.0,
            feedrate: 1000.0,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToolConfig {
    pub steps_per_unit_x: i32,
    pub steps_per_unit_y: i32,
    pub steps_per_unit_z: i32,
    pub steps_per_unit_e: i32,
}
impl ToolConfig {
    pub fn new() -> Self {
        ToolConfig {
            steps_per_unit_x: 100,
            steps_per_unit_y: 100,
            steps_per_unit_z: 100,
            steps_per_unit_e: 100,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        let resolution = 1000.0;
        Position {
            x: (((x * resolution) as i64) as f32) / resolution,
            y: (((y * resolution) as i64) as f32) / resolution,
        }
    }
}

fn calculate_angles(start_pos: Position, end_pos: Position, center: Position) -> (f32, f32, f32) {
    let radius = (center.x * center.x + center.y * center.y).sqrt();

    let start_angle = (-1.0 * center.y).atan2(-1.0 * center.x);
    let stop_angle =
        (end_pos.y - (start_pos.y + center.y)).atan2(end_pos.x - (start_pos.x + center.x));

    return (radius, start_angle, stop_angle);
}

fn calculate_units(start_angle: f32, stop_angle: f32) -> (Position, Position) {
    let start_unit = Position::new(start_angle.cos(), start_angle.sin());
    let stop_unit = Position::new(stop_angle.cos(), stop_angle.sin());
    return (start_unit, stop_unit);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_units() {
        {
            let (_, start_angle, stop_angle) = calculate_angles(
                Position { x: 10.0, y: 10.0 },
                Position { x: 20.0, y: 20.0 },
                Position { x: 0.0, y: 10.0 },
            );

            let result = calculate_units(start_angle, stop_angle);

            assert_eq!(Position { x: 0.0, y: -1.0 }, result.0);
            assert_eq!(Position { x: 1.0, y: 0.0 }, result.1);
        }
        {
            let (_, start_angle, stop_angle) = calculate_angles(
                Position { x: 30.0, y: 30.0 },
                Position { x: 40.0, y: 40.0 },
                Position { x: 10.0, y: 0.0 },
            );

            let result = calculate_units(start_angle, stop_angle);

            assert_eq!(Position { x: -1.0, y: 0.0 }, result.0);
            assert_eq!(Position { x: 0.0, y: 1.0 }, result.1);
        }
    }

    #[test]
    fn test_calculate_angles() {
        {
            let (radius, start_angle, stop_angle) = calculate_angles(
                Position { x: 10.0, y: 10.0 },
                Position { x: 20.0, y: 20.0 },
                Position { x: 0.0, y: 10.0 },
            );

            assert_eq!(-0.5 * PI, start_angle);
            assert_eq!(0.0, stop_angle);
            assert_eq!(10.0, radius);
        }
        {
            let (radius, start_angle, stop_angle) = calculate_angles(
                Position { x: 30.0, y: 30.0 },
                Position { x: 40.0, y: 40.0 },
                Position { x: 10.0, y: 0.0 },
            );

            assert_eq!(-1.0 * PI, start_angle);
            assert_eq!(0.5 * PI, stop_angle);
            assert_eq!(10.0, radius);
        }
    }
}

pub struct SimpleMachine {
    program: gcode::GCodeProgram,
    pc: i32,
    step: i32,
    queue: mpsc::Sender<CommandEntry>,
    sync: mpsc::Receiver<SyncEntry>,
    config_sync: mpsc::Sender<ToolConfig>,
    toolstate: ToolState,
    toolconfig: ToolConfig,
}

impl SimpleMachine {
    pub fn new(
        filepath: String,
        queue: mpsc::Sender<CommandEntry>,
        sync: mpsc::Receiver<SyncEntry>,
        config_sync: mpsc::Sender<ToolConfig>,
    ) -> SimpleMachine {
        let construct = SimpleMachine {
            program: gcode::parse(filepath),
            pc: 0,
            step: 1,
            queue: queue,
            sync: sync,
            config_sync: config_sync,
            toolstate: ToolState::new(),
            toolconfig: ToolConfig::new(),
        };
        construct
            .config_sync
            .send(construct.toolconfig.clone())
            .expect("Sent failed!");

        return construct;
    }

    pub fn update_toolstate(entry: &SyncEntry, toolconfig: &ToolConfig, toolstate: &mut ToolState) {
        toolstate.x += entry.steps_x as f32 / toolconfig.steps_per_unit_x as f32;
        toolstate.y += entry.steps_y as f32 / toolconfig.steps_per_unit_y as f32;
        toolstate.z += entry.steps_z as f32 / toolconfig.steps_per_unit_z as f32;
        toolstate.e += entry.steps_e as f32 / toolconfig.steps_per_unit_e as f32;
        toolstate.feedrate = entry.rate;
    }

    fn set_toolstate(&mut self, entry: &SyncEntry) {
        SimpleMachine::update_toolstate(&entry, &self.toolconfig, &mut self.toolstate);
    }

    fn process(&mut self) -> i32 {
        match self.program.get(&self.pc) {
            Some(entry) => {
                let command_sent = match &entry[0].command {
                    // Movement
                    'G' => match &entry[0].major {
                        0 => self.movement_interpolated(&entry),
                        1 => self.movement_interpolated(&entry),
                        2 => self.movement_arc(&entry, true),
                        3 => self.movement_arc(&entry, false),
                        _ => {
                            println!("Unsupported move: {:?}", entry);
                            false
                        }
                    },
                    'O' => {
                        println!("Set name of section");
                        false
                    }
                    _ => {
                        println!("Unsupported");
                        false
                    }
                };

                if command_sent {
                    // Sync the current toolstate
                    match self.sync.recv() {
                        Ok(entry) => {
                            self.set_toolstate(&entry);
                            println!("Sync: WorkItem: {:?}, state: {:?}", &entry, &self.toolstate);
                        }
                        Err(_) => {
                            println!("Unable to fetch work item");
                        }
                    }
                }
                self.pc += self.step;
                0
            }
            None => 1,
        }
    }

    fn movement_interpolated(&self, parameters: &gcode::GCodeBlock) -> bool {
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
                value: next.feedrate,
            });
        }

        let start_x = FixedResolution::new(current.x, self.toolconfig.steps_per_unit_x);
        let start_y = FixedResolution::new(current.y, self.toolconfig.steps_per_unit_y);
        let start_z = FixedResolution::new(current.z, self.toolconfig.steps_per_unit_z);
        let start_e = FixedResolution::new(current.e, self.toolconfig.steps_per_unit_e);
        let mut current_x = start_x.clone();
        let mut current_y = start_y.clone();
        let mut current_z = start_z.clone();
        let mut current_e = start_e.clone();
        let stop_x = FixedResolution::new(next.x, self.toolconfig.steps_per_unit_x);
        let stop_y = FixedResolution::new(next.y, self.toolconfig.steps_per_unit_y);
        let stop_z = FixedResolution::new(next.z, self.toolconfig.steps_per_unit_z);
        let stop_e = FixedResolution::new(next.e, self.toolconfig.steps_per_unit_e);

        let movement_vector = (
            stop_x.subtract(start_x),
            stop_y.subtract(start_y),
            stop_z.subtract(start_z),
            stop_e.subtract(start_e),
        );

        let movement_amplitude = ((movement_vector.0.raw_value * movement_vector.0.raw_value
            + movement_vector.1.raw_value * movement_vector.1.raw_value
            + movement_vector.2.raw_value * movement_vector.2.raw_value
            + movement_vector.3.raw_value * movement_vector.3.raw_value)
            as f32)
            .sqrt();

        let mut step = 0;
        loop {
            let factor = if movement_amplitude != 0.0 {
                (step as f32) / movement_amplitude
            } else {
                movement_amplitude
            };
            let normalized_vector = (
                start_x.add(movement_vector.0.multiply_raw(factor)),
                start_y.add(movement_vector.1.multiply_raw(factor)),
                start_z.add(movement_vector.2.multiply_raw(factor)),
                start_e.add(movement_vector.3.multiply_raw(factor)),
            );

            match current_x.get_direction(&normalized_vector.0) {
                Some(direction) => {
                    current_x = current_x.increment(direction);
                    self.add_to_queue(CommandEntry {
                        command: Command::StepperX,
                        value: direction as f32,
                    })
                }
                None => {
                    if current_x.equal(&stop_x) {
                        0
                    } else {
                        1
                    }
                }
            };

            match current_y.get_direction(&normalized_vector.1) {
                Some(direction) => {
                    current_y = current_y.increment(direction);
                    self.add_to_queue(CommandEntry {
                        command: Command::StepperY,
                        value: direction as f32,
                    })
                }
                None => {
                    if current_y.equal(&stop_y) {
                        0
                    } else {
                        1
                    }
                }
            };

            match current_z.get_direction(&normalized_vector.2) {
                Some(direction) => {
                    current_z = current_z.increment(direction);
                    self.add_to_queue(CommandEntry {
                        command: Command::StepperZ,
                        value: direction as f32,
                    })
                }
                None => {
                    if current_z.equal(&stop_z) {
                        0
                    } else {
                        1
                    }
                }
            };

            match current_e.get_direction(&normalized_vector.3) {
                Some(direction) => {
                    current_e = current_e.increment(direction);
                    self.add_to_queue(CommandEntry {
                        command: Command::StepperE,
                        value: direction as f32,
                    })
                }
                None => {
                    if current_e.equal(&stop_e) {
                        0
                    } else {
                        1
                    }
                }
            };

            if step > movement_amplitude as i32 {
                break;
            }

            step += 1;
        }

        self.add_to_queue(CommandEntry {
            command: Command::Done,
            value: 0.0,
        });

        return true;
    }

    fn movement_arc(&self, parameters: &gcode::GCodeBlock, clockwise: bool) -> bool {
        println!("Arc movement");
        let mut next = self.toolstate.clone();
        let mut center: (f32, f32) = (0.0, 0.0);
        for parameter in parameters.iter().skip(1) {
            match parameter.command {
                'X' => next.x = parameter.major as f32 + parameter.minor,
                'Y' => next.y = parameter.major as f32 + parameter.minor,
                'I' => center.0 = parameter.major as f32 + parameter.minor,
                'J' => center.1 = parameter.major as f32 + parameter.minor,
                'E' => next.e = parameter.major as f32 + parameter.minor,
                'F' => next.feedrate = parameter.major as f32 + parameter.minor,
                _ => println!("Unsupported parameter, {:?}", parameter),
            }
        }

        let mut current = self.toolstate.clone();

        let (radius, start_angle, raw_stop_angle) = calculate_angles(
            Position::new(current.x, current.y),
            Position::new(next.x, next.y),
            Position::new(center.0, center.1),
        );
        let stop_angle = raw_stop_angle - if clockwise { 0.0 } else { 2.0 * PI };

        let mut angle = FixedResolution::new(
            start_angle,
            self.toolconfig.steps_per_unit_x * self.toolconfig.steps_per_unit_x,
        );
        let stop = FixedResolution::new(
            stop_angle,
            self.toolconfig.steps_per_unit_x * self.toolconfig.steps_per_unit_y,
        );

        let start_x = FixedResolution::new(current.x, self.toolconfig.steps_per_unit_x);
        let start_y = FixedResolution::new(current.y, self.toolconfig.steps_per_unit_y);
        let center_x = FixedResolution::new(current.x + center.0, self.toolconfig.steps_per_unit_x);
        let center_y = FixedResolution::new(current.y + center.1, self.toolconfig.steps_per_unit_y);
        let mut current_x = start_x.clone();
        let mut current_y = start_y.clone();
        let stop_x = FixedResolution::new(next.x, self.toolconfig.steps_per_unit_x);
        let stop_y = FixedResolution::new(next.y, self.toolconfig.steps_per_unit_y);

        if current.feedrate != next.feedrate {
            current.feedrate = next.feedrate;
            self.add_to_queue(CommandEntry {
                command: Command::Feedrate,
                value: next.feedrate,
            });
        }

        loop {
            match angle.get_direction(&stop) {
                Some(direction) => {
                    angle = angle.increment(direction);
                    let cartesian = ((radius * angle.repr().cos()), (radius * angle.repr().sin()));
                    let x = (center_x.repr() + cartesian.0);
                    let y = (center_y.repr() + cartesian.1);

                    let next_x = FixedResolution::new(x, self.toolconfig.steps_per_unit_x);
                    let next_y = FixedResolution::new(y, self.toolconfig.steps_per_unit_y);

                    match current_x.get_direction(&next_x) {
                        Some(direction) => {
                            current_x = current_x.increment(direction);
                            self.add_to_queue(CommandEntry {
                                command: Command::StepperX,
                                value: direction as f32,
                            });
                        }
                        None => {}
                    };

                    match current_y.get_direction(&next_y) {
                        Some(direction) => {
                            current_y = current_y.increment(direction);
                            self.add_to_queue(CommandEntry {
                                command: Command::StepperY,
                                value: direction as f32,
                            });
                        }
                        None => {}
                    };

                    if current_x.equal(&stop_x) && current_y.equal(&stop_y) {
                        0
                    } else {
                        1
                    }
                }
                None => {
                    if angle.equal(&stop) {
                        0
                    } else {
                        1
                    }
                }
            };

            if angle.equal(&stop) {
                break;
            }
        }

        self.add_to_queue(CommandEntry {
            command: Command::Done,
            value: 0.0,
        });

        return true;
    }

    fn add_to_queue(&self, entry: CommandEntry) -> i32 {
        self.queue.send(entry).expect("Sent failed!");
        1
    }
}
