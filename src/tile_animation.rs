use std::time::Duration;

use bracket_terminal::prelude::ColorPair;
use specs::{Entities, Entity, Join, Read, ReadStorage, System, Write, WriteStorage};

use crate::{
    components::{DeleteCondition, FinishedActivity, Position, Renderable, SizeFlexor, Transform},
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
            mut delete_conditions,
        ): Self::SystemData,
    ) {
        for request in anim_builder.requests.iter() {
            let new_anim = entities.create();
            match request {
                AnimationRequest::StaticTile(atlas_index, at, fgbg, delete_condition) => {
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
            }
        }
        anim_builder.requests.clear();
    }
}

pub struct TileAnimationUpdater;

impl<'a> System<'a> for TileAnimationUpdater {
    type SystemData = (
        WriteStorage<'a, Transform>,
        WriteStorage<'a, SizeFlexor>,
        WriteStorage<'a, DeleteCondition>,
        Read<'a, DeltaTime>,
        Entities<'a>,
    );

    fn run(&mut self, (mut transforms, mut flexors, mut delete_conditions, dt, entities): Self::SystemData) {
        for (e, transform, flex) in (&entities, &mut transforms, &mut flexors).join() {
            if flex.curr >= flex.points.len() {
                let _ = delete_conditions.insert(e, DeleteCondition::Timed(Duration::ZERO));
                continue;
            }
            transform.scale = lerp_point(
                &transform.scale,
                flex.points[flex.curr].0,
                flex.points[flex.curr].1,
                flex.scalar * dt.0.as_secs_f32(),
            );
            if (transform.scale.x - flex.points[flex.curr].0).abs() <= 0.1 {
                flex.curr += 1;
            }
        }
    }
}

// NOTE: Since some tile animations live on other entities that contain important component, they
// will not always have a delte condition and could instead be cleaned up by other means for
// example the dead tile cleanup which checks HealthStats.
pub struct TileAnimationCleanUpSystem;

impl<'a> System<'a> for TileAnimationCleanUpSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, FinishedActivity>,
        WriteStorage<'a, DeleteCondition>,
        WriteStorage<'a, Transform>,
        Read<'a, DeltaTime>,
    );

    fn run(&mut self, (entities, finished_activities, mut delete_conditions, mut transforms, dt): Self::SystemData) {
        let mut remove_mes = vec![];
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
                        remove_mes.push(e);
                        // for now timed tile anims are on a related important object
                        // let _ = entities.delete(e);
                    }
                }
            }
        }

        for e in remove_mes.iter() {
            transforms.remove(*e);
            delete_conditions.remove(*e);
        }
    }
}
