use crate::agent::states::StateTransitionQueue;
use crate::agent::{Agent, MovementState};
use crate::comp::damage::DamageReceiver;
use crate::comp::exp::{Experienced, Leveled, SP};
use crate::comp::inventory::PlayerInventory;
use crate::comp::pos::Position;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health, Mana};
use crate::db::character::CharacterData;
use crate::db::user::ServerUser;
use crate::game::mind::Mind;
use crate::input::PlayerInput;
use bevy_ecs::prelude::*;
use silkroad_game_base::{Character, Race, SpawningState, Stats};

#[derive(Component)]
pub(crate) struct Player {
    pub user: ServerUser,
    pub character: Character,
}

impl Player {
    fn from_db_character(data: &CharacterData) -> Character {
        Character {
            id: data.id as u32,
            name: data.charname.clone(),
            race: Race::Chinese,
            scale: data.scale as u8,
            level: data.level as u8,
            max_level: data.max_level as u8,
            exp: data.exp as u64,
            sp: data.sp as u32,
            sp_exp: data.sp_exp as u32,
            stats: Stats::new_preallocated(data.strength as u16, data.intelligence as u16),
            stat_points: data.stat_points as u16,
            current_hp: data.current_hp as u32,
            current_mp: data.current_mp as u32,
            berserk_points: data.berserk_points as u8,
            gold: data.gold as u64,
            beginner_mark: data.beginner_mark,
            gm: data.gm,
            state: SpawningState::Loading,
            masteries: Vec::new(),
            skills: Vec::new(),
        }
    }

    pub fn from_db_data(user: ServerUser, character: &CharacterData) -> Self {
        let char = Self::from_db_character(character);
        Player { user, character: char }
    }
}

#[derive(Component)]
pub(crate) struct Buffed {
    // pub buffs: Vec<Buff>
}

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    player: Player,
    inventory: PlayerInventory,
    game_entity: GameEntity,
    agent: Agent,
    pos: Position,
    buff: Buffed,
    visibility: Visibility,
    input: PlayerInput,
    state_queue: StateTransitionQueue,
    speed: MovementState,
    damage_receiver: DamageReceiver,
    health: Health,
    mana: Mana,
    level: Leveled,
    sp: SP,
    exp: Experienced,
    mind: Mind,
}

impl PlayerBundle {
    pub fn new(
        player: Player,
        game_entity: GameEntity,
        inventory: PlayerInventory,
        agent: Agent,
        pos: Position,
        visibility: Visibility,
    ) -> Self {
        let max_hp = player.character.max_hp();
        let max_mana = player.character.max_mp();
        let sp = player.character.sp;
        let sp_exp = player.character.sp_exp;
        let exp = player.character.exp;
        let level = player.character.level;
        Self {
            player,
            game_entity,
            inventory,
            agent,
            pos,
            buff: Buffed {},
            visibility,
            input: Default::default(),
            state_queue: Default::default(),
            speed: MovementState::default_player(),
            damage_receiver: DamageReceiver::default(),
            health: Health::new(max_hp),
            mana: Mana::with_max(max_mana),
            sp: SP::new(sp),
            level: Leveled::new(level),
            exp: Experienced::new(exp, sp_exp as u64),
            mind: Mind::default(),
        }
    }
}
