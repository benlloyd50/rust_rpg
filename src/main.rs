use bracket_terminal::prelude::*;

struct State {}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        let mut draw_batch = DrawBatch::new();
        // draw_batch.target(1);
        // draw_batch.cls();

        draw_batch.target(0);
        draw_batch.cls();
        draw_batch.print(Point::new(1, 1), "Hello Ben");
        draw_batch.print_color_with_z(Point::new(1, 2), &format!("FPS: {}", ctx.fps), ColorPair::new(PINK, BLACK), 1000);

        draw_batch.submit(0).expect("Batch error??");
        render_draw_buffer(ctx).expect("Render error??");
    }
}

pub const WIDTH: u32 = 40;
pub const HEIGHT: u32 = 30;

bracket_terminal::embedded_resource!(TILE_FONT, "../resources/example_tiles.png");
bracket_terminal::embedded_resource!(CHAR_FONT, "../resources/terminal8x8.png");

fn main() -> BError {
    bracket_terminal::link_resource!(TILE_FONT, "resources/example_tiles.png");
    bracket_terminal::link_resource!(CHAR_FONT, "resources/terminal8x8.png");

    let context = BTermBuilder::new()
        .with_title("Tile RPG")
        .with_simple_console(WIDTH, HEIGHT, "terminal8x8.png")
        .with_font("terminal8x8.png", 8u32, 8u32)
        .with_dimensions(WIDTH, HEIGHT)
        // .with_simple_console(WIDTH, HEIGHT, "example_tiles.png")
        // .with_font("example_tiles.png", 16u32, 16u32)
        // .with_tile_dimensions(16u32, 16u32)
        .build()?;

    let gs: State = State {};
    main_loop(context, gs)
}
