use bevy::prelude::*;

pub trait PlaceableItem:
    Sync + Send + core::fmt::Debug + core::hash::Hash + PartialEq + Eq + Clone
{
}

#[derive(Component)]
pub struct Placeable<T: PlaceableItem + ?Sized>(pub Box<T>);

#[derive(Component)]
pub struct Tileable;

#[derive(Component)]
pub struct Wall {
    pub health: usize,
}

#[derive(Bundle)]
pub struct WallBundle {}
