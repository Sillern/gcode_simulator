use crate::gcode;
use crate::window;
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
        return (self.raw_value as f32 / self.resolution as f32);
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
    pub fn multiply(&self, value: FixedResolution) -> FixedResolution {
        return FixedResolution {
            raw_value: self.raw_value * value.raw_value,
            resolution: self.resolution,
        };
    }

    pub fn equal(&self, comp: &FixedResolution) -> bool {
        self.raw_value == comp.raw_value
    }

    pub fn less_than(&self, comp: &FixedResolution) -> bool {
        self.raw_value < comp.raw_value
    }

    pub fn next_value(&self, next: &FixedResolution) -> Option<FixedResolution> {
        match self.get_direction(next) {
            Some(direction) => Some(self.increment(direction)),
            None => None,
        }
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

    pub fn lerp(
        &self,
        start_x: &FixedResolution,
        start_y: &FixedResolution,
        stop_x: &FixedResolution,
        stop_y: &FixedResolution,
    ) -> FixedResolution {
        FixedResolution {
            raw_value: (start_y.raw_value * (stop_x.raw_value - self.raw_value)
                + stop_y.raw_value * (self.raw_value - start_x.raw_value))
                / (stop_x.raw_value - start_x.raw_value),
            resolution: stop_y.resolution,
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
) {
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

                    if (counter as f32) % (gui_syncentry.rate / 20.0) == 0.0 {
                        toolstate.send(gui_syncentry.clone()).expect("Sent failed!");
                        // Reset the counters
                        gui_syncentry.steps_x = 0;
                        gui_syncentry.steps_y = 0;
                        gui_syncentry.steps_z = 0;
                        gui_syncentry.steps_e = 0;
                    }

                    counter += 1;
                }
                Err(something) => {
                    println!("Unable to fetch work item: {:?}", something);
                    return_code = 1;
                }
            }
        }
    });

    machine_thread_handle.join().expect("Thread failed!");
    stepper_thread_handle.join().expect("Thread failed!");
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
        construct.config_sync.send(construct.toolconfig.clone());

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

            let mut added_to_queue = 0;
            added_to_queue += match current_x.get_direction(&normalized_vector.0) {
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

            added_to_queue += match current_y.get_direction(&normalized_vector.1) {
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

            added_to_queue += match current_z.get_direction(&normalized_vector.2) {
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

            added_to_queue += match current_e.get_direction(&normalized_vector.3) {
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

    fn add_to_queue(&self, entry: CommandEntry) -> i32 {
        self.queue.send(entry).expect("Sent failed!");
        1
    }
}
