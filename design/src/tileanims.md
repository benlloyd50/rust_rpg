# Tile Animations
There are many cool effects I hope to add into the game to add a bit of juice and make things just overall look cool. The
effect of these would be something like a fishing rod tile popping up in the water where someone is fishing. It could also
be a music note around a bard that is playing some nice music. Whatever the case, the goal is to handle these situations
similarly. 

## Use Cases
- I want to be able to queue up a tile animation to be created from any system.
- I want to be able to delete tile animations after a certain condition is met from the time they were created.
- I want a tile animation that will only play for a duration of time before disappearing.

## Solutions

1. RAMBLING: can i have a struct that contains the parent entity id for a tile animation and create a trait that is implemented to the
    struct that calls a func that returns a bool in order to check if the tile anim should be deleted?
