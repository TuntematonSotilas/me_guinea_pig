use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButton;
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use bevy::window::PrimaryWindow;
use std::env;
use std::path::PathBuf;
use std::process::Command;

const PLAYER_SPEED: f32 = 180.0;
const DAY_LENGTH_SECS: f32 = 60.0;
const NEED_DECAY_PER_SEC: f32 = 6.0;
const MAX_NEED: f32 = 100.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct BackgroundMaterial(Handle<ColorMaterial>);

#[derive(Component)]
struct TargetPoint {
    position: Vec3,
}

#[derive(Component)]
struct Needs {
    food: f32,
    water: f32,
    health: f32,
    happiness: f32,
}

#[derive(Component)]
struct Interactable {
    action: InteractionAction,
}

#[derive(Clone, Copy)]
enum InteractionAction {
    Eat,
    Drink,
    Clean,
    Play,
}

#[derive(Resource)]
struct DayNightCycle {
    elapsed: f32,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if should_launch_android(&args) {
        if let Err(err) = run_android_debug() {
            eprintln!("failed to launch Android debug build: {err}");
            std::process::exit(1);
        }
        return;
    }

    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(DayNightCycle { elapsed: 0.0 })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_tap_to_move,
                move_player,
                update_day_night,
                decay_needs,
                handle_interactions,
            ),
        )
        .run();
}

fn should_launch_android(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "android")
}

fn run_android_debug() -> Result<(), String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = PathBuf::from(manifest_dir).join("scripts/run-android.sh");

    let status = Command::new("bash")
        .arg(&script_path)
        .current_dir(manifest_dir)
        .status()
        .map_err(|err| format!("failed to execute {}: {err}", script_path.display()))?;

    if !status.success() {
        return Err(format!("Android debug launcher exited with status {status}"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::should_launch_android;

    #[test]
    fn detects_android_launch_argument() {
        assert!(should_launch_android(&["me_guinea_pig".to_string(), "android".to_string()]));
        assert!(!should_launch_android(&["me_guinea_pig".to_string()]));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    let background_material = materials.add(ColorMaterial::from_color(Color::srgb(0.84, 0.9, 0.97)));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1200.0, 800.0))),
        MeshMaterial2d(background_material.clone()),
        Transform::from_xyz(0.0, 0.0, -1.0),
        Background,
        BackgroundMaterial(background_material),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(48.0, 48.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.92, 0.64, 0.38)))),
        Transform::from_xyz(0.0, 0.0, 1.0),
        Player,
        Needs {
            food: 70.0,
            water: 70.0,
            health: 80.0,
            happiness: 75.0,
        },
        TargetPoint { position: Vec3::ZERO },
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 40.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.3, 0.6, 0.25)))),
        Transform::from_xyz(-200.0, 0.0, 0.5),
        Interactable { action: InteractionAction::Eat },
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 40.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.35, 0.7)))),
        Transform::from_xyz(200.0, 0.0, 0.5),
        Interactable { action: InteractionAction::Drink },
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 40.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.85, 0.85, 0.2)))),
        Transform::from_xyz(0.0, -120.0, 0.5),
        Interactable { action: InteractionAction::Play },
    ));
}

fn handle_tap_to_move(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    touches: Res<Touches>,
    mouse_inputs: Res<ButtonInput<MouseButton>>,
    mut players: Query<&mut TargetPoint, With<Player>>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    let mut target_position: Option<Vec2> = None;

    if let Some(touch) = touches.iter_just_pressed().next() {
        target_position = Some(touch.position());
    }

    if target_position.is_none() && mouse_inputs.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            target_position = Some(cursor_position);
        }
    }

    let Some(position) = target_position else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, position) else {
        return;
    };

    let Some(mut target_point) = players.iter_mut().next() else {
        return;
    };

    target_point.position = Vec3::new(world_position.x, world_position.y, 1.0);
}

fn move_player(
    time: Res<Time>,
    mut players: Query<(&mut Transform, &TargetPoint), With<Player>>,
) {
    for (mut transform, target_point) in &mut players {
        let target = target_point.position;
        if (transform.translation - target).length_squared() < 1.0e-4 {
            continue;
        }

        let delta = target - transform.translation;
        let movement = delta.normalize_or_zero() * PLAYER_SPEED * time.delta_secs();
        let next_position = transform.translation + movement;

        if (next_position - target).length_squared() < movement.length_squared() {
            transform.translation = target;
        } else {
            transform.translation = next_position;
        }
    }
}

fn decay_needs(time: Res<Time>, mut players: Query<&mut Needs, With<Player>>) {
    for mut needs in &mut players {
        needs.food = (needs.food - NEED_DECAY_PER_SEC * time.delta_secs()).clamp(0.0, MAX_NEED);
        needs.water = (needs.water - NEED_DECAY_PER_SEC * time.delta_secs()).clamp(0.0, MAX_NEED);
        needs.health = (needs.health - NEED_DECAY_PER_SEC * 0.25 * time.delta_secs()).clamp(0.0, MAX_NEED);
        needs.happiness = (needs.happiness - NEED_DECAY_PER_SEC * 0.2 * time.delta_secs()).clamp(0.0, MAX_NEED);
    }
}

fn handle_interactions(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    touches: Res<Touches>,
    mouse_inputs: Res<ButtonInput<MouseButton>>,
    mut players: Query<&mut Needs, With<Player>>,
    interactables: Query<(&Transform, &Interactable)>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };

    let mut target_position: Option<Vec2> = None;
    if let Some(touch) = touches.iter_just_pressed().next() {
        target_position = Some(touch.position());
    }

    if target_position.is_none() && mouse_inputs.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            target_position = Some(cursor_position);
        }
    }

    let Some(position) = target_position else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, position) else {
        return;
    };

    let Some(mut needs) = players.iter_mut().next() else {
        return;
    };

    for (transform, interactable) in &interactables {
        let rect = transform.translation.truncate();
        let distance = (rect - world_position).length();
        if distance < 90.0 {
            match interactable.action {
                InteractionAction::Eat => needs.food = (needs.food + 20.0).clamp(0.0, MAX_NEED),
                InteractionAction::Drink => needs.water = (needs.water + 20.0).clamp(0.0, MAX_NEED),
                InteractionAction::Clean => needs.health = (needs.health + 15.0).clamp(0.0, MAX_NEED),
                InteractionAction::Play => needs.happiness = (needs.happiness + 20.0).clamp(0.0, MAX_NEED),
            }
        }
    }
}

fn update_day_night(
    time: Res<Time>,
    mut cycle: ResMut<DayNightCycle>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    backgrounds: Query<&BackgroundMaterial, With<Background>>,
) {
    cycle.elapsed += time.delta_secs();
    let phase = (cycle.elapsed / DAY_LENGTH_SECS) % 1.0;
    let angle = phase * std::f32::consts::TAU;
    let daylight = (angle.sin() * 0.5 + 0.5).clamp(0.0, 1.0);

    let r = 0.12 + (0.84 - 0.12) * daylight;
    let g = 0.16 + (0.9 - 0.16) * daylight;
    let b = 0.24 + (0.97 - 0.24) * daylight;

    for background_material in &backgrounds {
        if let Some(mut material) = materials.get_mut(&background_material.0) {
            material.color = Color::srgb(r, g, b);
        }
    }
}
