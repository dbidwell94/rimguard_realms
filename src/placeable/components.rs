use bevy::prelude::*;

#[derive(Component)]
pub struct NowPlacing;

#[derive(Component)]
pub struct TempPlaceholder;

#[derive(Component)]
/// Used to mark an entity as a built entity. When it is built, navmesh should be updated
/// to account for the new entity.
pub struct Built;

/// auto create structs and impl PlaceableItem for them.
/// within struct body, define `placeable_on_wall`, `tileable` and fields.
/// # Example
/// ```
/// placeables!(
///     struct TestPlaceable {
///         placeable_on_wall: false,
///         tileable: true,
///     },
///     struct TestPlaceable2 {
///         placeable_on_wall: true,
///         tileable: false,
///         field1: usize,
///     }
/// );
/// ```
macro_rules! placeables {
    (
        $(
            struct $name: ident {
                placeable_on_wall: $placeable: expr,
                tileable: $tileable: expr,
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
                pub placeable_on_wall: bool,
                pub tileable: bool,
                pub max_resources: usize,
                pub current_resources: usize,
                pub built: bool,
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $(
                            $field: Default::default(),
                        )*
                        placeable_on_wall: $placeable,
                        tileable: $tileable,
                        max_resources: 0,
                        current_resources: 0,
                        built: false
                    }
                }
            }
        )*

        pub trait PlaceableItemExt {
            fn to_struct(&self) -> PlaceableType;
        }

        #[derive(Component, Clone)]
        pub enum PlaceableType {
            $(
                $name($name),
            )*
        }

        impl PlaceableType {
            pub fn is_tileable(&self) -> bool {
                match self {
                    $(
                        PlaceableType::$name(item) => item.tileable,
                    )*
                }
            }

            pub fn placeable_on_wall(&self) -> bool {
                match self {
                    $(
                        PlaceableType::$name(item) => item.placeable_on_wall,
                    )*
                }
            }

            pub fn get_current_resources(&self) -> usize {
                match self {
                    $(
                        PlaceableType::$name(item) => item.current_resources,
                    )*
                }
            }

            pub fn set_current_resources(&mut self, amount: usize) {
                match self {
                    $(
                        PlaceableType::$name(item) => item.current_resources = amount,
                    )*
                }
            }

            pub fn get_missing_resource_count(&self) -> usize {
                match self {
                    $(
                        PlaceableType::$name(item) => item.max_resources - item.current_resources,
                    )*
                }
            }
        }
    };
}

#[derive(Component)]
pub struct Tileable;

placeables! (
    struct Wall {
        placeable_on_wall: false,
        tileable: true,
    },
    struct Turret {
        placeable_on_wall: true,
        tileable: false,
    }
);

#[derive(Bundle, Clone)]
pub struct PlaceableBundle {
    pub placeable: PlaceableType,
    pub sprite_bundle: SpriteBundle,
}
