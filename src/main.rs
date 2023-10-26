use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::PI;
use bevy_debug_grid::*;
use bevy::gltf::Gltf;

mod hover;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_systems(Startup, load_gltf)
        //.add_systems(Startup, spawn_gltf_objects)
        .add_systems(Startup, setup)
        .add_plugins(hover::MouseRayPlugin)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(DebugGridPlugin::with_floor_grid())
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(SceneBundle {
        scene: asset_server.load("room.glb#Scene0"),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        projection: Projection::Orthographic(OrthographicProjection{scale: 0.01, ..OrthographicProjection::default()}),
        ..Default::default()
    });


}
