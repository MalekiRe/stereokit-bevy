use bevy::prelude::{App, Component, Plugin, Query, TransformBundle};
use stereokit::{model, values};
use stereokit::enums::{DisplayMode, RenderLayer};
use stereokit::functions::{sk_run, SKSettings};
use stereokit::pose::Pose;
use stereokit::values::Matrix;

pub struct StereoKitPlugin;

impl Plugin for StereoKitPlugin {
    fn build(&self, app: &mut App) {
        app.set_runner(stereokit_runner).add_system(render_models);
    }
}

#[derive(Component)]
pub struct Visible(pub bool);

#[derive(Component)]
pub struct Model(pub model::Model);

#[derive(Component)]
pub struct Color(pub values::Color128);

#[derive(Component)]
pub struct Position(pub TransformBundle);

fn render_models(query: Query<(&Visible, &Model, &Position, &Color)>) {
    for render_obj in query.iter() {
        let transform_global = render_obj.2.0.global;
        if render_obj.0.0 {
            render_obj.1.0.draw(
                Pose::new(
                    transform_global.translation.into(),
                    transform_global.rotation.into())
                    .pose_matrix(transform_global.scale.into()),
                render_obj.3.0,
                RenderLayer::Layer0);
        }
    }
}

fn stereokit_runner(mut app: App) {
    SKSettings::default().display_preference(DisplayMode::Flatscreen).init();
    sk_run(&mut Box::new(&mut || {
        app.update();
    }), &mut Box::new(&mut || {
    }));
}