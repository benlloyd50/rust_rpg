# Map, Tiles, Entities

The map exists as a struct containing lots of useful information. It stores the index into the atlas for the sprite, where static world tiles are located, 
and the contents of a tile such as the entities currently existing on it. It also has some meta data like the size of the map.

The map's *old* (current as of writing, old as of reading hopefully) implementation struggles with the case of static world tiles
that have entity data. There should be a way to contain information such as a static water tile that is fishable and blocks the player.
While another water tile adjacent does neither.

## Use Cases
- I want to be able to query the map in a simple manner to know what I am going to collide into.
- I need to be able to check a tile to see if it possesses some quality on an entity.
- I have a tile with many entities i.e. an item, a monster, a door. The tile_contents show 3 entity ids are present.
I want to move into that tile but I will need to handle all the entities there in a deterministic(?) order i.e. attack monster,
check if door is unlocked, check if door is opened, pick up item(s).


## Solutions

### Implemented) Create Entity When Making Static Tiles
When we create a static tile on the map, two steps should occur if entity data will be needed:

1. Update the tile on the map with the tile for water
2. Create an entity with the same position and the components for the desired behavior

On deletion we need to remove the entity so there is no out of sync data between map and entities in a position. There must also be a way
to prioritize actions based on the content in the tile. In the third use case


- Pros:
    - This should allow entities to have many different components for different features
    - Only create entities when needed
    - 
- Cons:
    - entity and tile mismatch/out of sync may be possible

