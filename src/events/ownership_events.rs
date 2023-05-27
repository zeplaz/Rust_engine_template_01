/*
example usage:
fn example_system(
    mut query: Query<(&EntityId, &mut EntityInfo)>,
    mut ownership_change_events: EventWriter<OwnershipChangeEvent>,
) {
entity_info.set_owner_id(new_owner_id, ownership_change_events);
.add_event::<OwnershipChangeEvent>()
.add_system(ownership_change_listener)
*/

use crate::idgen::EntityId;

pub struct OwnershipChangeEvent {
    pub entity_id: EntityId,
    pub old_owner_id: EntityId,
    pub new_owner_id: EntityId,
}


fn ownership_change_listener(mut events: EventReader<OwnershipChangeEvent>, mut query: Query<&mut Sprite>) {
    for event in events.iter() {
        // Update the sprite's color based on the new owner_id
        if let Ok(mut sprite) = query.get_mut(event.entity_id) {
            sprite.color = get_color_for_owner(event.new_owner_id);
        }

        // Perform any other necessary updates related to the ownership change,
        // such as modifying building lists or updating trade rules.
    }
}
