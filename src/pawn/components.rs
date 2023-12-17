use crate::assets::CharacterFacing;
use bevy::prelude::*;
pub use pawn_status::ClearStatus;
use std::collections::VecDeque;
pub use work_order::ClearWorkOrder;

#[derive(Component, Reflect)]
pub struct Pawn {
    pub move_path: VecDeque<Vec2>,
    pub move_to: Option<Vec2>,
    pub health: usize,
    pub max_health: usize,
    pub animation_timer: Timer,
    pub work_timer: Timer,
    pub search_timer: Timer,
    pub retry_pathfinding_timer: Timer,
    pub moving: bool,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct HealthBar;

#[derive(Bundle)]
pub struct HealthBundle {
    pub health_bundle: SpriteBundle,
    pub health_bar: HealthBar,
}

#[derive(Bundle)]
pub struct PawnBundle {
    pub character_facing: CharacterFacing,
    pub name: Name,
    pub sprite_bundle: SpriteSheetBundle,
    pub pawn: Pawn,
    pub pawn_status: pawn_status::PawnStatus,
    pub resources: CarriedResources,
}

#[derive(Component, Reflect)]
pub struct CarriedResources(pub usize);

pub mod pawn_status {
    use bevy::{ecs::system::EntityCommands, prelude::*};

    macro_rules! pawn_status {
        ($($name:ident),*) => {
            $(
                #[derive(Clone, Eq, PartialEq, Debug, Reflect)]
                pub struct $name;
            )*

            #[derive(Component, Clone, Eq, PartialEq, Debug, Reflect)]
            pub enum PawnStatus {
                $(
                    $name($name),
                )*
            }

            pub trait AddStatus {
                fn add_status(&mut self, status: PawnStatus) -> &mut Self;
            }

            pub trait ClearStatus {
                fn clear_status(&mut self) -> &mut Self;
            }
            impl ClearStatus for EntityCommands<'_, '_, '_> {
                fn clear_status(&mut self) -> &mut Self {
                    self.remove::<PawnStatus>();
                    self
                }
            }

            impl AddStatus for EntityCommands<'_, '_, '_> {
                fn add_status(&mut self, status: PawnStatus) -> &mut Self {
                    self.clear_status();
                    self.try_insert(status)
                }
            }
        }
    }

    pawn_status!(
        Idle,
        Pathfinding,
        PathfindingError,
        Moving,
        Mining,
        Attacking,
        Building
    );
}

pub mod work_order {
    use bevy::{ecs::system::EntityCommands, prelude::*};

    macro_rules! work_orders {
        (
            $(struct $name: ident {
            $(
                $field: ident: $ty: ty
            ),* $(,)?
            }),*
    ) => {
            $(
                #[derive(Clone, Eq, PartialEq, Reflect)]
                pub struct $name {
                    $(
                        pub $field: $ty
                    ),*
                }
            )*

            #[derive(Component, Clone, Eq, PartialEq, Reflect)]
            pub enum WorkOrder {
                $(
                    $name($name),
                )*
            }

            pub trait ClearWorkOrder {
                fn clear_work_order(&mut self) -> &mut Self;
            }

            pub trait AddWorkOrder {
                fn add_work_order(&mut self, order: WorkOrder) -> &mut Self;
            }

            impl AddWorkOrder for EntityCommands<'_, '_, '_> {
                fn add_work_order(&mut self, order: WorkOrder) -> &mut Self {
                    self.clear_work_order();
                    self.try_insert(order)
                }
            }

            impl ClearWorkOrder for EntityCommands<'_, '_, '_> {
                fn clear_work_order(&mut self) -> &mut Self {
                    self.remove::<WorkOrder>();
                    self
                }
            }
        };
    }

    work_orders!(
        struct MineStone {
            stone_entity: Entity,
        },
        struct ReturnToFactory {},
        struct PickupStoneFromFactory {
            for_entity: Entity,
        },
        struct BuildItem {
            item_entity: Entity,
        },
        struct AttackPawn {
            pawn_entity: Entity,
        },
        struct AttackFactory {}
    );
}
