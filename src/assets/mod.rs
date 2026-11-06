use bevy::{asset::embedded_asset, prelude::*};

pub struct EmbeddedAssetsPlugin;

impl Plugin for EmbeddedAssetsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "textures/100-offline-sprite.png");
        embedded_asset!(app, "sounds/button-press.mp3");
        app.init_state::<SpriteState>();
        app.init_resource::<SpriteHandle>();
        app.add_systems(
            Update,
            check_sprite_loaded.run_if(in_state(SpriteState::Init)),
        );
        #[cfg(feature = "dev")]
        app.add_systems(
            Update,
            bevy::dev_tools::states::log_transitions::<SpriteState>,
        );
    }
}

pub const SPRITE_PATH: &str = "embedded://odino/assets/textures/100-offline-sprite.png";

pub const BUTTON_PRESS_PATH: &str = "embedded://odino/assets/sounds/button-press.mp3";

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum SpriteState {
    #[default]
    Init,
    Loaded,
}

#[derive(Resource, Deref)]
pub struct SpriteHandle(Handle<Image>);

impl FromWorld for SpriteHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        SpriteHandle(asset_server.load(SPRITE_PATH))
    }
}

fn check_sprite_loaded(
    image: Res<SpriteHandle>,
    mut rdr: MessageReader<AssetEvent<Image>>,
    mut next_state: ResMut<NextState<SpriteState>>,
) {
    for evt in rdr.read() {
        if evt.is_loaded_with_dependencies(image.id()) {
            next_state.set(SpriteState::Loaded);
        }
    }
}
