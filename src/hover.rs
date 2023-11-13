use bevy::prelude::*;
pub struct DraggablePlugin;

use bevy::prelude::*;
use std::collections::HashMap;
use bevy::render::mesh::VertexAttributeValues;

#[derive(Component)]
pub struct Hoverable;

#[derive(Component)]
struct Hover;

#[derive(Component, Default)]
struct MouseRay {
    ray: Ray,
}
#[derive(Component)]
pub struct MouseRaySource;

#[derive(Resource)]
struct HoverMaterial(Handle<StandardMaterial>);

#[derive(Resource)]
struct HoverMaterialStore(HashMap<Entity, Handle<StandardMaterial>>);

impl MouseRay {
    pub fn cursor_to_pos(position: &Vec2, window: &Window) -> Vec2 {
            let (window_width, window_height) = (window.width(), window.height());
            Vec2::new(
                position.x / window_width * 2.0 - 1.0,
                // cursor_pos is from a `winit::CursorMoved` event
                // where positive x goes right and positive y goes **down**
                // see https://docs.rs/winit/latest/winit/event/enum.WindowEvent.html#variant.CursorMoved
                // in bevy, positive y goes **up**
                // flip y to convert
                1.0 - (position.y / window_height * 2.0),
            )
    }

    pub fn pos_from_camera(
        camera: &Camera,
        transform: &GlobalTransform,
        cursor_pos: Vec2, // [-1, 1]
        ) -> Ray {
            let clip_space_pos = Vec3::new(
                cursor_pos.x,
                cursor_pos.y,
                0.0,
            );
            let inverse_projection = camera.projection_matrix().inverse();
            let eye_space_pos = inverse_projection.transform_point3(clip_space_pos);
            let world_space_pos = transform.compute_matrix() * eye_space_pos.extend(1.0);

            Ray { 
                origin: transform.translation(),
                direction: (world_space_pos.truncate() - transform.translation()).normalize()
            }
    }
}

#[derive(Component)]
pub struct Draggable;

#[derive(Component)]
struct Dragged {
    start_pos: Vec3
}

fn add_mouse_ray(mut commands: Commands) {
    commands.spawn(MouseRay::default());
}

fn add_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {

    let hover_material = materials.add(StandardMaterial {
        base_color: Color::RED,
        ..Default::default()
    });

    commands.insert_resource(HoverMaterial(hover_material));
    commands.insert_resource(HoverMaterialStore(HashMap::new()));

}

fn update_mouse_ray(
    mut query: Query<&mut MouseRay>,
    windows: Query<&Window>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if let (Ok(window), Ok(mut mouse_ray)) = (windows.get_single(), query.get_single_mut()) {
        for event in cursor_moved_events.iter() {
            let (camera, camera_transform) = camera_query.single();
            let cursor_pos = MouseRay::cursor_to_pos(&event.position, window);
            let ray = MouseRay::pos_from_camera(camera, camera_transform, cursor_pos);
            mouse_ray.ray = ray;
        }
    }
}

fn update_hover_start(
    mut commands: Commands,
    mesh_assets: Res<Assets<Mesh>>,
    hover_material: ResMut<HoverMaterial>,
    mut hover_material_store: ResMut<HoverMaterialStore>,
    ray_query: Query<&MouseRay>,
    query: Query<(&Handle<Mesh>, &Handle<StandardMaterial>, &GlobalTransform, &Hoverable, Entity), Without<Hover>>,
) {
    for ray in ray_query.iter() {
        for (mesh_handle, material_handle, transform, _, entity) in query.iter() {
            if let Some(mesh) = mesh_assets.get(mesh_handle) {
                if check_intersect(ray, mesh, transform) {
                    //println!("Intersected {:?}", entity);
                    commands.entity(entity).insert(Hover{});

                    hover_material_store.0.insert(entity, material_handle.clone());

                    commands.entity(entity).insert(hover_material.0.clone());
                }
            }
        }
    }
}

fn update_hover_end(
    mut commands: Commands,
    mesh_assets: Res<Assets<Mesh>>,
    mut hover_material_store: ResMut<HoverMaterialStore>,
    ray_query: Query<&MouseRay>,
    query: Query<(&Handle<Mesh>, &Handle<StandardMaterial>, &GlobalTransform, Entity), With<Hover>>,
) {
    for ray in ray_query.iter() {
        for (mesh_handle, _material_handle, transform, entity) in query.iter() {
            if let Some(mesh) = mesh_assets.get(mesh_handle) {
                if !check_intersect(ray, mesh, transform) {
                    if let Some(original_material_handle) = hover_material_store.0.remove(&entity) {
                        commands.entity(entity).insert(original_material_handle);
                    }
                    commands.entity(entity).remove::<Hover>();
                }
            }
        }
    }
}


