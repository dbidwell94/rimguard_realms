use crate::TILE_SIZE;
use bevy::prelude::*;

pub trait TranslationHelper {
    fn world_pos_to_tile(&self) -> Vec2;
    fn tile_pos_to_world(&self) -> Vec2;
}

impl TranslationHelper for Transform {
    fn world_pos_to_tile(&self) -> Vec2 {
        Vec2::new(
            (self.translation.x / TILE_SIZE).floor(),
            (self.translation.y / TILE_SIZE).floor(),
        )
    }

    fn tile_pos_to_world(&self) -> Vec2 {
        Vec2::new(
            self.translation.x * TILE_SIZE - TILE_SIZE / 2.,
            self.translation.y * TILE_SIZE - TILE_SIZE / 2.,
        )
    }
}

impl TranslationHelper for GlobalTransform {
    fn world_pos_to_tile(&self) -> Vec2 {
        Vec2::new(
            (self.translation().x / TILE_SIZE).floor(),
            (self.translation().y / TILE_SIZE).floor(),
        )
    }

    fn tile_pos_to_world(&self) -> Vec2 {
        Vec2::new(
            self.translation().x * TILE_SIZE - TILE_SIZE / 2.,
            self.translation().y * TILE_SIZE - TILE_SIZE / 2.,
        )
    }
}

impl TranslationHelper for Vec3 {
    fn world_pos_to_tile(&self) -> Vec2 {
        Vec2::new((self.x / TILE_SIZE).floor(), (self.y / TILE_SIZE).floor())
    }

    fn tile_pos_to_world(&self) -> Vec2 {
        Vec2::new(
            self.x * TILE_SIZE - TILE_SIZE / 2.,
            self.y * TILE_SIZE - TILE_SIZE / 2.,
        )
    }
}

impl TranslationHelper for Vec2 {
    fn world_pos_to_tile(&self) -> Vec2 {
        Vec2::new((self.x / TILE_SIZE).floor(), (self.y / TILE_SIZE).floor())
    }

    fn tile_pos_to_world(&self) -> Vec2 {
        Vec2::new(
            self.x * TILE_SIZE + TILE_SIZE / 2.,
            self.y * TILE_SIZE + TILE_SIZE / 2.,
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    pub fn from_tile_pos_vec(tile_pos: Vec2) -> Self {
        Self {
            x: tile_pos.x as i32,
            y: tile_pos.y as i32,
        }
    }

    pub fn from_world_pos_vec(world_pos: Vec2) -> Self {
        let tile_pos_vec = world_pos.world_pos_to_tile();
        Self::from_tile_pos_vec(tile_pos_vec)
    }
}

impl std::ops::Add for GridPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x as i32,
            y: self.y + rhs.y as i32,
        }
    }
}

impl std::ops::Sub for GridPos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x as i32,
            y: self.y - rhs.y as i32,
        }
    }
}

impl std::ops::AddAssign for GridPos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x as i32;
        self.y += rhs.y as i32;
    }
}

impl std::ops::SubAssign for GridPos {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x as i32;
        self.y -= rhs.y as i32;
    }
}

#[macro_export]
macro_rules! boxed {
    ($expr: expr) => {
        Box::new($expr)
    };
}

/// Check if an enum variant is equal to another enum variant
/// ignoring their values
pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
