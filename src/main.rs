mod gt911;
mod hx8369;

use esp_idf_sys as _;
use log::info;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    unsafe {
        gt911::GT911_RST();
        gt911::gt911_init(gt911::GT911_I2C_SLAVE_ADDR);
    };

    info!("Running demo");

    #[cfg(feature = "sl")]
    slint_demo();

    #[cfg(feature = "eg")]
    eg_demo();
}

#[cfg(feature = "sl")]
mod esp_lcd_backend;

#[cfg(feature = "sl")]
slint::include_modules!();

#[cfg(feature = "sl")]
fn slint_demo() {
    use esp_lcd_backend::EspBackend;

    slint::platform::set_platform(Box::<EspBackend>::default())
        .expect("backend already initialized");

    let main_window = MainWindow::new().unwrap();
    main_window.on_ok_button_clicked(|| {
        info!("OK button clicked");
    });

    main_window.run().unwrap();

    panic!("The MCU demo should not quit")
}

#[cfg(feature = "eg")]
fn eg_demo() {
    use embedded_graphics::{
        pixelcolor::Rgb565,
        prelude::{Point, RgbColor, Size},
        primitives::{Line, Primitive, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
        transform::Transform,
        Drawable,
    };

    use std::{thread, time::Duration};

    let mut display = hx8369::HX8369::new(800, 480);

    Line::new(Point::new(50, 20), Point::new(600, 350))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 1))
        .draw(&mut display)
        .unwrap();

    // Green 10 pixel wide line with translation applied
    Line::new(Point::new(50, 20), Point::new(600, 350))
        .translate(Point::new(-30, 10))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 10))
        .draw(&mut display)
        .unwrap();

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::RED)
        .stroke_width(3)
        .fill_color(Rgb565::GREEN)
        .build();

    Rectangle::new(Point::new(30, 200), Size::new(100, 150))
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    // Rectangle with translation applied
    Rectangle::new(Point::new(300, 20), Size::new(100, 150))
        .translate(Point::new(-20, -10))
        .into_styled(style)
        .draw(&mut display)
        .unwrap();

    display.flush();
    loop {
        if let Some(state) = gt911::read_touch() {
            info!("x: {}, y: {}, state: {:?}", state.x, state.y, state.pressed);
            if state.pressed {
                Rectangle::new(
                    Point::new(state.x as i32, state.y as i32),
                    Size::new(100, 100),
                )
                .translate(Point::new(-50, -50))
                .into_styled(style)
                .draw(&mut display)
                .unwrap();
                display.flush();
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
}
