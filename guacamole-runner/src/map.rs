use crate::{
    shipyard::{
        *,
    },
    consts::{
        *,
    },
    tetra::{
        math::{
            Vec3,
            Vec2,
        },
        graphics::{
            Color,
        },
    },
};

use vermarine_lib::{
    rendering::{
        draw_buffer::{
            DrawBuffer,
            DrawCommand,
        },
        Drawables,
    },
};

use rand::SeedableRng;
use rand::Rng;
use rand::rngs::StdRng;

pub struct HexTileData {
    pub ground_height: u8,
    pub wall_height: u8,
    pub is_tilled: bool,
    pub is_grown: bool,
}

impl HexTileData {
    pub fn new(height: u8) -> HexTileData {
        HexTileData {
            ground_height: height,
            wall_height: height,
            is_tilled: false,
            is_grown: false,
        }
    }
}

pub struct HexMap {
    pub tiles: Vec<HexTileData>,
    pub width: usize,
    pub height: usize,
    pub position: Vec2<f32>,
    pub tallest: u8,
}

impl HexMap {
    pub fn new(width: usize, height: usize) -> Self {
        let mut rand = StdRng::seed_from_u64(100);
        let mut tiles = Vec::<HexTileData>::with_capacity(width * height);

        let mut tallest = 0;
        for _ in 0..width * height {
            let value = rand.gen_range(0, MAX_FLOOR_HEIGHT + 1);
            let tile = HexTileData::new(value);            
            tiles.push(tile);
            if value > tallest {
                tallest = value;
            }
        }

        for _ in 0..5 {
            for section in 0..(width / 10) {
                let col = rand.gen_range(0, height + 1);
                let mut total = 0;
                for _ in 0..5 {
                    total += rand.gen_range(3, 7 + 1);    
                }
                total /= 5;
    
                for offset in 0..total {
                    if let Some(tile) = tiles.get_mut((col * width) + (section * 10) + offset) {
                        tile.is_tilled = true;
                    }
                } 
            }
        }

        let height_px = {
            height as f32 * FLOOR_VERT_STEP
        };
        let position = Vec2::new(
            0.,
            360. - height_px,
        );
        
        HexMap {
            tiles,
            width,
            height,
            position,
            tallest,
        }
    }

    pub fn pixel_to_hex_raw(&mut self, pos: Vec2<f32>, height_offset: f32) -> (f32, f32) {
        let mut pos = pos;
        pos -= Vec2::new(18., 18.);
        pos.x -= self.position.x;
        pos.y -= self.position.y;
        pos.y += height_offset;

        let size_x = FLOOR_WIDTH / f32::sqrt(3.0);
        // See axial_to_pixel for comment on why this value
        let size_y = 18.66666666666666666;

        let pos = Vec2::new(
            pos.x / size_x,
            pos.y / size_y,
        );

        let b0 = f32::sqrt(3.0) / 3.0;
        let b1 = -1.0 / 3.0;
        let b2 = 0.0;
        let b3 = 2.0 / 3.0;

        let q: f32 = b0 * pos.x + b1 * pos.y;
        let r: f32 = b2 * pos.x + b3 * pos.y;
        (q, r)
    }

    /// Returns a hex in offset coords
    #[allow(dead_code)]
    pub fn pixel_to_hex(&mut self, pos: Vec2<f32>) -> Option<(i32, i32)> {
        let mut tallest_height: Option<(u8, i32, i32)> = None;

        for height in 0..=self.tallest {
            let height_offset = height as f32 * FLOOR_DEPTH_STEP;

            let (q, r) = self.pixel_to_hex_raw(pos.clone(), height_offset);

            let (q, r, s) = (q, r, -r -q);

            let (x, y, _) = cube_round(q, r, s);
    
            if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32 {
                continue;
            }

            let tile = &self.tiles[self.width * y as usize + x as usize];
            let tile_height = tile.wall_height;

            if tile_height != height {
                continue;
            }
            if tallest_height.is_none() || tile_height > tallest_height.unwrap().0 {
                tallest_height = Some((tile_height, x, y));
            }
        }

        if let Some((_, x, y)) = tallest_height {
            return Some((x, y));
        }
        None
    }

    #[allow(dead_code)]
    pub fn axial_to_pixel(&mut self, q: i32, r: i32) -> (f32, f32) {
        let (q, r) = (q as f32, r as f32);

        let size_x = FLOOR_WIDTH / f32::sqrt(3.0);
        // this value is derived by solving for X in:
        // FLOOR_VERT_STEP * R = X * (3.0 / 2.0 * R) 
        // R can be 1 so we can simplify to:
        // FLOOR_VERT_STEP = X * 1.5
        // X = FLOOR_VERT_STEP / 1.5
        let size_y = 18.66666666666666666;

        let x = size_x * (f32::sqrt(3.0) * q + f32::sqrt(3.0) / 2.0 * r);
        let y = size_y * (3.0 / 2.0 * r);
        (
            x + 18. + self.position.x,
            y + 18. + self.position.y,
        )
    }
}

