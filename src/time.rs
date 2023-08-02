use std::time::Duration;

use bracket_terminal::prelude::BTerm;
use specs::{World, WorldExt};

#[derive(Default)]
pub struct DeltaTime(pub Duration);

/// Updates the DeltaTime resource in order to be used across systems which need said info
pub fn delta_time_update(ecs: &mut World, ctx: &mut BTerm) {
    let mut delta_timer = ecs.write_resource::<DeltaTime>();
    delta_timer.0 = Duration::from_secs_f32(ctx.frame_time_ms / 1000f32);
    // println!("Frame time: {:?}", delta_timer.0);
}
