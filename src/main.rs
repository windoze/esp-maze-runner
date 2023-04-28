mod gt911;
mod hx8369;

#[cfg(feature = "eg")]
mod utils;

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
        image::Image,
        pixelcolor::Rgb565,
        prelude::{Point, RgbColor, Size},
        primitives::{Primitive, PrimitiveStyleBuilder, Rectangle},
        transform::Transform,
        Drawable,
    };
    use rand::{seq::SliceRandom, Rng};
    #[cfg(feature = "ttf")]
    use rusttype::{point, Font, Scale};
    use tinybmp::Bmp;

    use std::{thread, time::Duration};


    let mut display = hx8369::HX8369::new(800, 480);

    display.fill(Rgb565::BLACK);

    const COLORS: [Rgb565; 8] = [
        Rgb565::RED,
        Rgb565::GREEN,
        Rgb565::BLUE,
        Rgb565::CYAN,
        Rgb565::MAGENTA,
        Rgb565::YELLOW,
        Rgb565::WHITE,
        Rgb565::BLACK,
    ];

    let bmp_data = include_bytes!("../assets/test.bmp");
    let bmp = Bmp::<Rgb565>::from_slice(bmp_data).unwrap();
    Image::new(&bmp, Point::new(40, 0))
        .draw(&mut display)
        .unwrap();

    #[cfg(feature = "ttf")]
    {
        use crate::{gt911::TouchState, utils::Blend};
        use embedded_graphics::prelude::DrawTarget;
        use embedded_graphics::Pixel;
        use std::iter::once;
        let font_data = include_bytes!("../assets/wqy-microhei.ttc");
        let font =
            Font::try_from_bytes(font_data as &[u8]).expect("error constructing a Font from bytes");
    
        // Desired font pixel height
        let height: f32 = 90.0;
        let pixel_height = height.ceil() as usize;
    
        let scale = Scale {
            x: height * 3.0,
            y: height * 3.0,
        };
    
        // The origin of a line of text is at the baseline (roughly where
        // non-descending letters sit). We don't want to clip the text, so we shift
        // it down with an offset when laying it out. v_metrics.ascent is the
        // distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent);
    
        // Glyphs to draw for "RustType". Feel free to try other strings.
        let glyphs: Vec<_> = font.layout("晴转多云，气温9-12°C", scale, offset).collect();
    
        for g in glyphs {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|x, y, v| {
                    let x = x as i32 + bb.min.x;
                    let y = y as i32 + bb.min.y;
                    if v < 0.5 {
                        return;
                    }
                    let background = display.get_pixel(x as usize, y as usize);
                    let foreground = Rgb565::new(255, 255, 255).blend(&background, v/2.0);
                    // let foreground = background.blend(&Rgb565::WHITE, v);
                    display
                        .draw_iter(once(Pixel::<Rgb565>(Point { x, y }, foreground)))
                        .unwrap();
                })
            }
        }
    }

    display.flush();

    loop {
        if let Some(state) = gt911::read_touch() {
            info!("state: {:?}", state);
            let style = PrimitiveStyleBuilder::new()
                .stroke_color(*COLORS.choose(&mut rand::thread_rng()).unwrap())
                .stroke_width(3)
                .fill_color(*COLORS.choose(&mut rand::thread_rng()).unwrap())
                .build();
            let size: i32 = rand::thread_rng().gen_range(50..150);
            if state.pressed {
                Rectangle::new(
                    Point::new(state.x as i32, state.y as i32),
                    Size::new(size as u32, size as u32),
                )
                .translate(Point::new(-(size / 2), -(size / 2)))
                .into_styled(style)
                .draw(&mut display)
                .unwrap();
                display.flush();
            }
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
