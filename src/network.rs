use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_ggrs::ggrs;
use matchbox_socket::WebRtcSocket;

use crate::component::ExchangeData;

pub struct GgrsConfig;

const NUM_PLAYERS: usize = 2;

impl ggrs::Config for GgrsConfig {
    type Input = ExchangeData;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are strings
    type Address = String;
}

#[derive(Resource)]
pub struct Session {
    socket: Option<WebRtcSocket>,
}

pub fn start_matchbox_socket(mut commands: Commands) {
    let room_url = format!("ws://127.0.0.1:3536/ragar?next={}", NUM_PLAYERS);
    info!("Connecting to matchbox server {}", room_url);

    let (socket, message_loop) = WebRtcSocket::new(room_url.as_str());
    IoTaskPool::get().spawn(message_loop).detach();

    commands.insert_resource(Session {
        socket: Some(socket),
    })
}

pub fn wait_for_player(mut session: ResMut<Session>) {
    // if there is no socket, we already started the game
    let Some(socket) = &mut session.socket else { return; };

    // Check for new connection
    socket.accept_new_connections();
    let players = socket.players();
    if players.len() < NUM_PLAYERS {
        return; // wait for more players
    }
    info!("All players has joined")
}
