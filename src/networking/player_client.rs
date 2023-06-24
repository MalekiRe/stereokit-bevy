use bevy_app::App;
use bevy_ecs::prelude::{Added, Changed, Commands, Entity, Query, Res, ResMut, With, Without, World};
use bevy_ecs::system::SystemState;
use bevy_quinnet::client::Client;
use bevy_quinnet::shared::channel::ChannelType;
use bevy_quinnet::shared::channel::ChannelType::{OrderedReliable, Unreliable};
use bevy_quinnet::shared::ClientId;
use bevy_transform::prelude::Transform;
use leknet::{ClientEntity, ClientMessage, EntityMap, LekClient, Networked, ServerEntity, TypeName};
use crate::networking::{IgnorePlayerAdd, ModelData, ModelData2, Player};
use serde::{Serialize, Deserialize};
use crate::networking::player_server::PlayerMsgServer;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerMsgClient {
    PlayerAdded(ServerEntity, Transform),
    PlayerChanged(ServerEntity, Transform),
    EntityMap(ServerEntity, ClientEntity),
    GetAllPlayers(ClientId),
}

impl TypeName for PlayerMsgClient {
    fn get_type_name() -> String {
        "stereokit_bevy::networking::PlayerMsgClient".to_string()
    }
}

impl ClientMessage for PlayerMsgClient {
    fn client(self, world: &mut World) {
        match self {
            PlayerMsgClient::PlayerAdded(server_entity, player_position) => {
                player_added_msg(world, server_entity, player_position);
            }
            PlayerMsgClient::PlayerChanged(server_entity, player_position) => {
                player_changed_msg(world, server_entity, player_position);
            }
            PlayerMsgClient::EntityMap(server_entity, client_entity) => {
                let mut system_state: SystemState<ResMut<EntityMap>> = SystemState::new(world);
                let mut entity_map: ResMut<EntityMap> = system_state.get_mut(world);
                entity_map.0.insert(client_entity, server_entity);
            }
            PlayerMsgClient::GetAllPlayers(client_id) => {
                get_all_players_msg(world, client_id);
            }
        }
    }

    fn _client(world: &mut World, msg_bytes: &[u8]) {
        bincode::deserialize::<Self>(msg_bytes).unwrap().client(world)
    }

    fn channel_type(&self) -> ChannelType {
        match self {
            PlayerMsgClient::PlayerAdded(_, _) => OrderedReliable,
            PlayerMsgClient::PlayerChanged(_, _) => Unreliable,
            PlayerMsgClient::EntityMap(_, _) => OrderedReliable,
            PlayerMsgClient::GetAllPlayers(_) => OrderedReliable,
        }
    }

    fn plugin(app: &mut App) {
        app.add_system(player_added);
        app.add_system(player_changed);
    }
}

fn get_all_players_msg(world: &mut World, client_id: ClientId) {
    let mut system_state: SystemState<(
        Query<
            (Entity, &Player, &Transform),
            (With<Networked>),
        >,
        ResMut<Client>,
        Res<EntityMap>,
    )> = SystemState::new(world);
    let (query, mut client, entity_map) = system_state.get_mut(world);
    let mut client: ResMut<Client> = client;
    let entity_map: Res<EntityMap> = entity_map;
    let mut players = vec![];
    for (entity, _, transform) in query.iter() {
        let server_entity = match entity_map.get_by_left(&ClientEntity(entity)) {
            None => continue,
            Some(server_entity) => server_entity.clone(),
        };
        players.push((
            server_entity,
            *transform,
        ))
    }
    client
        .connection_mut()
        .send_lek_msg(PlayerMsgServer::AllPlayerData(client_id, players))
        .unwrap();
}
fn player_changed_msg(world: &mut World, server_entity: ServerEntity, transform: Transform) {
    let mut client_entity = None;
    {
        let mut system_state: SystemState<ResMut<EntityMap>> = SystemState::new(world);
        let mut entity_map = system_state.get_mut(world);
        client_entity = entity_map.get_by_right(&server_entity).map(|a| a.clone());
    }
    if let Some(client_entity) = client_entity {
        let mut world_entity = world.entity_mut(client_entity.0);
        *world_entity.get_mut().unwrap() = transform;
    }
}
fn player_added_msg(world: &mut World, server_entity: ServerEntity, transform: Transform) {
    let mut system_state: SystemState<(ResMut<EntityMap>, Commands)> =
        SystemState::new(world);
    let (entity_map, commands) = system_state.get_mut(world);
    let mut entity_map: ResMut<EntityMap> = entity_map;
    let mut commands: Commands = commands;
    let client_entity = ClientEntity(
        commands
            .spawn((Player, transform, Networked))
            .insert(IgnorePlayerAdd)
            .id(),
    );
    entity_map.insert(client_entity, server_entity);
    system_state.apply(world);
}

fn player_added(
    query: Query<
        (Entity, &Transform, &Player),
        (Added<Networked>, Without<IgnorePlayerAdd>),
    >,
    mut client: ResMut<Client>,
) {
    if let Some(connection) = client.get_connection_mut() {
        for (entity, transform, _) in query.iter() {
            connection
                .send_lek_msg(PlayerMsgServer::PlayerAdded(
                    ClientEntity(entity),
                    *transform,
                ))
                .unwrap()
        }
    }
}

fn player_changed(
    query: Query<
        (Entity, &Transform, &Player),
        (
            Changed<Transform>,
            Without<IgnorePlayerAdd>,
            With<Networked>,
        ),
    >,
    mut client: ResMut<Client>,
    entity_map: Res<EntityMap>,
) {
    if let Some(connection) = client.get_connection_mut() {
        for (entity, transform, _) in query.iter() {
            if let Some(server_entity) = entity_map.get_by_left(&ClientEntity(entity)) {
                connection
                    .send_lek_msg(PlayerMsgServer::PlayerChanged(
                        *server_entity,
                        *transform,
                    ))
                    .unwrap()
            }
        }
    }
}