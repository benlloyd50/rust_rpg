# Fishing

## Use Cases
**Actor**: entity
- I would like to be able to use a fishing rod or a special ability to catch fish from bodies of water.
- I will be able to go to different bodies of water to find different fish.
- Fish will have different uses and different kinds should be interchangable for *most* recipes.

## Gameplay & Design
![Fishing In Game](../images/fishing_idea400%.png)

Fishing consists of 3 states:
- casting triggered from walking into a fishable tile
- waiting which includes the fishing minigame
- reeling when the fishing minigame concludes and the fish/item is recieved

### Casting
The action of starting to fish will be triggered by walking into a tile that has the fishable component. Via map indexing the map
will be aware of when the player walks into a fishable tile. 

### Waiting
The state of waiting will begin the frame after the entity starts fishing. If an npc is in this state the game will simply wait a
random amount of time and will attempt to catch the fish. If the player is the one in this state, a minigame will occur that requires
a timed press in order to secure the catch.

### Reeling
The state of reeling will occur if the waiting state results in a successful catch. This state is mainly used for cleaning up the 
effects and rewarding items.

## Limitations
- Each tile of water will have an entity containing a fishable component if it is fishable
    - The ability to fish on a tile will be generated and index post map creation
- The action of fishing will be an entity with a position and renderable so it is shown

## Related Features
- Cooking to create dishes with fish
- Herbalism to use the fish to create different effects
- Crafting to use the various byproducts of fish (oil, skin, etc.)

