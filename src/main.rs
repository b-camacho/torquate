use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use std::f32::consts::PI;
use bevy_debug_grid::*;
use bevy::gltf::Gltf;

mod hover;
mod bulb;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugins(bevy_flycam::prelude::PlayerPlugin)
        .add_systems(Startup, setup)
        .add_plugins(hover::MouseRayPlugin)
        .add_plugins(bulb::BulbPlugin)
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
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });

    commands.spawn(SceneBundle {
        scene: asset_server.load("room.glb#Scene0"),
        transform: Transform::from_scale(Vec3{x: 5.0, y:5.0, z: 5.0}),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-32.0, 24.0, 12.0).with_rotation(Quat::from_euler(EulerRot::ZYX, -0.0725, -0.668, -0.502)),
        //projection: Projection::Orthographic(OrthographicProjection{scale: 0.08, ..OrthographicProjection::default()}),
        ..Default::default()
    }).insert(hover::MouseRaySource{});


    for (idx, (x, z)) in [ (-1f32, -2f32), (-1f32, 2f32), (1f32, -2f32), (1f32, 2f32) ].into_iter().enumerate() {
        commands
            .spawn(bulb::BulbBundle {
                plb: PointLightBundle {
                    transform: Transform::from_xyz(x*3.0, 3.0, z*3.0),
                    ..Default::default()
                },
                bulb: bulb::Bulb { index: idx.try_into().unwrap() },
            })
            .with_children(|builder| {
                builder.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere {
                        radius: 1.0,
                        ..default()
                    })),
                    material: materials.add(StandardMaterial {
                        base_color: Color::YELLOW,
                        emissive: Color::rgba_linear(7.13, 0.0, 0.0, 0.0),
                        ..default()
                    }),
                    ..default()
                });
            });
    }


}
