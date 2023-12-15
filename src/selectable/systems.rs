use super::components::*;
use crate::CameraSelectedEvent;
use bevy::prelude::*;

pub fn select_selectables_in_bounds(
    mut commands: Commands,
    q_selectable: Query<(Entity, &GlobalTransform), (With<Selectable>, Without<Selected>)>,
    q_already_selected: Query<(Entity, &GlobalTransform), With<Selected>>,
    mut camera_bounds_event_reader: EventReader<CameraSelectedEvent>,
) {
    for CameraSelectedEvent {
        lower_right,
        upper_left,
    } in camera_bounds_event_reader.read()
    {
        // first, deselect all entities that are selected but not in the bounds
        for (entity, transform) in &q_already_selected {
            let translation = transform.translation();
            if translation.x < upper_left.x
                || translation.x > lower_right.x
                || translation.y < upper_left.y
                || translation.y > lower_right.y
            {
                commands.entity(entity).deselect();
            }
        }

        // then, select all entities that are in the bounds
        for (entity, transform) in &q_selectable {
            let translation = transform.translation();
            if translation.x >= upper_left.x
                && translation.x <= lower_right.x
                && translation.y >= upper_left.y
                && translation.y <= lower_right.y
            {
                commands.entity(entity).select();
            }
        }
    }
}
