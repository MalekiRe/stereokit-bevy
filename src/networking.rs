use crate::{model_draw, ModelBundle, ModelInfo};
use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Added, Entity, EventReader, NonSend, Or, Query, ReflectComponent, Res, ResMut, With, Without, World};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, SystemState};
use bevy_quinnet::client::Client;
use bevy_quinnet::server::{ConnectionEvent, Server};
use bevy_quinnet::shared::ClientId;
use bevy_reflect::{FromReflect, Reflect};
use bevy_transform::prelude::Transform;
use bimap::BiHashMap;
use glam::Vec3;
use leknet::{ClientEntity, EntityMap, LekClient, LekServer, MessageMap, Networked, ServerEntity};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::transmute;
use bevy_quinnet::shared::channel::ChannelId;
use bevy_transform::systems::sync_simple_transforms;
use stereokit::{Color128, Handed, Material, Model, RenderLayer, Settings, Sk, SkDraw, StereoKitMultiThread};

#[derive(Component)]
pub struct IgnoreModelAdd;
#[derive(Component)]
pub struct IgnoreModelChanged;

#[derive(Clone, Debug, Serialize, Deserialize, Component)]
pub enum ModelMsg {
    Client(ModelMsgClient),
    Server(ModelMsgServer),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelData {
    model_info: ModelInfo,
    transform: Transform,
    color128: Color128,
    render_layer: RenderLayer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelData2 {
    transform: Transform,
    color128: Color128,
    render_layer: RenderLayer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModelMsgClient {
    ModelAdded(ClientEntity, ModelData),
    ModelChanged(ServerEntity, ModelData2),
    AllModelData(ClientId, Vec<(ServerEntity, ModelData)>),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ModelMsgServer {
    ModelAdded(ServerEntity, ModelData),
    ModelChanged(ServerEntity, ModelData2),
    EntityMap(ServerEntity, ClientEntity),
    GetAllModelData(ClientId),
}

impl MessageMap for ModelMsg {
    fn server(self, world: &mut World, client_id: ClientId) {
        let msg = match self {
            ModelMsg::Client(msg) => msg,
            _ => panic!("client msg wasn't a client message"),
        };
        match msg {
            ModelMsgClient::ModelAdded(client_entity, model_data) => {
                let mut system_state: SystemState<(ResMut<Server>, Commands)> =
                    SystemState::new(world);
                let (mut server, mut commands) = system_state.get_mut(world);
                let mut server: ResMut<Server> = server;
                let mut commands: Commands = commands;
                let server_entity = ServerEntity(commands.spawn_empty().id());
                let endpoint = server.get_endpoint_mut().expect("no server endpoint");
                endpoint
                    .send_lek_msg(
                        client_id,
                        ModelMsg::Server(ModelMsgServer::EntityMap(server_entity, client_entity)),
                    )
                    .unwrap();
                for client_id2 in endpoint.clients() {
                    if client_id2 == client_id {
                        continue;
                    }
                    endpoint
                        .send_lek_msg(
                            client_id2.clone(),
                            ModelMsg::Server(ModelMsgServer::ModelAdded(
                                server_entity,
                                model_data.clone(),
                            )),
                        )
                        .unwrap()
                }
                system_state.apply(world);
            }
            ModelMsgClient::ModelChanged(server_entity, model_data) => {
                let mut system_state: SystemState<ResMut<Server>> = SystemState::new(world);
                let mut server = system_state.get_mut(world);
                let endpoint = server.endpoint_mut();
                for client_id2 in endpoint.clients() {
                    if client_id2 == client_id {
                        continue;
                    }
                    endpoint
                        .send_lek_msg(
                            client_id2.clone(),
                            ModelMsg::Server(ModelMsgServer::ModelChanged(
                                server_entity,
                                model_data.clone(),
                            )),
                        )
                        .unwrap();
                }
            }
            ModelMsgClient::AllModelData(client_id, all_model_data) => {
                let mut endpoint: SystemState<ResMut<Server>> = SystemState::new(world);
                let mut endpoint = endpoint.get_mut(world);
                let endpoint = endpoint.endpoint_mut();
                for (entity, model_data) in all_model_data {
                    endpoint.send_lek_msg(client_id.clone(), ModelMsg::Server(ModelMsgServer::ModelAdded(entity, model_data))).unwrap();
                }
            }
        }
    }

    fn client(self, world: &mut World) {
        let msg = match self {
            ModelMsg::Server(msg) => msg,
            _ => panic!("server msg wasn't a server message"),
        };
        match msg {
            ModelMsgServer::ModelAdded(server_entity, model_data) => {
                let mut system_state: SystemState<(ResMut<EntityMap>, Commands, NonSend<SkDraw>)> =
                    SystemState::new(world);
                let (entity_map, commands, sk) = system_state.get_mut(world);
                let mut entity_map: ResMut<EntityMap> = entity_map;
                let mut commands: Commands = commands;
                let sk: NonSend<SkDraw> = sk;
                let model =
                    sk.model_create_mesh(sk.mesh_gen_cube([0.1, 0.1, 0.1], 1), Material::DEFAULT);
                let client_entity = ClientEntity(
                    commands
                        .spawn(ModelBundle::new(
                            model,
                            model_data.model_info,
                            model_data.transform,
                            model_data.color128,
                            model_data.render_layer,
                        ))
                        .insert(IgnoreModelAdd)
                        .id(),
                );
                entity_map.insert(client_entity, server_entity);
                system_state.apply(world);
            }
            ModelMsgServer::ModelChanged(server_entity, model_data) => {
                let mut client_entity = None;
                {
                    let mut system_state: SystemState<ResMut<EntityMap>> = SystemState::new(world);
                    let mut entity_map = system_state.get_mut(world);
                    client_entity = entity_map.get_by_right(&server_entity).map(|a| a.clone());
                }
                if let Some(client_entity) = client_entity {
                    let mut world_entity = world.entity_mut(client_entity.0);
                    match model_data {
                        ModelData2 {
                            transform,
                            color128,
                            render_layer,
                        } => {
                            *world_entity.get_mut().unwrap() = transform;
                            *world_entity.get_mut().unwrap() = color128;
                            *world_entity.get_mut().unwrap() = render_layer;
                        }
                    }
                }
            }
            ModelMsgServer::EntityMap(server_entity, client_entity) => {
                let mut system_state: SystemState<ResMut<EntityMap>> = SystemState::new(world);
                let mut entity_map: ResMut<EntityMap> = system_state.get_mut(world);
                entity_map.0.insert(client_entity, server_entity);
            }
            ModelMsgServer::GetAllModelData(client_id) => {
                let mut system_state: SystemState<(
                    Query<
                        (Entity, &ModelInfo, &Transform, &Color128, &RenderLayer),
                        (With<Networked>, Without<IgnoreModelChanged>),
                    >,
                ResMut<Client>, Res<EntityMap>)> = SystemState::new(world);
                let (query, mut client, entity_map) = system_state.get_mut(world);
                let mut client: ResMut<Client> = client;
                let entity_map: Res<EntityMap> = entity_map;
                let mut models = vec![];
                for (entity, model_info, transform, color128, render_layer) in query.iter() {
                    let server_entity = match entity_map.get_by_left(&ClientEntity(entity)) {
                        None => continue,
                        Some(server_entity) => server_entity.clone(),
                    };
                    models.push((server_entity, ModelData {
                        model_info: model_info.clone(),
                        transform: *transform,
                        color128: *color128,
                        render_layer: *render_layer,
                    }))
                }
                client.connection_mut().send_lek_msg(ModelMsg::Client(ModelMsgClient::AllModelData(client_id, models))).unwrap();

            }
        }
    }

    fn client_plugin(app: &mut App) {
        app.add_system(model_added);
        app.add_system(model_changed);
    }

    fn server_plugin(app: &mut App) {
        app.add_system(new_client_connected);
    }

    fn channel_id(&self) -> ChannelId {
        match self {
            ModelMsg::Client(client) => {
                match client {
                    ModelMsgClient::ModelAdded(_, _) => ChannelId::OrderedReliable(0),
                    ModelMsgClient::ModelChanged(_, _) => ChannelId::Unreliable,
                    ModelMsgClient::AllModelData(_, _) => ChannelId::OrderedReliable(0),
                }
            }
            ModelMsg::Server(server) => {
                match server {
                    ModelMsgServer::ModelAdded(_, _) => ChannelId::OrderedReliable(0),
                    ModelMsgServer::ModelChanged(_, _) => ChannelId::Unreliable,
                    ModelMsgServer::EntityMap(_, _) => ChannelId::OrderedReliable(0),
                    ModelMsgServer::GetAllModelData(_) => ChannelId::OrderedReliable(0),
                }
            }
        }
    }

    fn get_type_name() -> String {
        String::from("stereokit_bevy::networking::ModelMsg")
    }

    fn _server(world: &mut World, msg_bytes: &[u8], client_id: ClientId) {
        bincode::deserialize::<Self>(msg_bytes)
            .unwrap()
            .server(world, client_id)
    }

    fn _client(world: &mut World, msg_bytes: &[u8]) {
        bincode::deserialize::<Self>(msg_bytes)
            .unwrap()
            .client(world)
    }
}


pub struct StereoKitBevyClient;
pub struct StereoKitBevyServer;

impl Plugin for StereoKitBevyClient {
    fn build(&self, app: &mut App) {
        ModelMsg::add_leknet_client(app);
        fn stereokit_loop(mut app: App) {
            Settings::default()
                .init()
                .unwrap()
                .run(|_| app.update(), |_| ());
        }
        app.set_runner(stereokit_loop);
        app.insert_resource(unsafe { stereokit::Sk::create_unsafe() });
        app.insert_non_send_resource(unsafe { stereokit::SkDraw::create_unsafe() });
        app.add_system(sync_simple_transforms);
        app.add_system(model_draw);
    }
}
impl Plugin for StereoKitBevyServer {
    fn build(&self, app: &mut App) {
        ModelMsg::add_leknet_server(app);
        fn server_loop(mut app: App) {
            loop {
                app.update()
            }
        }
        app.set_runner(server_loop);
    }
}

pub struct StereoKitBevyClientPlugins;
pub struct StereoKitBevyServerPlugins;

impl PluginGroup for StereoKitBevyClientPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(StereoKitBevyClient)
            .add(bevy_transform::TransformPlugin)
            .add(bevy_time::TimePlugin)
            .add(bevy_quinnet::client::QuinnetClientPlugin::default())
    }
}

impl PluginGroup for StereoKitBevyServerPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(StereoKitBevyServer)
            .add(bevy_time::TimePlugin)
            .add(bevy_quinnet::server::QuinnetServerPlugin::default())
    }
}

