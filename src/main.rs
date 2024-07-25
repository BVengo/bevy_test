use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::*;

pub const PLAYER_SIZE: f32 = 64.0;  // Sprite size
pub const PLAYER_SPEED: f32 = 500.0;

pub const ENEMY_SIZE: f32 = 64.0;  // Sprite size
pub const ENEMY_SPEED: f32 = 300.0;
pub const NUMBER_OF_ENEMIES: usize = 4;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (spawn_player, spawn_enemies, spawn_camera))  // Spawning systems
        .add_systems(Update, (player_movement, constrain_player_movement))  // Player movement systems
        .add_systems(Update, (enemy_movement, update_enemy_direction, constrain_enemy_movement))  // Enemy movement systems
        .run();
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Enemy {
    pub direction: Vec2
}

pub fn spawn_player(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();  // Always provided by Bevy

    commands.spawn(
        (
            Player {},
            SpriteBundle {
                transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
                texture: asset_server.load("sprites/ball_blue_large.png"),
                ..default()
            },
        )
    );
}

pub fn spawn_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();  // Always provided by Bevy

    for _ in 0..NUMBER_OF_ENEMIES {
        // Generate random position, bound within the screen (accounting for enemy size)
        let random_x = random::<f32>() * (window.width() - ENEMY_SIZE) + ENEMY_SIZE / 2.0;
        let random_y = random::<f32>() * (window.height() - ENEMY_SIZE) + ENEMY_SIZE / 2.0;

        commands.spawn(
            (
                Enemy {
                    direction: Vec2::new(random::<f32>(), random::<f32>()),
                },
                SpriteBundle {
                    transform: Transform::from_xyz(random_x, random_y, 0.0),
                    texture: asset_server.load("sprites/ball_red_large.png"),
                    ..default()
                },
            )
        );
    }
}

pub fn spawn_camera(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_query.get_single().unwrap(); // Always provided by Bevy

    commands.spawn(
      Camera2dBundle {
          transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 1.0),
          ..default()
      }
    );
}

pub fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>
) {
    if let Ok(mut transform) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyW) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keys.pressed(KeyCode::KeyS) {
            direction += Vec3::new(0.0, -1.0, 0.0);
        }

        if keys.pressed(KeyCode::KeyA) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }

        if keys.pressed(KeyCode::KeyD) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();

            transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
        }
    }
}

pub fn constrain_player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Confine the player to the screen
    if let Ok(mut transform) = player_query.get_single_mut() {
        let window = window_query.get_single().unwrap();

        let player_half = PLAYER_SIZE / 2.0;

        let mut translation = transform.translation;
        translation.x = translation.x.clamp(player_half, window.width() - player_half);
        translation.y = translation.y.clamp(player_half, window.height() - player_half);

        transform.translation = translation;
    }
}

pub fn enemy_movement(
    mut enemy_query: Query<(&mut Transform, &Enemy)>,
    time: Res<Time>
) {
    for (mut transform, enemy) in &mut enemy_query {
        let direction = Vec3::new(enemy.direction.x, enemy.direction.y, 0.0);
        transform.translation += direction * ENEMY_SPEED * time.delta_seconds();
    }
}

pub fn update_enemy_direction(
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_query.get_single().unwrap();

    let half_enemy_size = ENEMY_SIZE / 2.0;
    let x_min = 0.0 + half_enemy_size;
    let x_max = window.width() - half_enemy_size;
    let y_min = 0.0 + half_enemy_size;
    let y_max = window.height() - half_enemy_size;

    for (transform, mut enemy) in enemy_query.iter_mut() {
        let translation = transform.translation;

        if translation.x < x_min || translation.x > x_max {
            enemy.direction.x *= -1.0;
        }

        if translation.y < y_min || translation.y > y_max {
            enemy.direction.y *= -1.0;
        }
    }
}

pub fn constrain_enemy_movement(
    mut enemy_query: Query<&mut Transform, With<Enemy>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Confine the enemy to the screen
    let window = window_query.get_single().unwrap();

    for mut transform in &mut enemy_query {
        let enemy_half = ENEMY_SIZE / 2.0;

        let mut translation = transform.translation;
        translation.x = translation.x.clamp(enemy_half, window.width() - enemy_half);
        translation.y = translation.y.clamp(enemy_half, window.height() - enemy_half);

        transform.translation = translation;
    }
}