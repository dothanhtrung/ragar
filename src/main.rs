use bevy::prelude::*;
use bevy::window::{WindowDescriptor, WindowPlugin};
use bevy_ggrs::*;

mod component;
mod network;
mod system;

use crate::network::*;
use crate::system::*;

fn main() {
    let mut app = App::new();
    GGRSPlugin::<network::GgrsConfig>::new()
        .with_input_system(input::input)
        .with_rollback_schedule(Schedule::default().with_stage(
            "ROLLBACK_STAGE",
            SystemStage::single_threaded().with_system(input::moving),
        ))
        .register_rollback_component::<Transform>()
        .build(&mut app);

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        window: WindowDescriptor {
            // fill the entire browser window
            fit_canvas_to_parent: true,
            ..default()
        },
        ..default()
    }))
    .add_startup_system(spawn::setup)
    .add_startup_system(start_matchbox_socket)
    .add_system(wait_for_player)
    .run();
}
