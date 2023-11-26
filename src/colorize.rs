use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use std::collections::HashSet;

#[derive(Resource)]
struct Meshes {
    pub inner: HashSet<Handle<Mesh>>,
}

fn check_mesh_loaded(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<Mesh>>,
    mesh_handles: Res<Meshes>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } if mesh_handles.inner.contains(handle) => {
                if let Some(mesh) = meshes.get_mut(handle) {
                    println!("Mesh {handle:?} loaded, applying vertex color");
                    if let Some(VertexAttributeValues::Float32x3(positions)) =
                        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                    {
                        let (mut min_x, mut min_y, mut min_z) =
                            (f32::INFINITY, f32::INFINITY, f32::INFINITY);
                        let (mut max_x, mut max_y, mut max_z) =
                            (-f32::INFINITY, -f32::INFINITY, -f32::INFINITY);
                        for [x, y, z] in positions {
                            min_x = min_x.min(*x);
                            min_y = min_y.min(*y);
                            min_z = min_z.min(*z);
                            max_x = max_x.max(*x);
                            max_y = max_y.max(*y);
                            max_z = max_z.max(*z);
                        }
                        println!(
                            "Mesh boundaries: [{min_x}:{max_x},{min_y}:{max_y},{min_z}:{max_z}]"
                        );

                        use crate::util::MapRange;
                        let colors: Vec<[f32; 4]> = positions
                            .iter()
                            .map(|[x, y, z]| {
                                let mapped = (x + y + z).map(
                                    (min_x + min_y + min_z, max_x + max_y + max_z),
                                    (0.3, 0.9),
                                );
                                [mapped, mapped, mapped, 1.0]
                            })
                            .collect();
                        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
                    }
                }
            }
            _ => {}
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let paths = [
        "room.gltf#Mesh2/Primitive0",
        "room.gltf#Mesh3/Primitive0",
        "room.gltf#Mesh4/Primitive0",
    ];
    let mut m = Meshes {
        inner: HashSet::default(),
    };
    for p in paths {
        let h = asset_server.get_handle(p);
        m.inner.insert(h);
    }
    commands.insert_resource(m);
}

pub struct ColorizePlugin;
impl Plugin for ColorizePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, check_mesh_loaded);
    }
}
