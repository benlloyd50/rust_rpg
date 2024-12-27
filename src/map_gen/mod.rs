use specs::World;

use crate::map::Map;

pub struct MapSettings {
    world_type: WorldType,
}

enum WorldType {
    Normal,
}

pub fn create_random_map(ecs: &mut World, settings: MapSettings) -> Map {
    let mut map = Map::new(100, 100);

    map
}
