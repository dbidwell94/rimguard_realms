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
    mut navmesh: ResMut<crate::navmesh::Navmesh>,
) {
    for event in event_listener.read() {
        for placeable in &event.0 {
            let mut placeable = placeable.clone_bundle_dyn();
            placeable.sprite_bundle.transform.translation.z = 1.0;

            let placeable_grid_pos = placeable.sprite_bundle.transform.translation.xy();
            let placeable_grid_pos = GridPos::from_world_pos_vec(placeable_grid_pos);

            let entity = commands.spawn(placeable).id();
            work_orders.build_queue.push_back(entity);

            navmesh.0[placeable_grid_pos.x as usize][placeable_grid_pos.y as usize]
                .occupied_by
                .insert(entity);
        }
    }
}
