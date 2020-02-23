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

    fn draw_square(image: &mut Image, x: u32, y: u32) {
        image.set_pixel(x, y, Color::RED);
        image.set_pixel(x, y + 1, Color::RED);
        image.set_pixel(x, y - 1, Color::RED);

        image.set_pixel(x + 1, y, Color::RED);
        image.set_pixel(x + 1, y + 1, Color::RED);
        image.set_pixel(x + 1, y - 1, Color::RED);

        image.set_pixel(x - 1, y, Color::RED);
        image.set_pixel(x - 1, y + 1, Color::RED);
        image.set_pixel(x - 1, y - 1, Color::RED);
    }

    pub fn update_texture(&mut self) {
        match Image::from_color(800, 600, Color::WHITE) {
            Some(mut image) => {
                let offset_x = 10;
                let offset_y = 10;
                for pos in &self.trail {
                    if pos.z < 1.0 {
                        if ((pos.x > 0.0) && (pos.y > 0.0)) {
                            image.set_pixel(
                                offset_x + pos.x as u32,
                                offset_y + pos.y as u32,
                                Color::BLUE,
                            );
                        }
                    }
                }

                if ((self.tool.x > 0.0) && (self.tool.y > 0.0)) {
                    ToolTrail::draw_square(
                        &mut image,
                        offset_x + self.tool.x as u32,
                        offset_y + self.tool.y as u32,
                    );
                }

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
    let mut sample = 0;
    let sample_frequency = 400;

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
                sample += 1
            }
            Err(_) => {
                //println!("Unable to fetch work item: {:?}", something);
                is_running = false;
            }
        }

        if sample % sample_frequency == 0 {
            tooltrail.update_texture();
            window.clear(Color::WHITE);
            window.draw(&tooltrail);
            window.display()
        }
    }
}
