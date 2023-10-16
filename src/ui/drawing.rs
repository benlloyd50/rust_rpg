use bracket_terminal::prelude::{to_cp437, ColorPair, DrawBatch, Rect};

use crate::colors::white_fg;

pub trait AccentBox {
    fn draw_accent_box(&mut self, size: Rect, color: ColorPair);
}

impl AccentBox for DrawBatch {
    fn draw_accent_box(&mut self, size: Rect, color: ColorPair) {
        self.draw_hollow_box(
            Rect::with_exact(size.x1, size.y1, size.x2, size.y2),
            white_fg(color.fg.into()),
        );
        self.fill_region(
            Rect::with_exact(size.x1 + 1, size.y1 + 1, size.x2, size.y2),
            white_fg(color.bg.into()),
            to_cp437(' '),
        );
    }
}
