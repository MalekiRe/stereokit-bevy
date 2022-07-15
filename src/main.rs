use std::ops::Add;
use std::thread;
use bevy::DefaultPlugins;
use bevy::prelude::{App, Commands, Component, Plugin, Query, Transform, With};
use stereokit_rs::enums::{DisplayMode, RenderLayer};
use stereokit_rs::{functions, stereokit_sys};
use stereokit_rs::material::Material;
use stereokit_rs::mesh::Mesh;
use stereokit_rs::model::Model;
use stereokit_rs::stereokit_sys::{sin, sk_init, sk_run};
use stereokit_rs::values::{Color128, Matrix, Vec3};
use prisma::{Rgb, Rgba};
use stereokit_rs::pose::Pose;

fn main() {
    App::new().add_startup_system(add_cube).add_system(move_small).add_plugin(StereoKitPlugin).run();
}

pub struct StereoKitPlugin;

impl Plugin for StereoKitPlugin {
    fn build(&self, app: &mut App) {
        app.set_runner(my_runner).add_system(draw_system);
    }
}

fn add_cube(mut commands: Commands) {
    let material: Material = Material::copy_from_id("default/material").unwrap();
    let mesh_cube: Mesh = Mesh::gen_cube(Vec3::from([0.1, 0.1, 0.1]), 10).unwrap();
    let cube_model: Model = Model::from_mesh(mesh_cube, material).unwrap();
    commands.spawn().insert(Model2(cube_model)).insert(Transform::default());
}

fn move_small(
    mut q: Query<&mut Transform>,
) {
    for mut transform in q.iter_mut() {
        let current_time = unsafe {stereokit_sys::time_getf() as f32};
        transform.translation.y += current_time.sin() / 100.0;
    }
}

fn draw_system(query: Query<(&Model2, &Transform)>) {
    for obj in query.iter() {
        obj.0.0.draw(Pose::new(obj.1.translation.into(), obj.1.rotation.into()).as_matrix(), Color128::from(Rgba::new(Rgb::new(0.1, 0.1, 0.1), 0.1)), RenderLayer::Layer0);
    }
}

#[derive(Component)]
struct Model2(Model);

fn my_runner(mut app: App) {
    functions::SKSettings::default().display_preference(DisplayMode::Flatscreen).init();
    functions::sk_run(&mut Box::new(&mut || {
        app.update();
    }), &mut Box::new(&mut || {
    }));
}
