use bevy::prelude::*;

use crate::util::ColliderMap;

pub const SPEED_TO_SCORE: f32 = 1. / 3000.;

pub const D100: Config = Config {
    size: Vec2::new(600., 150.),
    offset: Vec2::new(600. * 0.5, 150. * 0.35),
    init_speed: 360.,
    init_jump_power: 0.1,
    trex_running_start: 2,
    trex_running_end: 3,
    trex_ducking_start: 0,
    trex_ducking_end: 1,
    trex_sprite: SpriteConfig {
        size: UVec2::new(44, 47),
        offset: UVec2::new(848, 2),
        cols: 6,
        rows: 1,
    },
    trex_ducking_sprite: SpriteConfig {
        size: UVec2::new(59, 30),
        offset: UVec2::new(1112, 19),
        cols: 2,
        rows: 1,
    },
    ground_sprite: SpriteConfig {
        size: UVec2::new(300, 14),
        offset: UVec2::new(2, 53),
        cols: 4,
        rows: 1,
    },
    num_sprite: SpriteConfig {
        size: UVec2::new(10, 11),
        offset: UVec2::new(655, 2),
        cols: 10,
        rows: 1,
    },
    cacti_sm: SpriteConfig {
        size: UVec2::new(17, 35),
        offset: UVec2::new(228, 2),
        cols: 6,
        rows: 1,
    },
    cacti_md: SpriteConfig {
        size: UVec2::new(25, 50),
        offset: UVec2::new(332, 2),
        cols: 3,
        rows: 1,
    },
    cacti_lg: SpriteConfig {
        size: UVec2::new(75, 50),
        offset: UVec2::new(407, 2),
        cols: 1,
        rows: 1,
    },
};

#[derive(Debug, Resource)]
pub struct Config {
    // World
    pub size: Vec2,
    pub offset: Vec2,
    // Dynamics
    pub init_speed: f32,
    pub init_jump_power: f32,
    // Sprite indices
    pub trex_running_start: usize,
    pub trex_running_end: usize,
    pub trex_ducking_start: usize,
    pub trex_ducking_end: usize,
    // Sprites
    pub trex_sprite: SpriteConfig,
    pub trex_ducking_sprite: SpriteConfig,
    pub ground_sprite: SpriteConfig,
    pub num_sprite: SpriteConfig,
    pub cacti_sm: SpriteConfig,
    pub cacti_md: SpriteConfig,
    pub cacti_lg: SpriteConfig,
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteConfig {
    pub size: UVec2,
    pub offset: UVec2,
    pub cols: u32,
    pub rows: u32,
}

impl Into<TextureAtlasLayout> for SpriteConfig {
    fn into(self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(self.size, self.cols, self.rows, None, Some(self.offset))
    }
}

impl SpriteConfig {
    pub fn rect(self, index: u32) -> URect {
        let col = index % self.cols;
        let row = index / self.cols;
        let x = self.offset.x + self.size.x * col;
        let y = self.offset.y + self.size.y * row;
        URect::new(x, y, x + self.size.x, y + self.size.y)
    }

    pub fn add_colliders(&self, img: &Image, map: &mut ColliderMap) {
        for i in 0..(self.cols * self.rows) {
            let rect = self.rect(i);
            map.add(img, &rect, None);
        }
    }
}
