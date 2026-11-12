use std::time::Duration;

use bevy::prelude::*;

use crate::{
    assets::SpriteHandle,
    config::{CLOUD_LAYER, Config},
    state::GameState,
};

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CloudTimer(Timer::from_seconds(0.0, TimerMode::Once)))
            .init_resource::<SkyHandles>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_clouds.run_if(in_state(GameState::Running)),
                    animate_clouds.run_if(in_state(GameState::Running)),
                ),
            );
    }
}

#[derive(Resource, Deref, DerefMut)]
struct CloudTimer(Timer);

#[derive(Resource)]
struct SkyHandles {
    cloud: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
struct Cloud;

impl FromWorld for SkyHandles {
    fn from_world(world: &mut World) -> Self {
        let cfg = world.resource::<Config>();
        let cloud: TextureAtlasLayout = cfg.cloud.into();
        let mut atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        SkyHandles {
            cloud: atlas_layouts.add(cloud),
        }
    }
}

fn spawn_clouds(
    t: Res<Time<Fixed>>,
    mut timer: ResMut<CloudTimer>,
    cfg: Res<Config>,
    image: Res<SpriteHandle>,
    layout: Res<SkyHandles>,
    mut commands: Commands,
) {
    timer.tick(t.delta());
    if timer.just_finished() {
        timer.set_duration(Duration::from_secs(rand::random_range(4..8)));
        timer.reset();
        commands.spawn((
            Sprite::from_atlas_image(
                image.clone(),
                TextureAtlas {
                    layout: layout.cloud.clone(),
                    index: 0,
                },
            ),
            Transform::from_xyz(
                cfg.size.x + cfg.cloud.size.x as f32,
                cfg.size.y * rand::random_range(0.4..0.7),
                CLOUD_LAYER,
            ),
            Cloud,
        ));
    }
}

fn animate_clouds(
    t: Res<Time<Fixed>>,
    cfg: Res<Config>,
    query: Query<(Entity, &mut Transform), With<Cloud>>,
    mut commands: Commands,
) {
    let d = t.delta_secs() * 50.;
    for (ent, mut tf) in query {
        tf.translation.x -= d;
        if tf.translation.x < -(cfg.cloud.size.x as f32) {
            commands.entity(ent).despawn();
        }
    }
}
