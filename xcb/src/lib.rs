#[macro_use]
extern crate bitflags;

pub mod connection;
pub mod event;
pub mod result;
pub mod window;

#[derive(Debug, Copy, Clone)]
pub struct Rectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}
