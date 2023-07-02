use bracket_terminal::prelude::*;
use specs::{prelude::*, Component, VecStorage};

mod player;
use player::manage_player_input;

use crate::player::Player;

// Size of the terminal window
pub const DISPLAY_WIDTH: u32 = 40;
pub const DISPLAY_HEIGHT: u32 = 30;

// CL - Console layer, represents the indices for each console
pub const CL_INTERACTABLES: usize = 0;
pub const CL_TEXT: usize = 1;

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

        let positions = self.ecs.read_storage::<Position>();
        for pos in positions.join() {
            draw_batch.set(Point::new(pos.x, pos.y), ColorPair::new(WHITE, BLACK), 2u16);
        }
        
        draw_batch.target(CL_TEXT);
        draw_batch.cls();
        draw_batch.print(Point::new(1, 1), "Hello Ben");
        draw_batch.print_color_with_z(
            Point::new(1, 2),
            &format!("FPS: {}", ctx.fps),
            ColorPair::new(PINK, BLACK),
            1000,
        );

        draw_batch.submit(0).expect("Batch error??");
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

fn main() -> BError {
    bracket_terminal::link_resource!(TILE_FONT, "resources/interactable_tiles.png");
    bracket_terminal::link_resource!(CHAR_FONT, "resources/terminal8x8.png");

    // Setup Terminal (incl Window, Input)
    let context = BTermBuilder::new()
        .with_title("Tile RPG")
        .with_font("terminal8x8.png", 8u32, 8u32)
        .with_font("interactable_tiles.png", 8u32, 8u32)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "interactable_tiles.png")
        .with_tile_dimensions(8u32, 8u32)
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terminal8x8.png")
        .with_dimensions(DISPLAY_WIDTH * 3, DISPLAY_HEIGHT * 3)
        .build()?;

    // Setup ECS
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Player>();

    world.create_entity().with(Position { x: 17, y: 20}).with(Player {}).build();

    let game_state: State = State { ecs: world };
    main_loop(context, game_state)
}
