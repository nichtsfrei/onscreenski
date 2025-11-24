use core::str;

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/"]
pub struct LayoutAssets;

impl LayoutAssets {
    pub fn default_layout_str() -> String {
        // Self-hosted assets must success
        let layout_file = LayoutAssets::get("default.toml").unwrap();
        String::from(str::from_utf8(layout_file.data.as_ref()).unwrap())
    }
}
