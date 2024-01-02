use crate::placeable::RequestPlacementEvent;
use crate::utils::*;
use bevy::prelude::*;

pub struct WorkOrderPlugin;

impl Plugin for WorkOrderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, listen_for_placeable_events);
    }
}

fn listen_for_placeable_events(
    mut commands: Commands,
    mut event_listener: EventReader<RequestPlacementEvent>,
    mut work_orders: ResMut<super::WorkQueue>,
    mut navmesh: ResMut<crate::navmesh::SpatialGrid>,
) {
    for event in event_listener.read() {
        for placeable in &event.0 {
            let mut placeable = placeable.clone();
            placeable.sprite_bundle.transform.translation.z = 1.0;
            placeable.sprite_bundle.sprite.color = Color::rgba(1.0, 1.0, 1.0, 0.5);

            let placeable_grid_pos = placeable.sprite_bundle.transform.translation.xy();
            let placeable_grid_pos = GridPos::from_world_pos_vec(placeable_grid_pos);

            let entity = commands.spawn(placeable).id();
            work_orders.build_queue.push_back(entity);

            let spatial_entity =
                navmesh.create_spatial_entity(entity, placeable_grid_pos, true, None, None);

            commands.entity(entity).insert(spatial_entity.watch());
        }
    }
}
