use crate::comp::pos::{Heading, LocalPosition};
use bevy_ecs_macros::Component;

#[derive(Component, Default)]
pub struct Synchronize {
    pub movement: Option<MovementUpdate>,
    pub damage: Vec<DamageReceived>,
    pub despawned: Vec<u32>,
}

pub enum MovementUpdate {
    StartMove(LocalPosition, LocalPosition),
    StartMoveTowards(LocalPosition, Heading),
    StopMove(LocalPosition, Heading),
    Turn(Heading),
}

impl Synchronize {
    pub fn clear(&mut self) {
        self.movement = None;
        self.damage.clear();
        self.despawned.clear();
    }
}

pub struct DamageReceived {
    pub amount: u32,
    pub crit: bool,
}