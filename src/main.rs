use std::{thread, time::Duration};

use esp_idf_svc::hal::{
    delay::Ets,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    units::FromValueType,
};
use gt911::GT911Builder;
use hx8369::{Backend, DisplayWrapper};
use slint::PhysicalSize;
use slint::{platform::software_renderer as renderer, SharedPixelBuffer};
use slint::{
    platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel},
    Rgb8Pixel,
};
use zune_jpeg::JpegDecoder;
mod gt911;
mod hx8369;
// mod maze;
// mod maze_painter;

const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 480;

const WALLPAPER: &[u8] = include_bytes!("../ui/assets/background.jpg");

slint::include_modules!();

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
    let window = MinimalSoftwareWindow::new(renderer::RepaintBufferType::ReusedBuffer);
    slint::platform::set_platform(Box::new(Backend::new(window.clone()))).unwrap();
    window.set_size(PhysicalSize::new(800, 480));

    let ui = AppWindow::new().unwrap();
    let _handle = ui.as_weak();

    let mut line_buffer = [Rgb565Pixel(0); SCREEN_WIDTH];
    let mut wrapper = DisplayWrapper {
        display: &mut display,
        line_buffer: &mut line_buffer,
    };

    let mut decoder = JpegDecoder::new(WALLPAPER);
    let pixels = decoder
        .decode()
        .inspect_err(|e| {
            log::error!("Error decoding image: {:?}", e);
        })
        .unwrap();
    let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        pixels.as_slice(),
        decoder.info().unwrap().width as u32,
        decoder.info().unwrap().height as u32,
    );
    let image = slint::Image::from_rgb8(buffer);

    let mut flag = false;
    loop {
        slint::platform::update_timers_and_animations();

        // Draw the scene if something needs to be drawn.
        window.draw_if_needed(|renderer| {
            renderer.render_by_line(&mut wrapper);
        });

        if !window.has_active_animations() {
            // if no animation is running, wait for the next input event
        }
        if !flag {
            flag = true;
            ui.global::<AppData>().set_background(image.clone());
        }
    }
}
