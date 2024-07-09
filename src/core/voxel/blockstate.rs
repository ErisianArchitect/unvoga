use super::{coord::Coord, direction::Direction};

pub struct BlockState {
    name: String,
}

pub enum PropertyField {
    Null,
    Int(i64),
    Float(f32),
    Bool(bool),
    String(String),
    // Coord(Coord),
    Direction(Direction),
    
}

pub struct BlockProperty {
    name: String,
    value: PropertyField
}