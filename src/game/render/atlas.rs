use std::collections::HashMap;

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) enum TextureType {
    Error,
    Air,
    Stone,
    Dirt,
    GrassSide,
    GrassTop,
}

pub(crate) struct TextureCoordinates {
    u_min: f32,
    u_max: f32,
    v_min: f32,
    v_max: f32,
}

impl TextureCoordinates {
    pub(crate) fn new(u_min: f32, u_max: f32, v_min: f32, v_max: f32) -> Self {
        Self {
            u_min,
            u_max,
            v_min,
            v_max,
        }
    }

    pub(crate) fn get(&self) -> (f32, f32, f32, f32) {
        (self.u_min, self.u_max, self.v_min, self.v_max)
    }
}

pub(crate) struct TextureAtlas {
    image: image::DynamicImage,
    coordinates: HashMap<TextureType, TextureCoordinates>,
}

impl TextureAtlas {
    pub(crate) fn init() -> Self {
        let textures = vec![
            (TextureType::Air, None),
            (
                TextureType::GrassTop,
                Some(include_bytes!("../../../assets/textures/voxels/grass_top.png").as_slice()),
            ),
            (
                TextureType::GrassSide,
                Some(include_bytes!("../../../assets/textures/voxels/grass_side.png").as_slice()),
            ),
            (
                TextureType::Dirt,
                Some(include_bytes!("../../../assets/textures/voxels/dirt.png").as_slice()),
            ),
            (
                TextureType::Stone,
                Some(include_bytes!("../../../assets/textures/voxels/stone.png").as_slice()),
            ),
        ];

        Self::build(textures)
    }

    fn build(textures: Vec<(TextureType, Option<&[u8]>)>) -> Self {
        let mut loaded_textures = Vec::new();
        let mut texture_size = 0u32;

        for (texture_type, bytes) in textures {
            if let Some(bytes) = bytes {
                let image = image::load_from_memory(bytes)
                    .expect("Failed to load texture")
                    .to_rgba8();

                if texture_size == 0 {
                    texture_size = image.width().min(image.height());
                }

                loaded_textures.push((texture_type, image));
            }
        }

        if texture_size == 0 {
            texture_size = 16;
        }

        let texture_count = loaded_textures.len() + 1;
        let textures_per_row = (texture_count as f32).sqrt().ceil() as u32;
        let atlas_width = textures_per_row * texture_size;
        let atlas_height = textures_per_row * texture_size;

        let mut atlas = image::RgbaImage::new(atlas_width, atlas_height);

        let mut coordinates = HashMap::new();

        let error_texture = Self::create_error_texture(texture_size);
        Self::copy_texture_to_atlas(&mut atlas, &error_texture, 0, 0, texture_size);
        coordinates.insert(
            TextureType::Error,
            Self::calculate_coordinates(0, 0, texture_size, atlas_width, atlas_height),
        );

        for (index, (texture_type, texture)) in loaded_textures.iter().enumerate() {
            let index = index + 1;
            let x = (index as u32 % textures_per_row) * texture_size;
            let y = (index as u32 / textures_per_row) * texture_size;

            Self::copy_texture_to_atlas(&mut atlas, texture, x, y, texture_size);
            coordinates.insert(
                *texture_type,
                Self::calculate_coordinates(x, y, texture_size, atlas_width, atlas_height),
            );
        }

        Self {
            image: image::DynamicImage::ImageRgba8(atlas),
            coordinates,
        }
    }

    pub(crate) fn image(&self) -> image::DynamicImage {
        self.image.clone()
    }

    pub(crate) fn get_coordinates(&self, texture: TextureType) -> &TextureCoordinates {
        self.coordinates
            .get(&texture)
            .expect("Should not request coordinates for a texture that is not in the atlas")
    }

    fn create_error_texture(size: u32) -> image::RgbaImage {
        let mut texture = image::RgbaImage::new(size, size);
        let checker_size = size / 2;

        let black = image::Rgba([0, 0, 0, 255]);
        let magenta = image::Rgba([255, 0, 255, 255]);

        for y in 0..size {
            for x in 0..size {
                let is_top = y < checker_size;
                let is_left = x < checker_size;

                let color = if is_top == is_left { black } else { magenta };
                texture.put_pixel(x, y, color);
            }
        }

        texture
    }

    fn copy_texture_to_atlas(
        atlas: &mut image::RgbaImage,
        texture: &image::RgbaImage,
        x: u32,
        y: u32,
        size: u32,
    ) {
        for texture_y in 0..size.min(texture.height()) {
            for texture_x in 0..size.min(texture.width()) {
                let pixel = texture.get_pixel(texture_x, texture_y);
                atlas.put_pixel(x + texture_x, y + texture_y, *pixel);
            }
        }
    }

    fn calculate_coordinates(
        x: u32,
        y: u32,
        size: u32,
        atlas_width: u32,
        atlas_height: u32,
    ) -> TextureCoordinates {
        let u_min = x as f32 / atlas_width as f32;
        let u_max = (x + size) as f32 / atlas_width as f32;
        let v_min = y as f32 / atlas_height as f32;
        let v_max = (y + size) as f32 / atlas_height as f32;

        TextureCoordinates::new(u_min, u_max, v_min, v_max)
    }
}
