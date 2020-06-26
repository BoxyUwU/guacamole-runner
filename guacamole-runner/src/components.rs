use vermarine_lib::components::Transform;

pub struct Player();

pub enum Direction {
    Up,
    Down,
}

pub struct Plane {
    pub direction: Direction, 
}

impl Plane {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
        }
    }
}

#[derive(Clone)]
pub struct Collider {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Collider {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn intersects(this: &Collider, this_pos: &Transform, other: &Collider, other_pos: &Transform) -> bool {
        if  (this.xmin(this_pos) >= other.xmin(other_pos) && this.xmin(this_pos) <= other.xmax(other_pos)) ||
            (this.xmax(this_pos) >= other.xmin(other_pos) && this.xmax(this_pos) <= other.xmax(other_pos)) || 
            (this.xmin(this_pos) <= other.xmin(other_pos) && this.xmax(this_pos) >= other.xmax(other_pos)) {
                if  (this.ymin(this_pos) >= other.ymin(other_pos) && this.ymin(this_pos) <= other.ymax(other_pos)) ||
                    (this.ymax(this_pos) >= other.ymin(other_pos) && this.ymax(this_pos) <= other.ymax(other_pos)) || 
                    (this.ymin(this_pos) <= other.ymin(other_pos) && this.ymax(this_pos) >= other.ymax(other_pos)) {
                        return true;
                    }
            }
        false
    }

    pub fn xmin(&self, pos: &Transform) -> i32 {
        self.x + pos.x as i32
    }

    pub fn xmax(&self, pos: &Transform) -> i32 {
        self.x + self.width as i32 + pos.x as i32
    }

    pub fn ymin(&self, pos: &Transform) -> i32 {
        self.y + pos.y as i32
    }

    pub fn ymax(&self, pos: &Transform) -> i32 {
        self.y + self.height as i32 + pos.y as i32
    }
}

#[derive(Debug)]
pub struct Height(pub f32);

pub struct Points(pub u32);

impl Points {
    pub fn new() -> Self {
        Self(0)
    }
}