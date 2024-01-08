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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Converts a GridPos to a bevy Vec2
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }

    /// Converts a tile position to a grid position
    pub fn from_tile_pos_vec(tile_pos: Vec2) -> Self {
        Self {
            x: tile_pos.x as i32,
            y: tile_pos.y as i32,
        }
    }

    /// Converts a world position to a grid position
    pub fn from_world_pos_vec(world_pos: Vec2) -> Self {
        let tile_pos_vec = world_pos.world_pos_to_tile();
        Self::from_tile_pos_vec(tile_pos_vec)
    }

    /// Returns the length of the vector
    pub fn length(&self) -> f32 {
        ((self.x.pow(2) + self.y.pow(2)) as f32).sqrt()
    }

    /// Normalizes the vector and rounds the values to the nearest integer
    pub fn normalize_int(&self) -> Self {
        let length = self.length();

        Self {
            x: (self.x as f32 / length).round() as i32,
            y: (self.y as f32 / length).round() as i32,
        }
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

/// Check if an enum variant is equal to another enum variant
/// ignoring their values
pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod varient_eq {
        use super::*;
        #[test]
        fn test_variant_eq_same_variant() {
            let a = Some(5);
            let b = Some(10);
            assert!(variant_eq(&a, &b));
        }

        #[test]
        fn test_variant_eq_different_variant() {
            let a = Some(5);
            let b = None;
            assert!(!variant_eq(&a, &b));
        }

        #[test]
        fn test_variant_eq_same_value_different_variant() {
            let a = Some(5);
            let b = Some(5);
            assert!(variant_eq(&a, &b));
        }
    }

    #[cfg(test)]
    mod grid_pos {
        use super::*;
        #[test]
        fn test_new() {
            let pos = GridPos::new(3, 4);
            assert_eq!(pos.x, 3);
            assert_eq!(pos.y, 4);
        }

        #[test]
        fn test_to_vec2() {
            let pos = GridPos::new(2, 5);
            let vec2 = pos.to_vec2();
            assert_eq!(vec2.x, 2.0);
            assert_eq!(vec2.y, 5.0);
        }

        #[test]
        fn test_from_tile_pos_vec() {
            let tile_pos = Vec2::new(1.5, 2.5);
            let pos = GridPos::from_tile_pos_vec(tile_pos);
            assert_eq!(pos.x, 1);
            assert_eq!(pos.y, 2);
        }

        #[test]
        fn test_from_world_pos_vec() {
            let world_pos = Vec2::new(3.5, 4.5) * TILE_SIZE;
            let pos = GridPos::from_world_pos_vec(world_pos);
            assert_eq!(pos.x, 3);
            assert_eq!(pos.y, 4);
        }

        #[test]
        fn test_length() {
            let pos = GridPos::new(3, 4);
            assert_eq!(pos.length(), 5.0);
        }

        #[test]
        fn test_normalize_int() {
            let pos = GridPos::new(3, 4);
            let normalized = pos.normalize_int();
            assert_eq!(normalized.x, 1);
            assert_eq!(normalized.y, 1);

            let pos = GridPos::new(2, 10);
            let normalized = pos.normalize_int();
            assert_eq!(normalized.x, 0);
            assert_eq!(normalized.y, 1);
        }

        #[test]
        fn test_add() {
            let pos1 = GridPos::new(2, 3);
            let pos2 = GridPos::new(4, 5);
            let result = pos1 + pos2;
            assert_eq!(result.x, 6);
            assert_eq!(result.y, 8);
        }

        #[test]
        fn test_sub() {
            let pos1 = GridPos::new(5, 7);
            let pos2 = GridPos::new(2, 3);
            let result = pos1 - pos2;
            assert_eq!(result.x, 3);
            assert_eq!(result.y, 4);
        }

        #[test]
        fn test_add_assign() {
            let mut pos = GridPos::new(2, 3);
            let other = GridPos::new(4, 5);
            pos += other;
            assert_eq!(pos.x, 6);
            assert_eq!(pos.y, 8);
        }

        #[test]
        fn test_sub_assign() {
            let mut pos = GridPos::new(5, 7);
            let other = GridPos::new(2, 3);
            pos -= other;
            assert_eq!(pos.x, 3);
            assert_eq!(pos.y, 4);
        }
    }
}
