use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::PI;

mod hover;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup) // from prelude
        .add_plugins(hover::MouseRayPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let mesh = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));
    let material = materials.add(Color::rgb(0.7, 0.7, 0.7).into());
    commands.spawn(PbrBundle {
        mesh,
        material,
        transform: Transform {
            translation: Vec3::default(),
            rotation: Quat::from_rotation_x(PI/2f32),
            scale: Vec3::from_array([1f32,1f32,1f32])
        },
        ..Default::default()
    });

}