#[test]
fn server_test() {
    let mut app = App::new();
    app.add_plugins(StereoKitBevyServerPlugins);
    app.add_startup_system(leknet::start_server);
    app.run();
}

#[test]
fn client_test() {
    let mut app = App::new();
    app.add_plugins(StereoKitBevyClientPlugins);
    app.add_startup_system(leknet::connect_to_server);
    app.add_startup_system(add_example_model);
    app.add_system(sync_example_model);
    app.run();
}

#[derive(Component)]
struct RightHand;

fn add_example_model(mut commands: Commands, sk: NonSend<SkDraw>) {
    let model_bundle = ModelBundle::new(
        sk.model_create_mesh(sk.mesh_gen_cube(Vec3::splat(0.1), 1), Material::DEFAULT),
                ModelInfo::Cube(Vec3::splat(0.1)),
        Default::default(),
        stereokit::named_colors::AQUAMARINE,
        Default::default(),
    );
    commands
        .spawn(model_bundle)
        .insert(RightHand)
        .insert(Networked);
}

fn sync_example_model(sk: Res<Sk>, mut query: Query<(&RightHand, &mut Transform)>) {
    for (_, mut transform) in query.iter_mut() {
        let palm = sk.input_hand(Handed::Right).palm;
        let temp = transform
            .with_rotation(palm.orientation)
            .with_translation(palm.position);
        transform.translation = temp.translation;
        transform.rotation = temp.rotation;
    }
}

