use crate::simple_machine;
use std::sync::mpsc;

extern crate sfml;

use sfml::graphics::{
    Color, Drawable, Image, RenderStates, RenderTarget, RenderWindow, Sprite, Texture,
};
use sfml::system::SfBox;
use sfml::window::{Event, Key, Style};

#[derive(Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

struct ToolTrail {
    trail: Vec<Position>,
    tool: simple_machine::ToolState,
    texture: Option<SfBox<Texture>>,
}

impl ToolTrail {
    pub fn new() -> Self {
        ToolTrail {
            trail: vec![],
            tool: simple_machine::ToolState::new(),
            texture: Texture::new(800, 600),
        }
    }

    pub fn add(&mut self, tool: &simple_machine::ToolState) {
        self.trail.push(Position {
            x: tool.x,
            y: tool.y,
            z: tool.z,
        });

        self.tool = tool.clone();
    }

    pub fn update_texture(&mut self) {
        match Image::from_color(800, 600, Color::WHITE) {
            Some(mut image) => {
                let scale = 2.0;
                for pos in &self.trail {
                    if pos.z < 1.0 {
                        image.set_pixel(
                            (scale * pos.x) as u32,
                            (scale * pos.y) as u32,
                            Color::BLUE,
                        );
                    }
                }

                image.set_pixel(
                    (scale * self.tool.x) as u32,
                    (scale * self.tool.y) as u32,
                    Color::RED,
                );

                match &mut self.texture {
                    Some(texture) => {
                        texture.update_from_image(&image, 0, 0);
                    }
                    None => (),
                }
            }
            None => (),
        }
    }

    pub fn draw(&self) -> Sprite {
        let mut sprite = Sprite::new();
        sprite.set_color(Color::WHITE);

        match &self.texture {
            Some(texture) => sprite.set_texture(&texture, true),
            None => (),
        };
        return sprite;
    }
}

impl Drawable for ToolTrail {
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
    let mut tooltrail = ToolTrail::new();

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
            Err(_) => (),
        }

        match toolstate.recv() {
            Ok(entry) => {
                simple_machine::SimpleMachine::update_toolstate(
                    &entry,
                    &toolconfig,
                    &mut current_state,
                );
                tooltrail.add(&current_state);
            }
            Err(_) => {
                //println!("Unable to fetch work item: {:?}", something);
                is_running = false;
            }
        }

        tooltrail.update_texture();
        window.clear(Color::WHITE);
        window.draw(&tooltrail);
        window.display()
    }
}
