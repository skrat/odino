use std::collections::HashMap;

use avian2d::{collision::collider::Collider, math::RVector};
use bevy::prelude::*;

use crate::config::Config;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ColliderMap(HashMap<URect, Collider>);

impl ColliderMap {
    pub fn add(&mut self, img: &Image, rect: &URect, threshold: Option<f32>) -> Collider {
        let points = points_for_convex_hull(img, rect, threshold);
        let collider = Collider::convex_hull(&points).unwrap();
        self.insert(*rect, collider.clone());
        collider
    }

    pub fn get_collider(&self, rect: &URect) -> Collider {
        self.get(rect)
            .and_then(|x| Some(x.clone()))
            .unwrap_or_else(|| {
                let size = rect.size();
                Collider::rectangle(size.x as f32, size.y as f32)
            })
    }
}

pub fn points_for_convex_hull(img: &Image, rect: &URect, threshold: Option<f32>) -> Vec<RVector> {
    let t = threshold.unwrap_or(0.5);
    let half = (rect.max - rect.min).as_vec2() / 2.;
    let mut points: Vec<RVector> = Vec::new();
    for y in rect.min.y..rect.max.y {
        for x in rect.min.x..rect.max.x {
            if img.get_color_at(x, y).unwrap().alpha() > t {
                points.push(RVector::new(
                    (x - rect.min.x) as f32 - half.x,
                    -((y - rect.min.y) as f32 - half.y),
                ));
            }
        }
    }
    points
}

pub fn add_layout<F>(world: &mut World, from: F) -> Handle<TextureAtlasLayout>
where
    F: Fn(&Config) -> TextureAtlasLayout,
{
    let layout = from(world.resource::<Config>());
    let mut atlas_layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
    atlas_layouts.add(layout)
}
