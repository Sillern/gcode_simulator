use image::{ImageBuffer, RgbImage};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

type Coordinate = (i64, i64, i64);
type Pixels = HashMap<Coordinate, u32>;

pub fn draw_frame(pixels: &Pixels, framenumber: i32) {
    type Color = (u8, u8, u8);

    let scale = 3;
    let border = 4;
    let size = (600, 600);
    let offset = (0, 0);
    let real_size = (
        ((size.0 + border * 2) * scale as u32),
        ((size.1 + border * 2) * scale as u32),
    );

    fn draw_square(img: &mut RgbImage, coord: (u32, u32), color: Color, size: u32, offset: u32) {
        let color_pixel = image::Rgb([color.0, color.1, color.2]);
        let black_pixel = image::Rgb([0, 0, 0]);
        for x in 0..size {
            for y in 0..size {
                img.put_pixel(
                    size * (coord.0 + offset) + x,
                    size * (coord.1 + offset) + y,
                    black_pixel,
                );
            }
        }

        for x in 1..size - 1 {
            for y in 1..size - 1 {
                img.put_pixel(
                    size * (coord.0 + offset) + x,
                    size * (coord.1 + offset) + y,
                    color_pixel,
                );
            }
        }
    }

    let mut img = ImageBuffer::from_fn(real_size.0, real_size.1, |_x, _y| {
        image::Rgb([255, 255, 255])
    });

    for (pos, pixel) in pixels {
        let color = match pixel {
            0 => (0xCC, 0xCC, 0xEE),
            1 => (0x12, 0x33, 0x12),
            2 => (0x44, 0x88, 0xAA),
            3 => (0x12, 0x12, 0xCC),
            4 => (0xCC, 0x12, 0x12),
            _ => (0xDD, 0xDD, 0xDD),
        };

        let x = (pos.0 - offset.0) as i32;
        let y = (pos.1 - offset.1) as i32;

        if x >= 0 && y >= 0 && x < size.0 as i32 && y < size.1 as i32 {
            draw_square(&mut img, (x as u32, y as u32), color, scale, border);
        }
    }

    img.save(format!("frames/frame{:05}.png", framenumber))
        .unwrap();
}
