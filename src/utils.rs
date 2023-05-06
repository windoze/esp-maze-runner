use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::RgbColor,
};

pub trait Blend {
    fn blend<U>(&self, other: &U, alpha: f32) -> Self
    where
        U: Into<Rgb888> + Clone;
}

impl<T> Blend for T
where
    T: Into<Rgb888> + From<Rgb888> + Clone + Sized,
{
    fn blend<U>(&self, other: &U, alpha: f32) -> Self
    where
        U: Into<Rgb888> + Clone,
    {
        let s: Rgb888 = self.clone().into();
        let other: Rgb888 = other.clone().into();
        let r = (s.r() as f32 * alpha + other.r() as f32 * (1.0 - alpha)) as u8;
        let g = (s.g() as f32 * alpha + other.g() as f32 * (1.0 - alpha)) as u8;
        let b = (s.b() as f32 * alpha + other.b() as f32 * (1.0 - alpha)) as u8;

        Self::from(Rgb888::new(r, g, b))
    }
}
