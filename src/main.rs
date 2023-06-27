use std::process::exit;

use bracket_terminal::prelude::*;
use specs::{prelude::*, Component, VecStorage};

struct State {
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

fn manage_player_input(state: &mut State, ctx: &BTerm) {
    match ctx.key {
        None => {},
        Some(key) => {
            match key  {
                VirtualKeyCode::W | VirtualKeyCode::Up => try_move_player(0, -1, &mut state.ecs),
                VirtualKeyCode::S | VirtualKeyCode::Down => try_move_player(0, 1, &mut state.ecs),
                VirtualKeyCode::A | VirtualKeyCode::Left => try_move_player(-1, 0, &mut state.ecs),
                VirtualKeyCode::D | VirtualKeyCode::Right => try_move_player(1, 0, &mut state.ecs),
                VirtualKeyCode::Escape => {exit(99);}
                _ => {}
            }
        }
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();

    for (pos, _) in (&mut positions, &players).join() {
        let target_pos = Point::new(pos.x as i32 + delta_x, pos.y as i32 + delta_y);
        
        // check target_pos is a valid position to move into (in map bounds, not blocked by another entity or tile)
        if target_pos.x < 0 || target_pos.y < 0 || target_pos.x >= WIDTH as i32 || target_pos.y >= HEIGHT as i32 {
            return;
        }

        println!("Moving");
        pos.x = target_pos.x as usize;
        pos.y = target_pos.y as usize;
    }
}

pub const WIDTH: u32 = 40;
pub const HEIGHT: u32 = 30;

// CL - Console layer, represents the indices for each console
pub const CL_INTERACTABLES: usize = 0;
pub const CL_TEXT: usize = 1;

/// Represents a position of anything that exists physically in the game world
#[derive(Debug, Component)]
#[storage(VecStorage)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Default, Component)]
#[storage(NullStorage)]
struct Player {}

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
        .with_simple_console(WIDTH, HEIGHT, "interactable_tiles.png")
        .with_tile_dimensions(8u32, 8u32)
        .with_simple_console_no_bg(WIDTH, HEIGHT, "terminal8x8.png")
        .with_dimensions(WIDTH * 3, HEIGHT * 3)
        .build()?;

    // Setup ECS
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Player>();

    world.create_entity().with(Position { x: 17, y: 20}).with(Player {}).build();

    let game_state: State = State { ecs: world };
    main_loop(context, game_state)
}