fn new_client_connected(mut connected: EventReader<ConnectionEvent>, mut server: ResMut<Server>) {
    let endpoint = server.endpoint_mut();
    for client in connected.iter() {
        let client_id: ClientId = client.id.clone();
        for client_id2 in endpoint.clients() {
            if client_id2 == client_id {
                continue;
            }
            endpoint
                .send_message(
                    client_id2,
                    ModelMsg::Server(ModelMsgServer::GetAllModelData(client_id.clone())).to_message(),
                )
                .unwrap();
        }
    }
}

fn model_added(
    query: Query<
        (Entity, &Model, &Transform, &Color128, &RenderLayer),
        (Added<Networked>, Without<IgnoreModelAdd>),
    >,
    mut client: ResMut<Client>,
) {
    if let Some(connection) = client.get_connection_mut() {
        for (entity, _, transform, color128, render_layer) in query.iter() {
            connection
                .send_lek_msg(
                    ModelMsg::Client(ModelMsgClient::ModelAdded(
                        ClientEntity(entity),
                        ModelData {
                            model_info: ModelInfo::Cube([0.1, 0.1, 0.1].into()),
                            transform: *transform,
                            color128: *color128,
                            render_layer: *render_layer,
                        },
                    )),
                )
                .unwrap()
        }
    }
}

fn model_changed(
    query: Query<
        (Entity, &ModelInfo, &Transform, &Color128, &RenderLayer),
        (Or<(Changed<Transform>, Changed<Color128>, Changed<RenderLayer>)>, Without<IgnoreModelAdd>, With<Networked>),
    >,
    mut client: ResMut<Client>,
    entity_map: Res<EntityMap>,
) {
    if let Some(connection) = client.get_connection_mut() {
        for (entity, _, transform, color128, render_layer) in query.iter() {
            if let Some(server_entity) = entity_map.get_by_left(&ClientEntity(entity)) {
                connection
                    .send_lek_msg(
                        ModelMsg::Client(ModelMsgClient::ModelChanged(
                            *server_entity,
                            ModelData2 {
                                transform: *transform,
                                color128: *color128,
                                render_layer: *render_layer,
                            },
                        )),
                    )
                    .unwrap()
            }
        }
    }
}
