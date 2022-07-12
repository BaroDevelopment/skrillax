pub(crate) mod drop;
pub(crate) mod monster;
pub(crate) mod player;
pub(crate) mod pos;
pub(crate) mod stats;
pub(crate) mod sync;
pub(crate) mod visibility;

use crate::db::user::ServerUser;
use crate::login::character_loader::Character;
use crate::population::capacity::PlayingToken;
use bevy_ecs::prelude::*;
use silkroad_network::stream::Stream;
use silkroad_protocol::{ClientPacket, ServerPacket};
use std::collections::VecDeque;
use std::time::Instant;
use tokio::sync::oneshot::Receiver;

#[derive(Component)]
pub(crate) struct Login;

#[derive(Component)]
pub(crate) struct LastAction(pub(crate) Instant);

#[derive(Component, Default)]
pub(crate) struct CharacterSelect {
    pub(crate) characters: Option<Vec<Character>>,
    pub(crate) character_receiver: Option<Receiver<Vec<Character>>>,
    pub(crate) character_name_check: Option<Receiver<bool>>,
    pub(crate) character_delete_task: Option<Receiver<bool>>,
    pub(crate) checked_name: Option<String>,
    pub(crate) character_create: Option<Receiver<()>>,
    pub(crate) character_restore: Option<Receiver<bool>>,
}

#[derive(Component)]
pub(crate) struct Client(pub(crate) Stream, pub(crate) VecDeque<ClientPacket>);

impl Client {
    pub fn send<T: Into<ServerPacket>>(&self, packet: T) {
        // We specifically ignore the error here because we'll handle the client being disconnected
        // at the end of the game tick. This means we might do some unnecessary things, but that's ok
        // for now. The upside is that this means there's a single point where we handle such errors.
        let _ = self.0.send(packet);
    }
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GameEntity {
    pub unique_id: u32,
    pub ref_id: u32,
}

#[derive(Component)]
pub(crate) struct Playing(pub(crate) ServerUser, pub(crate) PlayingToken);

#[derive(Component)]
pub(crate) struct Health {
    pub current_health: usize,
    pub max_health: usize,
}

impl Health {
    pub fn new(max_health: usize) -> Self {
        Self {
            current_health: max_health,
            max_health,
        }
    }
}

#[derive(Hash, Copy, Clone, Eq, PartialEq)]
pub struct EntityReference(pub Entity, pub(crate) GameEntity);
