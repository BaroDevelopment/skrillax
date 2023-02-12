use crate::agent::states::StateTransitionQueue;
use crate::agent::Agent;
use crate::comp::monster::{Monster, MonsterBundle, RandomStroll, SpawnedBy};
use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::comp::pos::Position;
use crate::comp::sync::Synchronize;
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health};
use crate::ext::EntityIdPool;
use crate::game::drop::SpawnDrop;
use crate::input::PlayerInput;
use crate::world::{EntityLookup, WorldData};
use bevy_ecs::change_detection::ResMut;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::prelude::{Commands, Query, Res};
use silkroad_data::{ObjectConsumable, ObjectConsumableCurrency, ObjectItem, ObjectType};
use silkroad_game_base::{Item, ItemTypeData};
use silkroad_protocol::chat::{
    ChatErrorCode, ChatMessageResponse, ChatMessageResult, ChatSource, ChatTarget, ChatUpdate,
};
use silkroad_protocol::gm::{GmCommand, GmResponse};
use silkroad_protocol::world::{BodyState, UpdatedState};
use std::time::Duration;
use tracing::debug;

pub(crate) fn handle_chat(
    mut query: Query<(&Client, &GameEntity, &PlayerInput, &Visibility, &Player)>,
    lookup: Res<EntityLookup>,
    others: Query<(&Client, &Player)>,
) {
    for (client, game_entity, input, visibility, player) in query.iter_mut() {
        for message in input.chat.iter() {
            debug!(id = ?client.0.id(), "Received chat message: {} @ {}", message.message, message.index);
            match message.target {
                ChatTarget::All => {
                    visibility
                        .entities_in_radius
                        .iter()
                        .filter_map(|entity| others.get(entity.0).ok())
                        .for_each(|(client, _)| {
                            client.send(ChatUpdate::new(
                                ChatSource::all(game_entity.unique_id),
                                message.message.clone(),
                            ));
                        });
                    client.send(ChatMessageResponse::new(
                        ChatMessageResult::Success,
                        message.target,
                        message.index,
                    ));
                },
                ChatTarget::AllGm => {
                    if player.character.gm {
                        others
                            .iter()
                            .filter(|(_, player)| player.character.gm)
                            .filter(|(_, other)| other.user.id != player.user.id)
                            .for_each(|(client, _)| {
                                client.send(ChatUpdate::new(
                                    ChatSource::allgm(game_entity.unique_id),
                                    message.message.clone(),
                                ));
                            });
                        client.send(ChatMessageResponse::new(
                            ChatMessageResult::Success,
                            message.target,
                            message.index,
                        ));
                    } else {
                        client.send(ChatMessageResponse::new(
                            ChatMessageResult::error(ChatErrorCode::InvalidTarget),
                            message.target,
                            message.index,
                        ));
                    }
                },
                ChatTarget::PrivateMessage => {
                    match message
                        .recipient
                        .as_ref()
                        .and_then(|target| lookup.get_entity_for_name(target))
                        .and_then(|entity| others.get(entity).ok())
                    {
                        Some((other, _)) => {
                            other.send(ChatUpdate::new(
                                ChatSource::privatemessage(player.character.name.clone()),
                                message.message.clone(),
                            ));
                            client.send(ChatMessageResponse::new(
                                ChatMessageResult::Success,
                                message.target,
                                message.index,
                            ));
                        },
                        None => {
                            client.send(ChatMessageResponse::new(
                                ChatMessageResult::error(ChatErrorCode::InvalidTarget),
                                message.target,
                                message.index,
                            ));
                        },
                    }
                },
                ChatTarget::NPC | ChatTarget::Notice | ChatTarget::Global => {
                    client.send(ChatMessageResponse::new(
                        ChatMessageResult::error(ChatErrorCode::InvalidTarget),
                        message.target,
                        message.index,
                    ));
                },
                _ => {},
            }
        }
    }
}

pub(crate) fn handle_gm_commands(
    mut query: Query<(Entity, &Client, &Position, &PlayerInput, &mut Synchronize)>,
    mut commands: Commands,
    mut id_pool: ResMut<EntityIdPool>,
    mut item_spawn: EventWriter<SpawnDrop>,
) {
    for (entity, client, position, input, mut sync) in query.iter_mut() {
        if let Some(ref command) = input.gm {
            match command {
                GmCommand::SpawnMonster { ref_id, amount, rarity } => {
                    let character_def = WorldData::characters().find_id(*ref_id).unwrap();
                    for _ in 0..(*amount) {
                        let unique_id = id_pool.request_id().unwrap();
                        let bundle = MonsterBundle {
                            monster: Monster {
                                target: None,
                                rarity: *rarity,
                            },
                            health: Health::new(character_def.hp),
                            position: position.clone(),
                            entity: GameEntity {
                                unique_id,
                                ref_id: *ref_id,
                            },
                            visibility: Visibility::with_radius(100.),
                            spawner: SpawnedBy { spawner: entity },
                            navigation: Agent::from_character_data(character_def),
                            sync: Default::default(),
                            stroll: RandomStroll::new(position.location.to_location(), 100., Duration::from_secs(1)),
                            state_queue: StateTransitionQueue::default(),
                        };
                        commands.spawn(bundle);
                    }
                    client.send(GmResponse::success_message(format!(
                        "Spawned {} of {}",
                        *amount, character_def.common.id
                    )));
                },
                GmCommand::MakeItem { ref_id, upgrade } => {
                    let item = WorldData::items().find_id(*ref_id).unwrap();
                    let object_type = ObjectType::from_type_id(&item.common.type_id).unwrap();
                    let item_type = if matches!(object_type, ObjectType::Item(ObjectItem::Equippable(_))) {
                        ItemTypeData::Equipment {
                            upgrade_level: *upgrade,
                        }
                    } else if matches!(
                        object_type,
                        ObjectType::Item(ObjectItem::Consumable(ObjectConsumable::Currency(
                            ObjectConsumableCurrency::Gold
                        )))
                    ) {
                        ItemTypeData::Gold { amount: 1 }
                    } else {
                        ItemTypeData::Consumable { amount: 1 }
                    };
                    item_spawn.send(SpawnDrop::new(
                        Item {
                            reference: item,
                            variance: None,
                            type_data: item_type,
                        },
                        position.location.to_location(),
                        None,
                    ));
                    client.send(GmResponse::success_message(format!("Dropped 1 of {}", item.common.id)));
                },
                GmCommand::Invincible => {
                    sync.state.push(UpdatedState::Body(BodyState::GMInvincible));
                    client.send(GmResponse::success_message("Enabled invincibility".to_string()));
                },
                GmCommand::Invisible => {
                    sync.state.push(UpdatedState::Body(BodyState::GMInvisible));
                    client.send(GmResponse::success_message("Enabled invisibility".to_string()));
                },
                _ => {},
            }
        }
    }
}