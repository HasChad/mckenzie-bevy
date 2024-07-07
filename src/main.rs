use bevy::{
    math::VectorSpace,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
    window::{WindowMode, WindowResized},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use color_eyre::eyre::Result;
use rand::prelude::*;
use std::f32::consts::PI;

const RES_WIDTH: u32 = 320;
const RES_HEIGHT: u32 = 180;

#[derive(Component)]
struct Cube {
    rotate_timer: Timer,
    random_look_x: f32,
    random_look_y: f32,
}

impl Default for Cube {
    fn default() -> Self {
        Cube {
            rotate_timer: Timer::from_seconds(3.0, TimerMode::Once),
            random_look_x: 0.0,
            random_look_y: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum CubeState {
    #[default]
    Happy,
    Sad,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "McKenzie Bevy".into(),
                        mode: WindowMode::Windowed,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .init_state::<CubeState>()
        .insert_resource(Msaa::Off)
        //plugins
        .add_plugins(WorldInspectorPlugin::new())
        //systems
        .add_systems(Startup, (setup, setup_camera))
        .add_systems(
            Update,
            (
                fit_canvas,
                //update,
                happy_cube_update.run_if(in_state(CubeState::Happy)),
                sad_cube_update.run_if(in_state(CubeState::Sad)),
            ),
        )
        .run();

    Ok(())
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    //cube
    commands
        .spawn((
            SceneBundle {
                scene: asset_server.load("mckenzie-cube.glb#Scene0"),
                transform: Transform::from_xyz(0.0, 0.0, -13.0),
                ..default()
            },
            Name::new("Cube"),
        ))
        .insert(Cube::default());

    //point light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 10_000_000.,
            range: 100.,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 8.0, -7.0),
        ..default()
    });

    //point light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 30_000_000.,
            range: 100.,
            ..default()
        },
        transform: Transform::from_xyz(-5.0, -8.0, 7.0),
        ..default()
    });
}

// ! Camera setup
fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // this Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // this camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    commands.spawn(Camera3dBundle {
        camera: Camera {
            // render before the "main pass" camera
            target: RenderTarget::Image(image_handle.clone()),
            ..default()
        },
        ..default()
    });

    // spawn the canvas
    commands.spawn(SpriteBundle {
        texture: image_handle,
        ..default()
    });

    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn(Camera2dBundle::default());
}

// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projections: Query<&mut OrthographicProjection, With<Camera2d>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        let mut projection = projections.single_mut();
        projection.scale = 1. / h_scale.min(v_scale).round();
    }
}

//MARK: Main Code
fn happy_cube_update(
    time: Res<Time>,
    windows: Query<&Window>,
    mut query_cube: Query<(&mut Transform, &mut Cube)>,
    mut next_state: ResMut<NextState<CubeState>>,
) {
    let mouse_pos = windows.single().cursor_position();
    let (mut cube_transform, mut cube_prop) = query_cube.single_mut();
    let (mut cube_rot_y, mut cube_rot_x, _) = cube_transform.rotation.to_euler(EulerRot::YXZ);

    match mouse_pos {
        Some(position) => {
            let mousepos_x = position.x - windows.single().resolution.width() / 2.;
            let mousepos_y = position.y - windows.single().resolution.height() / 2.;

            if !cube_prop.rotate_timer.finished() {
                cube_prop.rotate_timer.tick(time.delta());

                let percentage_complete = cube_prop.rotate_timer.elapsed_secs()
                    / cube_prop.rotate_timer.duration().as_secs_f32();

                cube_rot_x = cube_rot_x.lerp((mousepos_y / 20.0).to_radians(), percentage_complete);
                cube_rot_y = cube_rot_y.lerp((mousepos_x / 20.0).to_radians(), percentage_complete);

                cube_transform.rotation = Quat::from_axis_angle(Vec3::Y, cube_rot_y)
                    * Quat::from_axis_angle(Vec3::X, cube_rot_x);
            } else {
                cube_transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, (mousepos_x / 20.0).to_radians())
                        * Quat::from_axis_angle(Vec3::X, (mousepos_y / 20.0).to_radians())
                        * Quat::from_axis_angle(Vec3::Z, 0.0);
            }
        }
        None => {
            cube_prop.rotate_timer.reset();
            cube_prop.random_look_x = -cube_rot_x;
            cube_prop.random_look_y = if cube_rot_y < 0. {
                cube_rot_y + PI
            } else {
                cube_rot_y - PI
            };

            info!("cube random y = {}", cube_prop.random_look_y);
            info!("cube random x = {}", cube_prop.random_look_x);

            next_state.set(CubeState::Sad);
        }
    }
}

fn sad_cube_update(
    time: Res<Time>,
    windows: Query<&Window>,
    mut query_cube: Query<(&mut Transform, &mut Cube)>,
    mut next_state: ResMut<NextState<CubeState>>,
) {
    let mouse_pos = windows.single().cursor_position();
    let mut rng = rand::thread_rng();
    let (mut cube_transform, mut cube_prop) = query_cube.single_mut();
    let (mut cube_rot_y, mut cube_rot_x, _) = cube_transform.rotation.to_euler(EulerRot::YXZ);

    match mouse_pos {
        None => {
            if !cube_prop.rotate_timer.finished() {
                cube_prop.rotate_timer.tick(time.delta());

                let percentage_complete = cube_prop.rotate_timer.elapsed_secs()
                    / cube_prop.rotate_timer.duration().as_secs_f32();

                cube_rot_y = cube_rot_y.lerp(cube_prop.random_look_y, percentage_complete);
                cube_rot_x = cube_rot_x.lerp(cube_prop.random_look_x, percentage_complete);

                cube_transform.rotation = Quat::from_axis_angle(Vec3::Y, cube_rot_y)
                    * Quat::from_axis_angle(Vec3::X, cube_rot_x);
            } else {
                if rng.gen_bool(0.5) {
                    cube_prop.random_look_y = rng.gen_range(2.6..PI);
                } else {
                    cube_prop.random_look_y = rng.gen_range(-PI..-2.6);
                }
                cube_prop.random_look_x = rng.gen_range(-0.3..0.3);
                cube_prop.rotate_timer =
                    Timer::from_seconds(rng.gen_range(0.5..3.0), TimerMode::Once);
            }
        }
        Some(_) => {
            cube_prop.rotate_timer.reset();
            next_state.set(CubeState::Happy);
        }
    }
}
