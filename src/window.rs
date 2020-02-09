use crate::simple_machine;
use std::env;
use std::f32::consts::PI;
use std::sync::mpsc;

extern crate sfml;

use sfml::graphics::{
    CircleShape, Color, Drawable, RectangleShape, RenderStates, RenderTarget, RenderWindow, Shape,
    Transformable,
};
use sfml::window::{Event, Key, Style};

impl simple_machine::ToolState {
    pub fn draw(&self) -> CircleShape {
        let toolsize = if self.z == 0.0 {
            2.0
        } else {
            (self.z + 1.0) * 2.0
        };
        let mut head = CircleShape::new(toolsize, 100);
        head.set_position((self.x, self.y));
        head.set_fill_color(Color::RED);
        return head;
    }
}

impl Drawable for simple_machine::ToolState {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        render_target: &mut dyn RenderTarget,
        _: RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        render_target.draw(&self.draw());
    }
}

pub fn setup_window(
    toolstate: mpsc::Receiver<simple_machine::SyncEntry>,
    config_sync: mpsc::Receiver<simple_machine::ToolConfig>,
) {
    // Define some constants
    let width = 800;
    let height = 600;

    let mut window = RenderWindow::new(
        (width, height),
        "SimpleMachine",
        Style::CLOSE,
        &Default::default(),
    );
    window.set_vertical_sync_enabled(true);

    let mut is_running = true;
    let mut current_state = simple_machine::ToolState::new();

    while is_running {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed
                | Event::KeyPressed {
                    code: Key::Escape, ..
                } => return,
                _ => {}
            }
        }
        let mut toolconfig = simple_machine::ToolConfig::new();
        match config_sync.recv() {
            Ok(entry) => {
                println!("GUI got config sync: {:?}", &entry);
                toolconfig = entry;
            }
            Err(something) => (),
        }

        match toolstate.recv() {
            Ok(entry) => {
                simple_machine::SimpleMachine::update_toolstate(
                    &entry,
                    &toolconfig,
                    &mut current_state,
                );
            }
            Err(something) => {
                //println!("Unable to fetch work item: {:?}", something);
                is_running = false;
            }
        }

        window.clear(Color::WHITE);
        window.draw(&current_state);
        window.display()
    }
}
