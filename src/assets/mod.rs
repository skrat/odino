use bevy::{asset::embedded_asset, prelude::*};

pub struct EmbeddedAssetsPlugin;

impl Plugin for EmbeddedAssetsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "textures/100-offline-sprite.png");
        embedded_asset!(app, "sounds/button-press.mp3");
    }
}

pub const SPRITE_PATH: &str = "embedded://odino/assets/textures/100-offline-sprite.png";

pub const BUTTON_PRESS_PATH: &str = "embedded://odino/assets/sounds/button-press.mp3";
