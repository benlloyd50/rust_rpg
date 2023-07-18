use bracket_terminal::prelude::*;
use draw_sprites::draw_all_layers;
use specs::prelude::*;

mod draw_sprites;
mod player;
use player::manage_player_input;
mod map;
use map::{Map, MapIndexingSystem};
mod components;
use components::Position;

use crate::{components::{Renderable, Blocking, Breakable}, draw_sprites::debug_rocks, player::Player};

// Size of the terminal window
pub const DISPLAY_WIDTH: u32 = 40;
pub const DISPLAY_HEIGHT: u32 = 30;

// CL - Console layer, represents the indices for each console
pub const CL_TEXT: usize = 2; // Used for UI
pub const CL_WORLD: usize = 0; // Used for terrain tiles
pub const CL_INTERACTABLES: usize = 1; // Used for the few or so moving items/entities on screen

pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut mapindex = MapIndexingSystem;
        mapindex.run_now(&self.ecs);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        manage_player_input(self, ctx);
        self.run_systems();

        draw_all_layers(&self.ecs, ctx);
    }
}

bracket_terminal::embedded_resource!(TILE_FONT, "../resources/interactable_tiles.png");
bracket_terminal::embedded_resource!(CHAR_FONT, "../resources/terminal8x8.png");
bracket_terminal::embedded_resource!(TERRAIN_FOREST, "../resources/terrain_forest.png");

fn main() -> BError {
    bracket_terminal::link_resource!(TILE_FONT, "resources/interactable_tiles.png");
    bracket_terminal::link_resource!(CHAR_FONT, "resources/terminal8x8.png");
    bracket_terminal::link_resource!(TERRAIN_FOREST, "resources/terrain_forest.png");

    // Setup Terminal (incl Window, Input)
    let context = BTermBuilder::new()
        .with_title("Tile RPG")
        .with_fps_cap(60.0)
        .with_font("terminal8x8.png", 8u32, 8u32)
        .with_font("interactable_tiles.png", 8u32, 8u32)
        .with_font("terrain_forest.png", 8u32, 8u32)
        .with_dimensions(DISPLAY_WIDTH * 3, DISPLAY_HEIGHT * 3)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terrain_forest.png")
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "interactable_tiles.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terminal8x8.png")
        .build()?;

    // Setup ECS
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Player>();
    world.register::<Renderable>();
    world.register::<Blocking>();
    world.register::<Breakable>();

    // A very plain map
    let map = Map::new(DISPLAY_WIDTH as usize, DISPLAY_HEIGHT as usize);
    world.insert(map);

    world
        .create_entity()
        .with(Position::new(17, 20))
        .with(Player {})
        .with(Renderable::new(ColorPair::new(WHITE, BLACK), 2))
        .with(Blocking)
        .build();

    debug_rocks(&mut world);

    let game_state: State = State { ecs: world };
    main_loop(context, game_state)
}
