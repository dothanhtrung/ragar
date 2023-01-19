use crate::component::ExchangeData;
use bevy::math::Vec2;
use bevy::prelude::{default, In, Res, Windows};
use bevy_ggrs::{ggrs, PlayerInputs};

use crate::network::GgrsConfig;

pub fn input(_: In<ggrs::PlayerHandle>, windows: Res<Windows>) -> ExchangeData {
    let mut input = ExchangeData { ..default() };
    let window = windows.get_primary().unwrap();
    if let Some(position) = window.cursor_position() {
        input.mouse_x = position.x as u32;
        input.mouse_y = position.y as u32;
    }
    input
}

pub fn moving(inputs: Res<PlayerInputs<GgrsConfig>>) {}
