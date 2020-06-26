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
        Collider,
        Height,
        Points,
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
    let mut movement: Vec2<f32> = Vec2::zero();

    if input::is_key_down(&ctx, Key::Down) {
        movement += Vec2::new(-0.5, 2.);
    }
    if input::is_key_down(&ctx, Key::Up) {
        movement += Vec2::new(-0.5, -2.);
    }

    if movement == Vec2::zero() {
        if input::is_key_down(&ctx, Key::Left) {
            movement += Vec2::new(-5., 0.);
        }
        else if input::is_key_down(&ctx, Key::Right) {
            movement += Vec2::new(1., 0.);
        }
    } else if input::is_key_down(&ctx, Key::Left) {
        movement = Vec2::new(-5., 0.);
    }

    if movement != Vec2::new(0.0, 0.0) {
        movement *= PLAYER_SPEED;
        movement.x = movement.x.floor();
        movement.y = movement.y.floor();        
    }

    if let Some((_, transform)) = (&players, &mut transforms).iter().next() {
        transform.x += movement.x as f64;
        transform.y += movement.y as f64;

        transform.x = transform.x.max(-8. + -72. + (20. * 3.)).min(1240. - 62. + (20. * 3.));
        transform.y = transform.y.max(-35. + (18. * 3.)).min(647. + (18. * 3.));
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
            cur: 0,
            max,
        }
    }
}

pub fn platform_spawner(all_storages: AllStoragesViewMut) {
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
        let (x, mut y) = (rng.gen_range(800, 1280), rng.gen_range(0, 2) * 720);
        let direction;
        let rotation;
        let collider;
        if y == 0 {
            y = -36;
            direction = Direction::Down;
            rotation = std::f32::consts::PI;
            collider = Collider::new(-32 * 2, -10 * 2, 64 * 2, 26 * 2);
        } else {
            direction = Direction::Up;
            rotation = 0.;
            y += 36;
            collider = Collider::new(-32 * 2, -16 * 2, 64 * 2, 26 * 2);
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
            .with(collider)
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

pub fn grow_ground(transforms: View<Transform>, players: View<Player>, mut map: UniqueViewMut<HexMap>, mut points: UniqueViewMut<Points>) {
    use crate::map::cube_round;
    for (transform, _) in (&transforms, &players).iter() {
        let mut pos = Vec2::new(transform.x as f32, transform.y as f32);
        pos.y += 18. * 3.;
        pos.x += 18. * 3.;

        let adjacent = [
            (0, 0),
            (1, -1),
            (1, 0),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (0, -1),
        ];

        let (q, r) = map.pixel_to_hex_raw(pos / 2., 0.);
        let (q, r, _) = cube_round(q, r, -r - q);        

        for (q_mod, r_mod) in &adjacent {
            let r = r + r_mod;
            let q = q + q_mod;

            if q >= WIDTH as i32 || q < 0 || r >= HEIGHT as i32 || r < 0 {
                continue;
            }

            if let Some(tile) = map.tiles.get_mut((r * WIDTH as i32 + q) as usize) {
                if tile.is_tilled && !tile.is_grown{
                    tile.is_grown = true;
                    points.0 += POINTS_GROW;
                }
            }
        }
    }
}

pub fn player_platform_check(player: View<Player>, transforms: View<Transform>, colliders: View<Collider>, mut heights: ViewMut<Height>) {
    let (_, p_transform, p_collider, height) = (&player, &transforms, &colliders, &mut heights).iter().next().unwrap();
    height.0 -= FALL_SPEED;
    let (p_transform, p_collider) = ((*p_transform).clone(), (*p_collider).clone());

    for (transform, collider, _) in (&transforms, &colliders, !&player).iter() {
        if Collider::intersects(collider, transform, &p_collider, &p_transform) {
            height.0 = START_HEIGHT;
            return;
        }
    }
}

pub fn player_height_visualiser(player: View<Player>, height: View<Height>, mut sprite: ViewMut<Sprite>) {
    let (_, height, sprite) = (&player, &height, &mut sprite).iter().next().unwrap();
    let mut percent = height.0 / START_HEIGHT;
    percent *= percent;
    let start = 1.;
    let end = 3.;
    let offset = percent * (end - start);
    let lerped = start + offset;
    sprite.0.scale = Vec2::new(lerped, lerped);
}