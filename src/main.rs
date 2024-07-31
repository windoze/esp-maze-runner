use std::{thread, time::Duration};

use embedded_graphics::Drawable;
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::{Rgb565, RgbColor},
    primitives::PrimitiveStyleBuilder,
};
use esp_idf_svc::hal::{
    delay::Ets,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    units::FromValueType,
};
use gt911::GT911Builder;
use log::info;
use maze_painter::MazePainter;

mod gt911;
mod hx8369;
mod maze;
mod maze_painter;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 480;
const CELL_SIZE: usize = 20;
const MAZE_WIDTH: usize = 38;
const MAZE_HEIGHT: usize = 22;
const X_OFFSET: u16 = 25;
const Y_OFFSET: u16 = 20;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    let i2c = peripherals.i2c0;
    let sda = pins.gpio39;
    let scl = pins.gpio38;
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;
    let rst = PinDriver::output(pins.gpio4)?; // reset pin on GT911
    let builder = GT911Builder::new(i2c, rst, Ets)
        .address(0x5d)
        .orientation(gt911::Orientation::InvertedPortrait)
        .size(SCREEN_WIDTH as u16, SCREEN_HEIGHT as u16);
    let mut touch_screen = builder.build();

    // The board needs to set the pin 6 to high before resetting the touch screen
    PinDriver::output(pins.gpio6)?.set_high()?;
    thread::sleep(Duration::from_millis(5));
    touch_screen.reset()?;

    let mut display = hx8369::HX8369::new(SCREEN_WIDTH, SCREEN_HEIGHT);

    display.fill(Rgb565::BLACK);

    let mut maze = maze::Maze::new(MAZE_WIDTH, MAZE_HEIGHT);
    maze.generate(0, 0);

    let style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::YELLOW)
        .stroke_color(Rgb565::WHITE)
        .stroke_width(1)
        .build();

    let offset = Point {
        x: X_OFFSET as i32,
        y: Y_OFFSET as i32,
    };
    let cell_size = Size::new(CELL_SIZE as u32, CELL_SIZE as u32);

    let mut painter = MazePainter::new(maze, style, cell_size, offset);

    painter.draw(&mut display).ok();

    display.flush();

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::GREEN)
        .stroke_width(3)
        .build();

    loop {
        let touch = touch_screen.read_touch()?;
        if let Some(point) = touch {
            info!("state: {:?}", point);
            painter.on_click(point.x as i32, point.y as i32, style, &mut display);
            display.flush();
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
