use std::{thread, time::Duration};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::Size,
    prelude::{Point, RgbColor},
    primitives::PrimitiveStyleBuilder,
    Drawable,
};

use esp_idf_sys as _;
use log::info;
use maze_painter::MazePainter;

mod gt911;
mod hx8369;
mod maze;
mod maze_painter;
mod utils;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    unsafe {
        while !gt911::gt911_init(gt911::GT911_I2C_SLAVE_ADDR) {
            info!("GT911 init failed, retrying...");
            thread::sleep(Duration::from_millis(100));
            gt911::GT911_RST();
        }
    };

    info!("Running demo");

    run_maze();
}

fn run_maze() {
    let mut display = hx8369::HX8369::new(800, 480);

    display.fill(Rgb565::BLACK);

    let mut maze = maze::Maze::new(38, 22);
    maze.generate(0, 0);

    let style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::YELLOW)
        .stroke_color(Rgb565::WHITE)
        .stroke_width(1)
        .build();

    let offset = Point { x: 25, y: 20 };
    let cell_size = Size::new(20, 20);

    let mut painter = MazePainter::new(maze, style, cell_size, offset);

    painter.draw(&mut display).ok();

    display.flush();

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::GREEN)
        .stroke_width(3)
        .build();

    loop {
        if let Some(state) = gt911::read_touch() {
            info!("state: {:?}", state);
            if state.pressed
                && painter.on_click(state.x as i32, state.y as i32, style, &mut display)
            {
                display.flush();
            }
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
