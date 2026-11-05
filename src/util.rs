use std::collections::HashMap;

use avian2d::{collision::collider::Collider, math::RVector};
use bevy::prelude::*;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct ColliderMap(HashMap<AssetId<Image>, Collider>);

impl ColliderMap {
    pub fn add(
        &mut self,
        id: AssetId<Image>,
        img: &Image,
        rect: &URect,
        threshold: Option<f32>,
    ) -> Collider {
        let points = points_for_convex_hull(img, rect, threshold);
        let collider = Collider::convex_hull(&points).unwrap();
        self.insert(id, collider.clone());
        collider
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
