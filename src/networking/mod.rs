use crate::{model_draw, ModelInfo};
use bevy_app::{App, Plugin, PluginGroup, PluginGroupBuilder};
use bevy_ecs::prelude::Component;
use bevy_transform::prelude::Transform;
use bevy_transform::systems::sync_simple_transforms;
use leknet::{ClientMessage, LeknetClient, LeknetServer, ServerMessage};
use serde::{Deserialize, Serialize};
use stereokit::{Color128, RenderLayer, Settings};

mod model_client;
mod model_server;
#[cfg(test)]
mod tests;

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

#[derive(Component)]
pub struct IgnoreModelAdd;
#[derive(Component)]
pub struct IgnoreModelChanged;

pub struct StereoKitBevyClient;
pub struct StereoKitBevyServer;

impl Plugin for StereoKitBevyClient {
    fn build(&self, app: &mut App) {
        model_client::ModelMsgClient::add_plugin_client(app);
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
        model_server::ModelMsgServer::add_plugin_server(app);
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
            .add(LeknetClient)
            .add(bevy_transform::TransformPlugin)
            .add(bevy_time::TimePlugin)
            .add(bevy_quinnet::client::QuinnetClientPlugin::default())
    }
}

impl PluginGroup for StereoKitBevyServerPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(StereoKitBevyServer)
            .add(LeknetServer)
            .add(bevy_time::TimePlugin)
            .add(bevy_quinnet::server::QuinnetServerPlugin::default())
    }
}
