use bevy::prelude::*;

pub trait PlaceableItem: Sync + Send {
    fn max_resources(&self) -> usize;
    fn current_resources(&self) -> usize;
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
