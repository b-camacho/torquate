use bevy::prelude::*;
use crate::hover::Draggable;

#[derive(Component)]
pub struct Bulb {
    pub index: u8,
}

#[derive(Bundle)]
pub struct BulbBundle {
    pub plb: PointLightBundle,
    pub bulb: Bulb,
}

#[derive(Component)]
struct Ghost;

#[derive(Resource, Reflect)]
struct IntensityBounds { min: f32, max: f32 }

#[derive(Resource, Reflect)]
struct DistanceBounds { min: f32, max: f32 }



trait MapRange {
    fn map(&self, src: (f32, f32), dst: (f32, f32)) -> f32;
}

impl MapRange for f32 {
    fn map(&self, src: (f32, f32), dst: (f32, f32)) -> f32 {
        dst.0 + (self - src.0) * (dst.1 - dst.0) / (src.1 - src.0)
    }
}

use std::f32::consts::PI;
fn move_ghost(time: Res<Time>, mut query: Query<&mut Transform, With<Ghost>>) {
    // move the sphere around
    for mut t in query.iter_mut() {
        let phase = time.elapsed_seconds() % PI;
        t.translation = traj_orbit(phase.into(), Vec3::default(), 10.0);
        //t.translation = traj_yoyo(phase.into(), Vec3{x: -10.0, y: 1.0, z: -10.0}, Vec3{x: 10.0, y: 1.0, z: 10.0})
    }
}

fn dim_by_distance(
    ghost_query: Query<&Transform, With<Ghost>>,
    intensity_bounds: Res<IntensityBounds>,
    distance_bounds: Res<DistanceBounds>,
    mut light_query: Query<(&mut PointLight, &Transform), With<Bulb>>,
) {
    let ghost = ghost_query.single();
    for (mut light, transform) in light_query.iter_mut() {
        let d = ghost.translation.distance(transform.translation);
        light.intensity = intensity_bounds.max -
            d.map(
                (distance_bounds.min, distance_bounds.max),
                (intensity_bounds.min, intensity_bounds.max)
                );
    }
}

fn traj_orbit(phase: f64, center: Vec3, radius: f64) -> Vec3 {
    // phase is [0, Pi), map to [0, 2*Pi) to get full circle
    let p2 = phase * 2.0;
    center + Vec3::new((p2.cos() * radius) as f32, 0.0, (p2.sin() * radius) as f32)
}

fn traj_yoyo(phase: f64, start: Vec3, end: Vec3) -> Vec3 {
    let normalized_phase = (phase * 2.0 / PI as f64).abs();
    let triangle_wave = if normalized_phase < 1.0 {
        normalized_phase
    } else {
        2.0 - normalized_phase
    };

    start.lerp(end, triangle_wave as f32)
}

#[allow(non_snake_case, clippy::too_many_arguments)]
fn traj_lissajous(
    phase: f64,
    a: f64,
    b: f64,
    c: f64,
    delta: f64,
    gamma: f64,
    A: f64,
    B: f64,
    C: f64,
) -> Vec3 {
    let x = A * (a * phase + delta).sin();
    let y = B * (b * phase).sin();
    let z = C * (c * phase + gamma).sin();

    Vec3::new(x as f32, y as f32, z as f32)
}

fn spawn_ghost(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(Mesh::from(shape::UVSphere {
        radius: 1.0,
        ..Default::default()
    }));


    commands.insert_resource(IntensityBounds{min: 20f32, max: 200f32});
    commands.insert_resource(DistanceBounds{min: 5f32, max: 10f32});


    let material = materials.add(Color::rgb(0.7, 0.7, 0.7).into());
    commands
        .spawn(PbrBundle {
            mesh,
            material,
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..Default::default()
        })
        .insert(Ghost)
        .insert(Draggable);

}

pub struct BulbPlugin;

impl Plugin for BulbPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ghost)
            //.add_systems(Update, move_ghost)
            .add_systems(Update, dim_by_distance);
    }
}
