use serde::{Deserialize, Serialize};

use crate::ui::supported_keys::SupportedKeys;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyDefinition {
    pub top_legend: Option<String>,
    pub bottom_legend: Option<String>,
    pub scan_code: SupportedKeys,
    pub width: Option<f32>,
}

impl KeyDefinition {}

pub type Layout = Vec<Vec<KeyDefinition>>;
pub type Layer = (Layout, Layout);

#[derive(Serialize, Deserialize, Debug)]
pub struct LayoutDefinition {
    pub layer: Vec<Layer>,
}

impl LayoutDefinition {
    pub fn from_toml(toml_str: &str) -> Self {
        toml::from_str::<LayoutDefinition>(toml_str).unwrap()
    }
}
