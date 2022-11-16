use crate::prelude::*;

type Image = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

pub struct Texture {
    image: Image,
}

impl Texture {
    pub fn from_file(file_path: &str) -> Result<Self> {
        let tex = image::open(file_path)?;
        let image = tex.to_rgba8();
        Ok(Self { image })
    }

    pub fn color(&self, tex_coord: Vec2) -> Color {
        let width = self.image.width() as f32;
        let height = self.image.height() as f32;

        let u = tex_coord.x;
        let v = tex_coord.y;

        let x = (width * u) as u32;
        let y = (height - (height * v)) as u32;
        let pixel = self.image.get_pixel(x, y);

        (pixel[0], pixel[1], pixel[2])
    }
}
