use bracket_terminal::prelude::*;

struct State {}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut draw_batch = DrawBatch::new();
        draw_batch.target(CL_INTERACTABLES);
        draw_batch.cls();
        draw_batch.set(Point::new(3, 7), ColorPair::new(WHITE, BLACK), 2u16);

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

pub const WIDTH: u32 = 40;
pub const HEIGHT: u32 = 30;

// CL - Console layer
pub const CL_INTERACTABLES: usize = 0;
pub const CL_TEXT: usize = 1;

bracket_terminal::embedded_resource!(TILE_FONT, "../resources/interactable_tiles.png");
bracket_terminal::embedded_resource!(CHAR_FONT, "../resources/terminal8x8.png");

fn main() -> BError {
    bracket_terminal::link_resource!(TILE_FONT, "resources/interactable_tiles.png");
    bracket_terminal::link_resource!(CHAR_FONT, "resources/terminal8x8.png");

    let context = BTermBuilder::new()
        .with_title("Tile RPG")
        .with_font("terminal8x8.png", 8u32, 8u32)
        .with_font("interactable_tiles.png", 8u32, 8u32)
        .with_simple_console(WIDTH, HEIGHT, "interactable_tiles.png")
        .with_tile_dimensions(8u32, 8u32)
        .with_simple_console_no_bg(WIDTH, HEIGHT, "terminal8x8.png")
        .with_dimensions(WIDTH * 3, HEIGHT * 3)
        .build()?;

    let gs: State = State {};
    main_loop(context, gs)
}
