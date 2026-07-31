use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Component)] //orbit camera marker
struct OrbitCamera {
    radius: f32,           //distance from target point
    yaw: f32,              //horizonatal rotation
    pitch: f32,            //vertical rotation
    target: Vec3,          //camera point orbit
    sensitivity: f32,      //mouse rotation sensivity
    zoom_sensitivity: f32, //zoom sensitivity
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            radius: 3.0,
            yaw: 0.0,
            pitch: 0.35,
            target: Vec3::new(0.0, 0.5, 0.0),
            sensitivity: 0.005,
            zoom_sensitivity: 0.15,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cain's Domain".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::linear_rgb(0.75, 0.82, 0.9)))
        .add_systems(Startup, setup)
        .add_systems(Update, (orbit_camera_system, auto_rotate_platform))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 1.5, 3.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        OrbitCamera::default(),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-1.0, -1.0, -0.5), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 5_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(1.0, -0.5, 0.5), Vec3::Y),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::linear_rgb(0.85, 0.88, 0.95),
        brightness: 400.0,
    });

    let grid_size = 20;
    let cell_size = 1.0;
    let half_extent = grid_size as f32 * cell_size / 2.0;

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(half_extent)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.85, 0.87, 0.92),
            perceptual_roughness: 0.6,
            metallic: 0.1,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Grid lines
    // This Grid lines are for maping the positions of the gun and the future enimes..
    // And In future It will be removed
    let line_material = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.2, 0.2, 0.25),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });
    let line_thickness = 0.008;
    let line_height = 0.002;

    for i in 0..=grid_size {
        let offset = (i as f32 * cell_size) - half_extent;

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(half_extent * 2.0, line_height, line_thickness))),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(0.0, line_height, offset),
        ));

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(line_thickness, line_height, half_extent * 2.0))),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(offset, line_height, 0.0),
        ));
    }

    let platform_entity = commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            Visibility::default(),
            RotatingPlatform { speed: 0.3 },
        ))
        .id();

    let gun_scene: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("gun_Standard_sight.glb"));

    let gun_entity = commands
        .spawn((
            SceneRoot(gun_scene),
            Transform::from_xyz(0.0, 0.3, 0.0).with_scale(Vec3::splat(1.0)),
        ))
        .id();

    let collar_scene: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("gun_Collar.glb"));

    let collar_entity = commands
        .spawn((
            SceneRoot(collar_scene),
            Transform::from_xyz(0.0, 0.3, 0.0).with_scale(Vec3::splat(1.0)),
        ))
        .id();

    commands
        .entity(platform_entity)
        .add_children(&[gun_entity, collar_entity]);

    let fog_material = materials.add(StandardMaterial {
        base_color: Color::linear_rgba(0.85, 0.88, 0.95, 0.3),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    for i in 0..8 {
        let angle = (i as f32 / 8.0) * PI * 2.0;
        let x = angle.cos() * 15.0;
        let z = angle.sin() * 15.0;
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(3.0))),
            MeshMaterial3d(fog_material.clone()),
            Transform::from_xyz(x, 0.5, z),
        ));
    }
}

#[derive(Component)]
struct RotatingPlatform {
    speed: f32,
}

fn auto_rotate_platform(time: Res<Time>, mut query: Query<(&RotatingPlatform, &mut Transform)>) {
    for (platform, mut transform) in &mut query {
        transform.rotate_y(platform.speed * time.delta_secs());
    }
}

fn orbit_camera_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let Ok((mut orbit, mut transform)) = query.single_mut() else {
        return;
    };

    if mouse_button.pressed(MouseButton::Left) || mouse_button.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            orbit.yaw -= ev.delta.x * orbit.sensitivity;
            orbit.pitch -= ev.delta.y * orbit.sensitivity;
            orbit.pitch = orbit.pitch.clamp(-PI / 2.2, PI / 2.2);
        }
    } else {
        mouse_motion.read().for_each(|_| {});
    }
    for ev in scroll_events.read() {
        orbit.radius -= ev.y * orbit.zoom_sensitivity;
        orbit.radius = orbit.radius.clamp(0.5, 20.0);
    }

    let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
    let y = orbit.radius * orbit.pitch.sin();
    let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

    transform.translation = orbit.target + Vec3::new(x, y, z);
    transform.look_at(orbit.target, Vec3::Y);
}
