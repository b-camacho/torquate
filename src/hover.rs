use bevy::prelude::*;
pub struct DraggablePlugin;

use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;

#[derive(Component)]
struct Hoverable;

#[derive(Component)]
struct Hover;

#[derive(Component, Default)]
struct MouseRay {
    ray: Ray,
}

#[derive(Component)]
struct Draggable;

#[derive(Component)]
struct Dragged {
    start_pos: Vec3
}

fn add_mouse_ray(mut commands: Commands) {
    commands.spawn(MouseRay::default());
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

            let (window_width, window_height) = (window.width(), window.height());
            let cursor_pos = event.position;
            let clip_space_pos = Vec3::new(
                cursor_pos.x / window_width * 2.0 - 1.0,
                // cursor_pos is from a `winit::CursorMoved` event
                // where positive x goes right and positive y goes **down**
                // see https://docs.rs/winit/latest/winit/event/enum.WindowEvent.html#variant.CursorMoved
                // in bevy, positive y goes **up**
                // flip y to convert
                1.0 - (cursor_pos.y / window_height * 2.0),
                0.0,
            );

            let inverse_projection = camera.projection_matrix().inverse();
            let eye_space_pos = inverse_projection.transform_point3(clip_space_pos);
            let world_space_pos = camera_transform.compute_matrix() * eye_space_pos.extend(1.0);

            mouse_ray.ray.origin = camera_transform.translation();
            mouse_ray.ray.direction =
                (world_space_pos.truncate() - camera_transform.translation()).normalize();
        }
    }
}

fn update_hover_start(
    mut commands: Commands,
    mesh_assets: Res<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    ray_query: Query<&MouseRay>,
    query: Query<(&Handle<Mesh>, &Handle<StandardMaterial>, &GlobalTransform, Entity), Without<Hover>>,
) {
    for ray in ray_query.iter() {
        for (mesh_handle, material_handle, transform, entity) in query.iter() {
            if let Some(mesh) = mesh_assets.get(mesh_handle) {
                if check_intersect(ray, mesh, transform) {
                    commands.entity(entity).insert(Hover{});
                    let mut material = material_assets.get_mut(material_handle).unwrap();
                    material.base_color = Color::RED;
                }
            }
        }
    }
}

fn update_hover_end(
    mut commands: Commands,
    mesh_assets: Res<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    ray_query: Query<&MouseRay>,
    query: Query<(&Handle<Mesh>, &Handle<StandardMaterial>, &GlobalTransform, Entity), With<Hover>>,
) {
    for ray in ray_query.iter() {
        for (mesh_handle, material_handle, transform, entity) in query.iter() {
            if let Some(mesh) = mesh_assets.get(mesh_handle) {
                if !check_intersect(ray, mesh, transform) {
                    println!("Unintersected {:?}", entity);
                    let mut material = material_assets.get_mut(material_handle).unwrap();
                    material.base_color = Color::BLACK;
                    commands.entity(entity).remove::<Hover>();
                }
            }
        }
    }
}

fn update_drag_start(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    query: Query<(Entity, &Transform), With<Hover>>,
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
            // Here, calculate the new position based on the ray's position
            // For simplicity, let's assume that the ray's direction is normalized and that you want
            // to move the object based on its intersection with a plane at z = 0

            // Calculate intersection of ray with the plane at z = 0
            if ray.direction.z.abs() > f32::EPSILON {
                let t = -ray.origin.z / ray.direction.z;
                let intersection_point = ray.origin + ray.direction * t;

                // Calculate the offset from the start position
                let offset = intersection_point - dragged.start_pos;
                
                // Update the position, but only in x and y
                transform.translation.x = dragged.start_pos.x + offset.x;
                transform.translation.y = dragged.start_pos.y + offset.y;
            }
        }
    }
}

fn check_intersect(ray: &MouseRay, mesh: &Mesh, transform: &GlobalTransform) -> bool {
    if let Some(VertexAttributeValues::Float32x3(vertex_positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        // todo: generalize to U16, ideally without calling a function
        // this is the kind of thing that's really easy in C++ due to the
        // "if it happens to have the methods I need, I'm happy" template system
        if let Some(bevy::render::mesh::Indices::U32(indices)) = mesh.indices() {
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
        }

    }
    false 
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
            .add_systems(Update, update_mouse_ray)
            .add_systems(Update, update_hover_start)
            .add_systems(Update, update_hover_end)
            .add_systems(Update, update_drag_start)
            .add_systems(Update, update_drag_end)
            .add_systems(Update, drag_system);
    }
}