#[allow(dead_code)]
pub fn cube_to_offset(q: i32, r: i32) -> (i32, i32) {
    let col = q + (r - (r & 1)) / 2;
    let row = r;

    (col, row)
}

#[allow(dead_code)]
pub fn offset_to_cube(off_x: i32, off_y: i32) -> (i32, i32, i32) {
    let x = off_x - (off_y - (off_y as i32 & 1)) / 2;
    let z = off_y;
    let y = -x-z;
    
    (x, y, z)
}

pub fn cube_round(q: f32, r: f32, s: f32) -> (i32, i32, i32) {
    let mut qi = q.round() as i32;
    let mut ri = r.round() as i32;
    let mut si = s.round() as i32;

    let q_diff = f64::abs(qi as f64 - q as f64);
    let r_diff = f64::abs(ri as f64 - r as f64);
    let s_diff = f64::abs(si as f64 - s as f64);

    if q_diff > r_diff && q_diff > s_diff {
        qi = -ri - si;
    } else if r_diff > s_diff {
        ri = -qi - si;
    } else {
        si = -qi - ri;
    }

    (qi, ri, si)
}

pub fn render_hex_map(mut draw_buffer: UniqueViewMut<DrawBuffer>, drawables: NonSendSync<UniqueViewMut<Drawables>>, mut map: UniqueViewMut<HexMap>) {
    draw_buffer.new_command_pool(true);
    let command_pool = draw_buffer.get_command_pool();

    let (q, r) = map.pixel_to_hex_raw(Vec2::zero(), 0.);

    let startx = (q - 40.0)
        .max(0.0).min(map.width as f32 - 1.0) as usize;
    let endx = (q + 40.0)
        .max(0.0).min(map.width as f32 - 1.0) as usize;
    let starty = (r - 20.0)
        .max(0.0).min(map.height as f32 - 1.0) as usize;
    let endy = (r + 20.0)
        .max(0.0).min(map.height as f32 - 1.0) as usize;

    let (top_tex, wall_tex, brick_tex, brick_floor_tex, grown_tex, tilled_tex) = 
        (
            drawables.alias[textures::FLOOR], 
            drawables.alias[textures::WALL], 
            drawables.alias[textures::WALL_BRICK], 
            drawables.alias[textures::FLOOR_BRICK],
            drawables.alias[textures::FLOOR_GROWN],
            drawables.alias[textures::FLOOR_TILLED],
        );

    for height in 0..=MAX_BRICK_HEIGHT {
        let mut wall_buffer: Vec<DrawCommand> = Vec::with_capacity(1024);
        let mut wall_brick_buffer: Vec<DrawCommand> = Vec::with_capacity(1024);
        let mut top_buffer: Vec<DrawCommand> = Vec::with_capacity(1024);
        let mut top_brick_buffer: Vec<DrawCommand> = Vec::with_capacity(1024);
        let mut top_tilled_buffer: Vec<DrawCommand> = Vec::with_capacity(1024);
        let mut top_grown_buffer: Vec<DrawCommand> = Vec::with_capacity(1024);
        for y in starty..=endy {
            for x in startx..=endx {
                let tile = &map.tiles[map.width * y + x];
                if tile.wall_height < height {
                    continue;
                }

                let (draw_x, draw_y) = {
                    let offset_x = (FLOOR_WIDTH / 2.0) * y as f32;
                    let mut x = FLOOR_WIDTH * x as f32;
                    x += offset_x;
                    (
                        x,
                        (y as i32) as f32 * (FLOOR_VERT_STEP)
                    )
                };
                let (draw_x, draw_y) =
                    (
                        draw_x + map.position.x,
                        draw_y + map.position.y,
                    );
                
                if height <= tile.ground_height && height != 0 {
                    render_hex_walls(&mut wall_buffer, draw_x, draw_y, height, wall_tex);
                }
                else if height > tile.ground_height && height <= tile.wall_height {
                    render_hex_bricks(&mut wall_brick_buffer, draw_x, draw_y, height, brick_tex);
                }

                if tile.is_grown && height == tile.ground_height {
                    render_hex_top(&mut top_grown_buffer, draw_x, draw_y, tile.ground_height, grown_tex, Color::WHITE);
                }
                else if tile.is_tilled && height == tile.ground_height {
                    render_hex_top(&mut top_tilled_buffer, draw_x, draw_y, tile.ground_height, tilled_tex, Color::WHITE);
                }
                else if height == tile.ground_height && height == tile.wall_height {
                    render_hex_top(&mut top_buffer, draw_x, draw_y, tile.ground_height, top_tex, Color::WHITE);
                }
                else if height == tile.wall_height && height != tile.ground_height {
                    render_hex_brick_top(&mut top_brick_buffer, draw_x, draw_y, tile.wall_height, brick_floor_tex, Color::WHITE);
                }
            }
        }
        command_pool.commands.extend(&wall_buffer);
        command_pool.commands.extend(&wall_brick_buffer);
        command_pool.commands.extend(&top_buffer);
        command_pool.commands.extend(&top_brick_buffer);
        command_pool.commands.extend(&top_tilled_buffer);
        command_pool.commands.extend(&top_grown_buffer);
    }
    
    // Draw dots at hex centers
    /*let marker_tex = drawables.alias[textures::MARKER];
    for y_tile in starty..=endy {
        for x_tile in startx..=endx {
            let (x, y) = map.axial_to_pixel(x_tile as i32, y_tile as i32);
            let tile = &map.tiles[map.width * y_tile + x_tile];

            draw_buffer.draw(
                DrawCommand::new(marker_tex)
                    .position(Vec3::new(
                        x - 2.0, y - 2.0, tile.wall_height as f32 * FLOOR_DEPTH_STEP 
                    ))
                    .draw_iso(true)
            );
        }
    }*/

    draw_buffer.end_command_pool();
}

