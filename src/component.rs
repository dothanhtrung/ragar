use bevy::prelude::*;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct ExchangeData {
    pub mouse_x: u32,
    pub mouse_y: u32,
    pub new_food_x: u32,
    pub new_food_y: u32,
}
