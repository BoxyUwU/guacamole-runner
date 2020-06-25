use crate::{
    shipyard::{
        *,
    },
    tetra::{
        InputContext,
        input::{
            self,
            Key,
        },
        math::{
            Vec2,
        },
    },
    consts::{
        *,
    },
    map::{
        HexMap,
    },
    components::{
        Player,
        Plane,
        Direction,
    },
};

use vermarine_lib::{
    components::{
        Transform,
    },
    rendering::{
        Sprite,
        Drawables,
        draw_buffer::{
            DrawCommand,
        },
    },
};

pub fn move_player(ctx: UniqueView<InputContext>, players: View<Player>, mut transforms: ViewMut<Transform>) {
    let mut movement: Vec2<f32> = Vec2::new(0.0, 0.0);

    for entry in [
        (Key::Up, Vec2::new(0.0, -1.0)),
        (Key::Down, Vec2::new(0.0, 1.0)),
        (Key::Left, Vec2::new(-1.0, 0.0)),
        (Key::Right, Vec2::new(1.0, 0.0)),
    ].iter() {
        if input::is_key_down(&ctx, entry.0) {
            movement += entry.1;
        }
    }

    if movement != Vec2::new(0.0, 0.0) {
        movement.normalize();
        movement *= PLAYER_SPEED;
        movement.x = movement.x.floor();
        movement.y = movement.y.floor();        
    }

    if let Some((_, transform)) = (&players, &mut transforms).iter().next() {
        transform.x += movement.x as f64;
        transform.y += movement.y as f64;
    }
}

pub fn scroll_map(mut map: UniqueViewMut<HexMap>) {
    map.position.x -= SCROLL_RATE;
}

pub struct SpawnTimer {
    cur: i32,
    max: i32,
}

impl SpawnTimer {
    pub fn new(max: i32) -> Self {
        Self {
            cur: max,
            max,
        }
    }
}

pub fn platform_spawner(mut all_storages: AllStoragesViewMut) {
    let spawn = all_storages.run(|mut spawn_timer: UniqueViewMut<SpawnTimer>| {
        if spawn_timer.cur <= 0 {
            spawn_timer.cur = spawn_timer.max;
            true    
        } else {
            spawn_timer.cur -= 1;
            false
        }
    });

    if spawn {
        use rand::prelude::*;

        let mut rng = rand::thread_rng();
        let (x, mut y) = (rng.gen_range(1279, 1280), rng.gen_range(0, 2) * 720);
        let direction;
        let rotation;
        if y == 0 {
            y = -36;
            direction = Direction::Down;
            rotation = std::f32::consts::PI;
        } else {
            direction = Direction::Up;
            rotation = 0.;
            y += 36;
        }

        let tex = all_storages.run(|drawables: NonSendSync<UniqueView<Drawables>>| {
            drawables.alias[textures::AEROPLANE]
        });

        all_storages
            .entity_builder()
            .with(Transform::new(x as f64, y as f64))
            .with(Sprite::from_command(
                DrawCommand::new(tex)
                .scale(Vec2::new(2., 2.))
                .draw_layer(draw_layers::PLANE)
                .rotation(rotation)
                .origin(Vec2::new(36., 36.))
            ))
            .with(Plane::new(direction))
            .build();
    }
}

pub fn move_planes(mut transforms: ViewMut<Transform>, planes: View<Plane>) {
    for (transform, plane) in (&mut transforms, &planes).iter() {
        let movement;
        match plane.direction {
            Direction::Up => {
                movement = Vec2::new(-SCROLL_RATE as f64 * 2., -4.)
            }
            Direction::Down => {
                movement = Vec2::new(-SCROLL_RATE as f64 * 2., 4.)
            }
        }

        transform.x += movement.x;
        transform.y += movement.y;
    }
}