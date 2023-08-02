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

### ECS Spec
The component system structure will be setup accordingly. When the entity walks into a fishable tile they will start the fishing action.
This will include the FishAction ccomponent being added to said entity. The system that deals with fishing actions should call out to 
create a new tile animation that will show a fishing rod in front of the entity, it is not important whether there is another there or
not. The same system will also remove the fishing action from the entity and add the WaitingForFish component. The next system will be
for checking if a fish is on the line. This will try rolling a number and check against a fishing threshold of the entity that
if passing will add a FishOnTheLine component to the entity, in the process removing the WaitingForFish component. Finally,
FishOnTheLine gets dealt in two seperate systems(maybe one?). One for player which would be a minigame of pressing a button or
for the npc another rng call to see if they successfully caught the fish.

At some unknown point the tile animation will also need to be cleaned up, but will give some more thought on how to do that.
The fisher could hang on to the entity for the tile animation and call to delete it after finishing the fishing activity.

From here, the items being given will need to be handled seperately.

## Limitations
- Each tile of water will have an entity containing a fishable component if it is fishable
    - The ability to fish on a tile will be generated and index post map creation
- The action of fishing will be an entity with a tile animation component on it.
- Information about what fish are available could be stored on:
    - a resource set for the current map the player is in
    - the map setting every fishable tile to the same pool of fish
    - the tile setting every fishable tile to their own pool of fish
    - the entity fishing setting every fishable tile to the pool provided by the fisher
- When the entity is waiting for fish, movement should be ignored. The action could be cancelled by `some button`


## Related Features
- Cooking to create dishes with fish
- Herbalism to use the fish to create different effects
- Crafting to use the various byproducts of fish (oil, skin, etc.) and also modding fishing rods
