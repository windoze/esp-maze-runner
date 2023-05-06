use std::{
    cmp::{max, min},
    ffi::c_int,
};

use std::convert::Infallible;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Dimensions, DrawTarget, Point, Size},
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

    fn esp_lcd_panel_mirror(panel: EspLcdPanelHandleT, mirror_x: bool, mirror_y: bool)
        -> esp_err_t;

    fn esp_lcd_panel_swap_xy(panel: EspLcdPanelHandleT, swap_axes: bool) -> esp_err_t;

    fn esp_lcd_panel_set_gap(panel: EspLcdPanelHandleT, x_gap: c_int, y_gap: c_int) -> esp_err_t;

    fn esp_lcd_panel_invert_color(panel: EspLcdPanelHandleT, invert_color_data: bool) -> esp_err_t;

    fn esp_lcd_panel_disp_on_off(panel: EspLcdPanelHandleT, on_off: bool) -> esp_err_t;
}

pub struct HX8369 {
    handle: *mut esp_lcd_panel_t,

    width: usize,
    height: usize,
    buffer: Vec<u16>,

    min_y: usize,
    max_y: usize,
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
            buffer: vec![0; width * height],

            min_y: height,
            max_y: 0,
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

    pub fn get_width(&self) -> u32 {
        self.width as u32
    }

    pub fn get_height(&self) -> u32 {
        self.height as u32
    }

    pub fn invalidate(&mut self) {
        self.min_y = 0;
        self.max_y = self.height;
    }

    pub fn flush(&mut self) {
        // Nothing to flush
        if self.min_y > self.max_y {
            // Nothing to flush
            return;
        }
        // HX8369 can only send ~100K bytes at once, about 800x62 pixels in RGB565 format
        // so we need to split the buffer into chunks, LINES is set to 60 as we have screen height at 480
        // Flush in chunks of LINES
        for i in (self.min_y..self.max_y).step_by(LINES) {
            unsafe {
                // Don't exceed screen bounds, as well as max_y
                let y_end = min(min(i + LINES, self.height), self.max_y + 1);
                // Swap start and end if needed
                let y_start = min(i, y_end);
                let y_end = max(i, y_end);
                let y_end = min(y_end, self.height);
                // Skip if nothing to flush
                if y_start >= y_end {
                    continue;
                }
                esp_lcd_panel_draw_bitmap(
                    self.handle,
                    0,
                    y_start as i32,
                    self.width as i32,
                    y_end as i32,
                    (&self.buffer.as_slice()[y_start * self.width..y_end * self.width]) as *const _
                        as *const u8,
                );
            }
        }
        self.min_y = self.height;
        self.max_y = 0;
    }

    pub fn get_raw_buffer_mut<T>(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.buffer.as_mut_ptr() as *mut T,
                self.buffer.len() * core::mem::size_of::<u16>() / core::mem::size_of::<T>(),
            )
        }
    }

    pub fn get_raw_buffer<T>(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.buffer.as_ptr() as *mut T,
                self.buffer.len() * core::mem::size_of::<u16>() / core::mem::size_of::<T>(),
            )
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Rgb565 {
        if x >= self.width || y >= self.height {
            return <Rgb565 as embedded_graphics::prelude::RgbColor>::BLACK;
        }
        let w = self.width;
        self.get_raw_buffer()[y * w + x]
    }

    pub fn fill(&mut self, color: Rgb565) {
        self.min_y = 0;
        self.max_y = self.height;
        self.get_raw_buffer_mut()
            .iter_mut()
            .for_each(|p| *p = color);
        self.flush();
    }
}

pub trait DrawCanvas {
    type PixelColor: Clone
        + Copy
        + embedded_graphics::prelude::RgbColor
        + From<embedded_graphics::pixelcolor::Rgb888>
        + Into<embedded_graphics::pixelcolor::Rgb888>;
    fn get_pixel(&self, x: usize, y: usize) -> Self::PixelColor;
}

impl DrawCanvas for HX8369 {
    type PixelColor = Rgb565;
    fn get_pixel(&self, x: usize, y: usize) -> Self::PixelColor {
        self.get_pixel(x, y)
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
        // Record min and max dirty lines, the range in the middle will be flushed
        // This is an optimization as the data transfer seems not to be fast enough, it takes around 4ms to flush the whole screen
        for p in pixels {
            if p.0.x < 0 || p.0.y < 0 {
                continue;
            }
            let x = p.0.x as usize;
            let y = p.0.y as usize;
            if x >= self.width || y >= self.height {
                continue;
            }
            if self.min_y > y {
                self.min_y = y;
            }
            if self.max_y < y {
                self.max_y = y;
            }
            if x < self.width && y < self.height {
                let w = self.width;
                self.get_raw_buffer_mut()[y * w + x] = p.1;
            }
        }
        Ok(())
    }
}