fn update_drag_start(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<(Entity, &Transform), (With<Hover>, With<Draggable>)>,
    ) {
    for (entity, transform) in &query {
     if mouse_button_input.just_pressed(MouseButton::Left) {
         commands.entity(entity).insert(Dragged {
             start_pos: transform.translation
         });
     }
    }
}

fn update_drag_end(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<Entity, With<Dragged>>,
    ) {
    for entity in &query {
     if mouse_button_input.just_released(MouseButton::Left) {
         commands.entity(entity).remove::<Dragged>();
     }
    }
}

fn drag_system(
    mut query: Query<(&mut Transform, &Dragged)>,
    ray_query: Query<&MouseRay>,
) {
    for MouseRay{ray} in ray_query.iter() {
        for (mut transform, dragged) in query.iter_mut() {
                        // Define the y-coordinate of the plane
            let plane_y = dragged.start_pos.y; // Change this value as needed

            // Calculate the direction vector of the ray in the xy plane
            let direction_xy = Vec3::new(ray.direction.x, 0.0, ray.direction.z);

            // If the ray is not parallel to the plane
            if direction_xy.length() > f32::EPSILON {
                // Calculate intersection of ray with the plane at y = plane_y
                let t = (plane_y - ray.origin.y) / ray.direction.y;
                let intersection_point = ray.origin + ray.direction * t;

                // Calculate the offset from the start position, ignoring y
                let offset = Vec3::new(
                    intersection_point.x - dragged.start_pos.x,
                    0.0,
                    intersection_point.z - dragged.start_pos.z,
                );
                
                // update the position, only in x and z
                transform.translation.x = dragged.start_pos.x + offset.x;
                transform.translation.z = dragged.start_pos.z + offset.z;
            }
        }
    }
}

fn check_intersect(ray: &MouseRay, mesh: &Mesh, transform: &GlobalTransform) -> bool {
    if let Some(VertexAttributeValues::Float32x3(vertex_positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let inner_fn = |indices: &Vec<u32>| {
            for tri in indices.chunks_exact(3) {
                let v0 = Vec3::from(vertex_positions[tri[0] as usize]);
                let v1 = Vec3::from(vertex_positions[tri[1] as usize]);
                let v2 = Vec3::from(vertex_positions[tri[2] as usize]);

                // Transform the vertices from model space to world space
                let mat = transform.compute_matrix();
                let v0 = mat.transform_point3(v0);
                let v1 = mat.transform_point3(v1);
                let v2 = mat.transform_point3(v2);

                // Use Moller-Trumbore algorithm here to check for intersection
                if moller_trumbore(ray.ray.origin, ray.ray.direction, v0, v1, v2).is_some() {
                    return true
                }
            }
            false
        };

        match mesh.indices() {
            Some(bevy::render::mesh::Indices::U32(indices)) => inner_fn(indices),
            // TODO: very bad, clones mesh so I can avoid copy-pasting inner_fn
            Some(bevy::render::mesh::Indices::U16(indices)) => inner_fn(&indices.iter().map(|x| *x as u32).collect()),
            None => false,
        }
    } else { false }
}

pub fn moller_trumbore(
    ray_origin: Vec3,
    ray_direction: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<f32> {

    //
    let epsilon = 0.000_001;
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = ray_direction.cross(edge2);
    let a = edge1.dot(h);

    if a > -epsilon && a < epsilon {
        return None; // This ray is parallel to this triangle
    }

    let f = 1.0 / a;
    let s = ray_origin - v0;
    let u = f * s.dot(h);

    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray_direction.dot(q);

    if v < 0.0 || u + v > 1.0 {
        let upv = u+v;
        return None;
    }

    // At this stage we can compute t to find out where the intersection point is on the line
    let t = f * edge2.dot(q);

    if t > epsilon {
        // Ray intersection
        Some(t)
    } else {
        // This means that there is a line intersection but not a ray intersection
        None
    }
}

pub struct MouseRayPlugin;

impl Plugin for MouseRayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_mouse_ray)
            .add_systems(Startup, add_materials)
            .add_systems(Update, update_mouse_ray)
            .add_systems(Update, update_hover_start)
            .add_systems(Update, update_hover_end)
            .add_systems(Update, update_drag_start)
            .add_systems(Update, update_drag_end)
            .add_systems(Update, drag_system);
    }
}
