use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
    video::WindowContext,
};

pub struct Skin<'a> {
    pub background_texture: Texture<'a>,

    pub controls_texture: Texture<'a>,
    pub controls_tiles_per_dimension: (usize, usize),
    pub controls_tile_size: (usize, usize),

    pub font_texture: Texture<'a>,
    pub font_tiles_per_dimension: (usize, usize),
    pub font_tile_size: (usize, usize),

    pub window_margin: usize,
    pub label_spacing: usize,
}

impl<'a> Skin<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, skin_name: &str) -> Result<Skin<'a>, String> {
        // TODO: sanitize paths to avoid path traversal vulns
        // TODO: check width, height mod
        let background_texture = texture_creator.load_texture(format!("skins/{}/background.png", skin_name))?;

        let controls_texture = texture_creator.load_texture(format!("skins/{}/controls.png", skin_name))?;
        let controls_tiles_per_dimension = (16, 16);
        let controls_tile_size = {
            let query = controls_texture.query();
            (
                query.width as usize / controls_tiles_per_dimension.0,
                query.height as usize / controls_tiles_per_dimension.1,
            )
        };

        let font_texture = texture_creator.load_texture(format!("skins/{}/font.png", skin_name))?;
        let font_tiles_per_dimension = (16, 8);
        let font_tile_size = {
            let query = font_texture.query();
            (
                query.width as usize / font_tiles_per_dimension.0,
                query.height as usize / font_tiles_per_dimension.1,
            )
        };

        let window_margin = 12;
        let label_spacing = 12;

        Ok(Skin {
            background_texture,

            controls_texture,
            controls_tiles_per_dimension,
            controls_tile_size,

            font_texture,
            font_tiles_per_dimension,
            font_tile_size,

            window_margin,
            label_spacing,
        })
    }
}
