use crate::gcode;
use crate::window;
use std::sync::mpsc;
use std::thread;

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
                        0 => self.movement_rapid(&entry),
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

    fn movement_rapid(&self, parameters: &gcode::GCodeBlock) -> bool {
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
                value: next.feedrate,
            });
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.x, next.x, self.toolconfig.steps_per_unit_x) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperX,
                    value: direction as f32,
                }),
                None => break,
            };
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.y, next.y, self.toolconfig.steps_per_unit_y) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperY,
                    value: direction as f32,
                }),
                None => break,
            };
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.z, next.z, self.toolconfig.steps_per_unit_z) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperZ,
                    value: direction as f32,
                }),
                None => break,
            };
        }

        loop {
            // Queue stepper instructions
            match fixedresolution_step(&mut current.e, next.e, self.toolconfig.steps_per_unit_e) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperE,
                    value: direction as f32,
                }),
                None => break,
            };
        }

        self.add_to_queue(CommandEntry {
            command: Command::Done,
            value: 0.0,
        });
        return true;
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

        loop {
            let mut added_to_queue = false;

            // Queue stepper instructions
            added_to_queue = match fixedresolution_step(
                &mut current.x,
                next.x,
                self.toolconfig.steps_per_unit_x,
            ) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperX,
                    value: direction as f32,
                }),
                None => added_to_queue,
            };

            added_to_queue = match fixedresolution_step(
                &mut current.y,
                next.y,
                self.toolconfig.steps_per_unit_y,
            ) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperY,
                    value: direction as f32,
                }),
                None => added_to_queue,
            };

            added_to_queue = match fixedresolution_step(
                &mut current.z,
                next.z,
                self.toolconfig.steps_per_unit_z,
            ) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperZ,
                    value: direction as f32,
                }),
                None => added_to_queue,
            };

            added_to_queue = match fixedresolution_step(
                &mut current.e,
                next.e,
                self.toolconfig.steps_per_unit_e,
            ) {
                Some(direction) => self.add_to_queue(CommandEntry {
                    command: Command::StepperE,
                    value: direction as f32,
                }),
                None => added_to_queue,
            };

            if !added_to_queue {
                break;
            }
        }
        self.add_to_queue(CommandEntry {
            command: Command::Done,
            value: 0.0,
        });

        return true;
    }

    fn add_to_queue(&self, entry: CommandEntry) -> bool {
        self.queue.send(entry).expect("Sent failed!");
        true
    }
}
