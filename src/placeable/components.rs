use bevy::prelude::*;

#[derive(Component)]
pub struct NowPlacing;

pub trait PlaceableItem: Sync + Send + ClonePlaceableItem {
    fn max_resources(&self) -> usize;
    fn current_resources(&self) -> usize;
    fn set_current_resources(&mut self, resources: usize);
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

macro_rules! placeables {
    (
        $(
            struct $name: ident {
                $(
                    $field: ident: $ty: ty
                ),* $(,)?
            }
        ),*
    ) => {
        $(
            #[derive(Component, Debug, Copy, Clone)]
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
            }


        )*
    };
}

#[derive(Component, Clone)]
pub struct Placeable<T: PlaceableItem + ?Sized>(pub Box<T>);

#[derive(Component)]
pub struct Tileable;

placeables!(
    struct Wall {},
    struct Turret {}
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
