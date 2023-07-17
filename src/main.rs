use bracket_terminal::prelude::*;
use drawsprites::draw_sprites;
use specs::{prelude::*, Component, VecStorage};

mod drawsprites;
mod player;
use player::manage_player_input;
mod map;
use map::{render_map, Map};
mod components;

use crate::{components::Renderable, player::Player, drawsprites::xy_to_idx};

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

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        manage_player_input(self, ctx);

        // TODO: extract func
        let mut draw_batch = DrawBatch::new();

        draw_batch.target(CL_INTERACTABLES);
        draw_batch.cls();
        draw_sprites(&self.ecs, &mut draw_batch);
        draw_batch.submit(CL_INTERACTABLES).expect("Batch error??");

        draw_batch.target(CL_TEXT).cls().print_color_with_z(
            Point::new(1, 2),
            &format!("FPS: {}", ctx.fps),
            ColorPair::new(PINK, BLACK),
            1000,
        );
        draw_batch.submit(CL_TEXT).expect("Batch error??");

        draw_batch.target(CL_WORLD);
        draw_batch.cls();
        render_map(&self.ecs, &mut draw_batch);
        draw_batch.submit(CL_WORLD).expect("Batch error??");

        render_draw_buffer(ctx).expect("Render error??");
    }
}

/// Represents a position of anything that exists physically in the game world
#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Position {
    x: usize,
    y: usize,
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

    // A very plain map
    let map = Map::new(DISPLAY_WIDTH as usize, DISPLAY_HEIGHT as usize);
    world.insert(map);

    world
        .create_entity()
        .with(Position { x: 17, y: 20 })
        .with(Player {})
        .with(Renderable::new(ColorPair::new(WHITE, BLACK), 2))
        .build();
    world
        .create_entity()
        .with(Position { x: 14, y: 10 })
        .with(Renderable::new(ColorPair::new(DARKSALMON, BLACK), xy_to_idx(1, 4, 16)))
        .build();
    world
        .create_entity()
        .with(Position { x: 15, y: 10 })
        .with(Renderable::new(ColorPair::new(ROSYBROWN, BLACK), xy_to_idx(1, 4, 16)))
        .build();
    world
        .create_entity()
        .with(Position { x: 16, y: 10 })
        .with(Renderable::new(ColorPair::new(BURLYWOOD, BLACK), xy_to_idx(1, 4, 16)))
        .build();

    let game_state: State = State { ecs: world };
    main_loop(context, game_state)
}
