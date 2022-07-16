use bevy::app::App;
use bevy::hierarchy::{Children, Parent};
use bevy::math::Vec3;
use bevy::prelude::{ChildBuilder, Commands, Quat, Transform, TransformBundle, TransformPlugin};
use prisma::Rgb;
use stereokit::constants::QUAT_IDENTITY;
use stereokit::material::Material;
use stereokit::mesh::Mesh;
use stereokit::model;
use stereokit::values::{Color128};
use crate::rendering::{Color, Model, Position, StereoKitPlugin, Visible};

pub mod rendering;


fn main() {
    App::new().add_plugin(StereoKitPlugin).add_plugin(TransformPlugin).add_startup_system(add_cube).run();
}

fn add_cube(mut commands: Commands) {
    let material: Material = Material::copy_from_id("default/material").unwrap();
    let mesh_cube: Mesh = Mesh::gen_cube(stereokit::values::Vec3::from([0.1, 0.1, 0.1]), 10).unwrap();
    let cube_model = model::Model::from_mesh(mesh_cube, material).unwrap();
    let child = commands.spawn().insert(Children::default())
        .insert(Model(cube_model))
        .insert(Position(TransformBundle::default()))
        .insert(Visible(true)).insert(Color(Color128::new(Rgb::new(0.5, 0.5, 0.5), 1.0))).id();
}



// fn move_small(
//     mut q: Query<&mut Transform>,
// ) {
//     for mut transform in q.iter_mut() {
//         let current_time = unsafe {stereokit_sys::time_getf() as f32};
//         transform.translation.y += current_time.sin() / 100.0;
//     }
// }
