use crate::{
    shipyard::{
        *,
    },
    tetra::{
        InputContext,
        graphics::{
            Camera,
        },
        input::{
            self,
            Key,
            MouseButton,
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
};

pub fn move_camera(mut camera: UniqueViewMut<Camera>, input: UniqueView<InputContext>) {
    let mut movement: Vec2<f32> = Vec2::new(0.0, 0.0);

    for entry in [
        (Key::Up, Vec2::new(0.0, -1.0)),
        (Key::Down, Vec2::new(0.0, 1.0)),
        (Key::Left, Vec2::new(-1.0, 0.0)),
        (Key::Right, Vec2::new(1.0, 0.0)),
    ].iter() {
        if input::is_key_down(&input, entry.0) {
            movement += entry.1;
        }
    }

    if movement != Vec2::new(0.0, 0.0) {
        movement.normalize();
        movement *= CAM_SPEED;
        movement.x = movement.x.floor();
        movement.y = movement.y.floor();        
        camera.position += movement;
    }
}

pub fn update_hex_map(input_ctx: UniqueView<InputContext>, mut map: UniqueViewMut<HexMap>, camera: UniqueView<Camera>) {
    let (sel_x, sel_y) = 
        if let Some(hex) = map.pixel_to_hex(camera.mouse_position(&input_ctx)) {
            hex
        } else {
            return;
        };
    let (x, y) = (sel_x as usize, sel_y as usize);
    
    let map_width = map.width;
    let tile = &mut map.tiles[y * map_width + x];

    if input::is_mouse_button_pressed(&input_ctx, MouseButton::Left) {
        if tile.ground_height > tile.wall_height && tile.ground_height > 0 {
            tile.ground_height -= 1;
        }
        else if tile.wall_height > tile.ground_height && tile.wall_height > 0 {
            tile.wall_height -= 1;
        }
        else if tile.wall_height == tile.ground_height && tile.wall_height > 0 {
            tile.wall_height -= 1;
            tile.ground_height -= 1;
        }
    } else if input::is_mouse_button_pressed(&input_ctx, MouseButton::Right) {
        if tile.ground_height > tile.wall_height {
            tile.wall_height = tile.ground_height + 1;
        }
        else if tile.wall_height >= tile.ground_height && tile.wall_height < MAX_BRICK_HEIGHT {
            tile.wall_height += 1;
        }
    }

    let height = tile.wall_height;
    if height > map.tallest {
        map.tallest = height;
    }
}

pub struct CamScroller {
    cur: i32,
    max: i32,
    stored: f32,
}

impl CamScroller {
    pub fn new(max: i32) -> Self {
        Self {
            cur: max,
            max,
            stored: 0.,
        }
    }
}

pub fn scroll_map(mut scroller: UniqueViewMut<CamScroller>, mut map: UniqueViewMut<HexMap>) {
    if scroller.cur <= 0 {
        scroller.stored += SCROLL_RATE;
        scroller.cur = scroller.max;
    } else {
        scroller.cur -= 1;
    }

    while scroller.stored >= 1.0 {
        scroller.stored -= 1.0;
        map.position.x -= 1.0;
    }
}