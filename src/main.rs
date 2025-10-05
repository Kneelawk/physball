use bevy::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, add_sphere)
        .run()
}

fn add_sphere(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    cmd.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    cmd.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));

    cmd.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.6, 0.8))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
