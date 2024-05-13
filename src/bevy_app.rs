use std::f32::consts::PI;

use crate::WorkerApp;
use bevy::color::palettes::css::BLANCHED_ALMOND;
use bevy::input::mouse::MouseWheel;
use bevy::{
    color::palettes::basic::SILVER,
    math::bounding::{Aabb3d, Bounded3d, RayCast3d},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use rand::Rng;

pub(crate) fn init_app() -> WorkerApp {
    let mut app = App::new();

    let mut default_plugins = DefaultPlugins.set(ImagePlugin::default_nearest());
    default_plugins = default_plugins.set(bevy::window::WindowPlugin {
        primary_window: Some(bevy::window::Window {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..default()
        }),
        ..default()
    });
    app.add_plugins(default_plugins);

    app.add_systems(Startup, setup)
        .add_systems(Update, (rotate, update_aabbes, mouse_events_system))
        .add_systems(PostUpdate, render_active_shapes);

    WorkerApp::new(app)
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component, Clone)]
enum Shape {
    Box(Cuboid),
    // Capsule(Capsule3d),
    // Torus(Torus),
    // Cylinder(Cylinder),
    // None,
}
/// 标记是否 选中/高亮
#[derive(Component, Default)]
struct ActiveState {
    hover: bool,
    selected: bool,
}

impl ActiveState {
    fn is_active(&self) -> bool {
        if self.hover || self.selected {
            true
        } else {
            false
        }
    }
}

const X_EXTENT: f32 = 19.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    let meshe_handles = [
        meshes.add(Cuboid::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Capsule3d::default()),
        meshes.add(Torus::default()),
        meshes.add(Cylinder::default()),
        meshes.add(Cuboid::default()),
        meshes.add(Sphere::default().mesh().ico(5).unwrap()),
        meshes.add(Sphere::default().mesh().uv(32, 18)),
    ];
    // 包围盒形状
    let shapes = [
        Shape::Box(Cuboid::from_size(Vec3::splat(1.1))),
        Shape::Box(Cuboid::from_size(Vec3::new(1., 2., 1.))),
        Shape::Box(Cuboid::from_size(Vec3::new(1.75, 0.52, 1.75))),
        Shape::Box(Cuboid::default()),
        Shape::Box(Cuboid::from_size(Vec3::new(1., 2., 1.))),
        Shape::Box(Cuboid::from_size(Vec3::new(1.75, 0.52, 1.75))),
        Shape::Box(Cuboid::default()),
        Shape::Box(Cuboid::from_size(Vec3::splat(1.1))),
        Shape::Box(Cuboid::default()),
        Shape::Box(Cuboid::default()),
    ];

    let num_shapes = meshe_handles.len();
    let mut rng = rand::thread_rng();

    for i in 0..num_shapes {
        for z in 0..8 {
            for y in 0..2 {
                let index = rng.gen_range(0..num_shapes);
                let mesh = meshe_handles[index].to_owned();
                let shape = shapes[index].to_owned();
                let transform = Transform::from_xyz(
                    -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                    1. + 4.5 * y as f32,
                    (3.0 - z as f32) * 1.95 + 1.5,
                );
                commands.spawn((
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: debug_material.clone(),
                        transform: transform.with_rotation(Quat::from_rotation_x(-PI / 4.)),
                        ..default()
                    },
                    shape.clone(),
                    ActiveState::default(),
                ));
            }
        }
    }

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
        material: materials.add(Color::from(SILVER)),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 22., 8.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}

/// 绘制 选中/高亮 包围盒
fn render_active_shapes(mut gizmos: Gizmos, query: Query<(&Shape, &Transform, &ActiveState)>) {
    let color = BLANCHED_ALMOND;
    for (shape, transform, active_state) in query.iter() {
        if !active_state.is_active() {
            continue;
        }
        let translation = transform.translation.xyz();
        match shape {
            Shape::Box(cuboid) => {
                gizmos.primitive_3d(*cuboid, translation, transform.rotation, color);
            } // Shape::Capsule(c) => {
              //     gizmos.primitive_3d(*c, translation, transform.rotation, color);
              // }
        }
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// entity 的 aabb
#[derive(Component, Debug)]
struct CurrentVolume(Aabb3d);

/// 更新 aabb
fn update_aabbes(
    mut commands: Commands,
    mut config_store: ResMut<GizmoConfigStore>,

    query: Query<(Entity, &Shape, &Transform), Or<(Changed<Shape>, Changed<Transform>)>>,
) {
    for (_, config, _) in config_store.iter_mut() {
        config.line_width = 3.;
    }

    for (entity, shape, transform) in query.iter() {
        let translation = transform.translation;
        let rotation = transform.rotation;

        let aabb = match shape {
            Shape::Box(b) => b.aabb_3d(translation, rotation),
        };
        commands.entity(entity).insert(CurrentVolume(aabb));
    }
}

/// 构造一条相机射线
fn ray_from_screenspace(
    cursor_pos_screen: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Ray3d> {
    let mut viewport_pos = cursor_pos_screen;
    if let Some(viewport) = &camera.viewport {
        viewport_pos -= viewport.physical_position.as_vec2();
    }
    camera
        .viewport_to_world(camera_transform, viewport_pos)
        .map(Ray3d::from)
}

fn mouse_events_system(
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(Entity, &CurrentVolume, &mut ActiveState)>,
) {
    for event in cursor_moved_events.read() {
        let (camera, transform) = cameras.get_single().unwrap();
        let ray = ray_from_screenspace(event.position, camera, transform).unwrap();
        let ray_cast = RayCast3d::from_ray(ray, 30.);
        // 计算射线拾取
        for (entity, volume, mut status) in query.iter_mut() {
            let toi = ray_cast.aabb_intersection_at(&volume.0);
            status.hover = toi.is_some();

            // 通知
        }
    }

    for _event in mouse_wheel_events.read() {}
}
