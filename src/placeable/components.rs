use bevy::prelude::*;

#[derive(Component)]
pub struct NowPlacing;

pub trait PlaceableItem: Sync + Send + ClonePlaceableItem {
    fn max_resources(&self) -> usize;
    fn current_resources(&self) -> usize;
    fn set_current_resources(&mut self, resources: usize);
    fn placeable_on_wall(&self) -> bool;
}

pub trait ClonePlaceableItem {
    fn clone_placeable_item(&self) -> Box<dyn PlaceableItem>;
}

impl<T> ClonePlaceableItem for T
where
    T: 'static + PlaceableItem + Clone,
{
    fn clone_placeable_item(&self) -> Box<dyn PlaceableItem> {
        Box::new(self.clone())
    }
}

/// auto create structs and impl PlaceableItem for them.
/// within struct body, define `placeable_on_wall` and fields.
/// # Example
/// ```
/// placeables!(
///     struct TestPlaceable {
///         placeable_on_wall: false,
///     },
///     struct TestPlaceable2 {
///         placeable_on_wall: true,
///         field1: usize,
///     }
/// );
/// ```
macro_rules! placeables {
    (
        $(
            struct $name: ident {
                placeable_on_wall: $placeable: expr,
                $(
                    $field: ident: $ty: ty
                ),* $(,)?
            }
        ),*
    ) => {
        $(
            #[derive(Debug, Copy, Clone)]
            pub struct $name {
                $(
                    pub $field: $ty,
                )*
                pub max_resources: usize,
                pub current_resources: usize,
            }

            impl PlaceableItem for $name {
                fn max_resources(&self) -> usize {
                    self.max_resources
                }

                fn current_resources(&self) -> usize {
                    self.current_resources
                }

                fn set_current_resources(&mut self, resources: usize) {
                    self.current_resources = resources;
                }

                fn placeable_on_wall(&self) -> bool {
                    $placeable
                }
            }
        )*
    };
}

#[derive(Component, Clone)]
pub struct Placeable<T: PlaceableItem + ?Sized>(pub Box<T>);

#[derive(Component)]
pub struct Tileable;

placeables! (
    struct Wall {
        placeable_on_wall: false,
    },
    struct Turret {
        placeable_on_wall: true,
    }
);

#[derive(Bundle, Clone)]
pub struct PlaceableBundle<T: PlaceableItem + ?Sized + 'static> {
    pub placeable: Placeable<T>,
    pub sprite_bundle: SpriteBundle,
}

impl<T> PlaceableBundle<T>
where
    T: PlaceableItem + ?Sized + 'static,
{
    pub fn clone_bundle_dyn(&self) -> PlaceableBundle<dyn PlaceableItem> {
        PlaceableBundle {
            placeable: Placeable(self.placeable.0.clone_placeable_item()),
            sprite_bundle: self.sprite_bundle.clone(),
        }
    }
}
