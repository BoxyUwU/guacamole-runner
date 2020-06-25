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