pub fn render_hex_top(draw_buffer: &mut Vec<DrawCommand>, x: f32, y: f32, height: u8, texture: u64, color: Color) {
    let mut draw_command = create_floor_draw_cmd(x, y, height as f32 * FLOOR_DEPTH_STEP, height, texture); 
    if color != Color::WHITE {
        draw_command = draw_command.color(color);
    }
    draw_buffer.push(draw_command);
}

fn create_floor_draw_cmd(x: f32, y: f32, height: f32, color: u8, texture: u64) -> DrawCommand {
    let color = 
        if color == 0 {
            let v = 0.55;
            Color::rgba(v, v, v, 1.0)
        } else if color == 1 {
            let v = 0.8;
            Color::rgba(v, v, v, 1.0)
        } else {
            let v = 0.95;
            Color::rgba(v, v, v, 1.0)
        };

    DrawCommand::new(texture)
        .position(Vec3::new(x, y, height))
        .draw_layer(draw_layers::FLOOR)
        .draw_iso(true)
        .color(color)
}

pub fn render_hex_brick_top(draw_buffer: &mut Vec<DrawCommand>, x: f32, y: f32, height: u8, texture: u64, color: Color) {
    let mut draw_command = create_brick_floor_draw_cmd(x, y, height as f32 * FLOOR_DEPTH_STEP, height, texture); 
    if color != Color::WHITE {
        draw_command = draw_command.color(color);
    }
    draw_buffer.push(draw_command);
}

fn create_brick_floor_draw_cmd(x: f32, y: f32, height: f32, color: u8, texture: u64) -> DrawCommand {
    let color = 
        if color == 1 {
            let v = 0.65;
            Color::rgba(v, v, v, 1.0)
        } else if color == 2 {
            let v = 0.8;
            Color::rgba(v, v, v, 1.0)
        } else if color == 3 {
            let v = 0.9;
            Color::rgba(v, v, v, 1.0)
        } else {
            let v = 1.0;
            Color::rgba(v, v, v, 1.0)
        };

    DrawCommand::new(texture)
        .position(Vec3::new(x, y, height))
        .draw_layer(draw_layers::FLOOR)
        .draw_iso(true)
        .color(color)
}

pub fn render_hex_walls(draw_buffer: &mut Vec<DrawCommand>, x: f32, y: f32, height: u8, wall_tex: u64) {
    let start_height = height as f32 * FLOOR_DEPTH_STEP - WALL_VERT_OFFSET;
    let color = 
        if height % 2 == 1 {
            1
        } else {
            2
        };
    draw_buffer.push(
        create_wall_draw_cmd(x, y, start_height, color, wall_tex)
    );
}

fn create_wall_draw_cmd(x: f32, y: f32, height: f32, color: u8, texture: u64) -> DrawCommand {
    let color =
        if color == 1 {
            let v = 0.5;
            Color::rgba(v, v, v, 1.0)
        } else if color == 2{
            let v = 0.7;
            Color::rgba(v, v, v, 1.0)
        } else {
            let v = 1.0;
            Color::rgba(v, v, v, 1.0)
        };

    DrawCommand::new(texture)
        .position(Vec3::new(x, y, height))
        .draw_layer(draw_layers::WALL)
        .draw_iso(true)
        .color(color)
}

pub fn render_hex_bricks(draw_buffer: &mut Vec<DrawCommand>, x: f32, y: f32, height: u8, brick_tex: u64) {
    let start_height = height as f32 * FLOOR_DEPTH_STEP - WALL_VERT_STEP;
    draw_buffer.push(
        create_wall_brick_draw_cmd(x, y, start_height, height, brick_tex)
    );
}

fn create_wall_brick_draw_cmd(x: f32, y: f32, height: f32, color: u8, texture: u64) -> DrawCommand {
    let color =
        if color == 1 {
            let v = 0.3;
            Color::rgba(v, v, v, 1.0)
        } else if color == 2 {
            let v = 0.55;
            Color::rgba(v, v, v, 1.0)
        } else if color == 3 {
            let v = 0.7;
            Color::rgba(v, v, v, 1.0)
        } else {
            let v = 0.80;
            Color::rgba(v, v, v, 1.0)
        };

    DrawCommand::new(texture)
        .position(Vec3::new(x, y, height))
        .draw_layer(draw_layers::WALL)
        .draw_iso(true)
        .color(color)
}