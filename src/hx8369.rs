use std::{convert::Infallible, ffi::c_int};

use embedded_graphics::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::{Dimensions, DrawTarget, Point, RgbColor, Size},
    primitives::Rectangle,
};

#[allow(non_camel_case_types)]
#[repr(C)]
struct esp_lcd_panel_t {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

type EspLcdPanelHandleT = *mut esp_lcd_panel_t;

#[allow(non_camel_case_types)]
type esp_err_t = i32;

#[allow(dead_code)]
extern "C" {
    fn hx8369_init() -> *mut esp_lcd_panel_t;

    fn esp_lcd_panel_reset(panel: EspLcdPanelHandleT) -> esp_err_t;

    fn esp_lcd_panel_init(panel: EspLcdPanelHandleT) -> esp_err_t;

    fn esp_lcd_panel_draw_bitmap(
        panel: *mut esp_lcd_panel_t,
        x_start: i32,
        y_start: i32,
        x_end: i32,
        y_end: i32,
        color_data: *const u8,
    ) -> esp_err_t;

    fn esp_lcd_panel_mirror(
        panel: EspLcdPanelHandleT,
        mirror_x: bool,
        mirror_y: bool,
    ) -> esp_err_t;

    fn esp_lcd_panel_swap_xy(panel: EspLcdPanelHandleT, swap_axes: bool) -> esp_err_t;

    fn esp_lcd_panel_set_gap(
        panel: EspLcdPanelHandleT,
        x_gap: c_int,
        y_gap: c_int,
    ) -> esp_err_t;

    fn esp_lcd_panel_invert_color(
        panel: EspLcdPanelHandleT,
        invert_color_data: bool,
    ) -> esp_err_t;

    fn esp_lcd_panel_disp_on_off(panel: EspLcdPanelHandleT, on_off: bool) -> esp_err_t;
}

pub struct HX8369 {
    handle: *mut esp_lcd_panel_t,

    width: usize,
    height: usize,
    buffer: Vec<RawU16>,
}

const LINES: usize = 60;

#[allow(dead_code)]
impl HX8369 {
    pub fn new(width: usize, height: usize) -> Self {
        let handle = unsafe { hx8369_init() };
        Self {
            handle,
            width,
            height,
            buffer: vec![Rgb565::BLACK.into(); width * height],
        }
    }

    pub fn draw_bitmap<T: Sized>(
        &self,
        x_start: i32,
        y_start: i32,
        x_end: i32,
        y_end: i32,
        color_data: &T,
    ) -> i32 {
        unsafe {
            esp_lcd_panel_draw_bitmap(
                self.handle,
                x_start,
                y_start,
                x_end,
                y_end,
                color_data as *const _ as *const u8,
            )
        }
    }

    pub fn reset(&self) {
        unsafe { esp_lcd_panel_reset(self.handle) };
    }

    pub fn mirror(&self, mirror_x: bool, mirror_y: bool) {
        unsafe { esp_lcd_panel_mirror(self.handle, mirror_x, mirror_y) };
    }

    pub fn swap_axes(&self, swap_axes: bool) {
        unsafe { esp_lcd_panel_swap_xy(self.handle, swap_axes) };
    }

    pub fn set_gap(&self, gap_x: i32, gap_y: i32) {
        unsafe { esp_lcd_panel_set_gap(self.handle, gap_x, gap_y) };
    }

    pub fn invert_color(&self, invert: bool) {
        unsafe { esp_lcd_panel_invert_color(self.handle, invert) };
    }

    pub fn flush(&self) {
        for i in 0..(self.height / LINES) {
            unsafe {
                esp_lcd_panel_draw_bitmap(
                    self.handle,
                    0,
                    (i * LINES) as i32,
                    self.width as i32,
                    (i * LINES + LINES) as i32,
                    (&self.buffer.as_slice()[i * LINES * self.width..(i + 1) * LINES * self.width])
                        as *const _ as *const u8,
                );
            }
        }
    }
}

impl Dimensions for HX8369 {
    fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
        Rectangle::new(
            Point::new(0, 0),
            Size::new(self.width as u32, self.height as u32),
        )
    }
}

impl DrawTarget for HX8369 {
    type Color = Rgb565;

    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for p in pixels {
            let x = p.0.x as usize;
            let y = p.0.y as usize;
            if x < self.width && y < self.height {
                self.buffer[y * self.width + x] = p.1.into();
            }
        }
        Ok(())
    }
}
