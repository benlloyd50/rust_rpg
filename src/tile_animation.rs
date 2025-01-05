use std::time::Duration;

use bracket_lib::terminal::ColorPair;
use specs::{Entities, Entity, Join, Read, ReadStorage, System, Write, WriteStorage};

use crate::{
    components::{DeleteCondition, FinishedActivity, GlyphFlash, Position, Renderable, SizeFlexor, Transform},
    draw_sprites::lerp_point,
    time::DeltaTime,
    z_order::TILE_ANIM_Z,
};

#[derive(Default)]
pub struct TileAnimationBuilder {
    requests: Vec<AnimationRequest>,
}

impl TileAnimationBuilder {
    pub fn new() -> Self {
        Self { requests: Vec::new() }
    }

    pub fn request(&mut self, anim_request: AnimationRequest) {
        self.requests.push(anim_request);
    }
}

pub enum AnimationRequest {
    StaticTile(u8, Position, ColorPair, DeleteCondition),
    StretchShrink(Entity, SizeFlexor),
    GlyphFlash(Entity, Duration, Renderable),
}

pub struct TileAnimationSpawner;

impl<'a> System<'a> for TileAnimationSpawner {
    type SystemData = (
        Entities<'a>,
        Write<'a, TileAnimationBuilder>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, SizeFlexor>,
        WriteStorage<'a, GlyphFlash>,
        WriteStorage<'a, DeleteCondition>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut anim_builder,
            mut positions,
            mut transforms,
            mut renderables,
            mut flexors,
            mut color_flashes,
            mut delete_conditions,
        ): Self::SystemData,
    ) {
        for request in anim_builder.requests.iter() {
            match request {
                AnimationRequest::StaticTile(atlas_index, at, fgbg, delete_condition) => {
                    let new_anim = entities.create();
                    let _ = positions.insert(new_anim, *at);
                    let _ = renderables.insert(
                        new_anim,
                        Renderable { color_pair: *fgbg, atlas_index: *atlas_index, z_priority: TILE_ANIM_Z },
                    );
                    let _ = delete_conditions.insert(new_anim, *delete_condition);
                }
                AnimationRequest::StretchShrink(who, flexor) => {
                    let _ = flexors.insert(*who, flexor.clone());

                    let pos = positions.get(*who).unwrap();
                    let default = Transform::new(pos.x as f32, pos.y as f32, 0.0, 1.0, 1.0);
                    let _ = transforms.insert(*who, default);
                }
                AnimationRequest::GlyphFlash(who, time_left, flash) => {
                    let at = positions.get(*who).unwrap();

                    let new_anim = entities.create();
                    let _ = positions.insert(new_anim, *at);
                    let _ =
                        color_flashes.insert(new_anim, GlyphFlash { _time_left: *time_left, sprite: flash.clone() });
                    let _ = delete_conditions.insert(new_anim, DeleteCondition::Timed(*time_left));
                }
            }
        }
        anim_builder.requests.clear();
    }
}

pub struct TileAnimationUpdater;

impl<'a> System<'a> for TileAnimationUpdater {
    type SystemData = (WriteStorage<'a, Transform>, WriteStorage<'a, SizeFlexor>, Read<'a, DeltaTime>, Entities<'a>);

    fn run(&mut self, (mut transforms, mut flexors, dt, entities): Self::SystemData) {
        let mut remove_mes = vec![];
        for (e, transform, flex) in (&entities, &mut transforms, &mut flexors).join() {
            if flex.curr >= flex.points.len() {
                remove_mes.push(e);
                continue;
            }
            transform.scale = lerp_point(
                &transform.scale,
                flex.points[flex.curr].0,
                flex.points[flex.curr].1,
                flex.scalar * dt.0.as_secs_f32(),
            );
            if (transform.scale.x - flex.points[flex.curr].0).abs() <= 0.01 {
                println!("{}", transform.scale.x);
                flex.curr += 1;
            }
        }
        for remove in remove_mes {
            flexors.remove(remove);
        }
    }
}

// NOTE: Since some tile animations live on other entities that contain important component, they
// will not always have a delte condition and could instead be cleaned up by other means for
// example the dead tile cleanup which checks HealthStats.
pub struct TileAnimationCleanUpSystem;

impl<'a> System<'a> for TileAnimationCleanUpSystem {
    type SystemData =
        (Entities<'a>, ReadStorage<'a, FinishedActivity>, WriteStorage<'a, DeleteCondition>, Read<'a, DeltaTime>);

    fn run(&mut self, (entities, finished_activities, mut delete_conditions, dt): Self::SystemData) {
        for (e, condition) in (&entities, &mut delete_conditions).join() {
            match condition {
                DeleteCondition::ActivityFinish(spawner) => {
                    if finished_activities.contains(*spawner) {
                        let _ = entities.delete(e);
                    }
                }
                DeleteCondition::Timed(time_left) => {
                    *time_left = time_left.saturating_sub(dt.0);
                    if time_left.is_zero() {
                        let _ = entities.delete(e);
                    }
                }
            }
        }
    }
}
