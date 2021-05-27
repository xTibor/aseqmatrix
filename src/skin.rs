use sdl2::{
    image::LoadTexture,
    render::{Texture, TextureCreator},
    video::WindowContext,
};

pub struct Skin<'a> {
    pub background_texture: Texture<'a>,
    pub foreground_texture: Texture<'a>,
}

impl<'a> Skin<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>, skin_name: &str) -> Result<Skin<'a>, String> {
        // TODO: sanitize paths to avoid path traversal vulns
        let background_texture = texture_creator.load_texture(format!("skins/{}/background.png", skin_name))?;
        let foreground_texture = texture_creator.load_texture(format!("skins/{}/foreground.png", skin_name))?;
        Ok(Skin {
            background_texture,
            foreground_texture,
        })
    }
}
