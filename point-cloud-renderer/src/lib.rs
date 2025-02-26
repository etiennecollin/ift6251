use image::{ImageBuffer, Rgba};

pub mod camera;
pub mod loader;
pub mod point;
pub mod render;
pub mod scene;
pub mod screen;

type ColorDepth = u8;
pub type PixelType = Rgba<ColorDepth>;
pub type ImageType = ImageBuffer<PixelType, Vec<ColorDepth>>;
