use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::*;

pub const PLAYER_SIZE: f32 = 64.0;  // Sprite size
pub const PLAYER_SPEED: f32 = 500.0;

pub const ENEMY_SIZE: f32 = 64.0;  // Sprite size
pub const ENEMY_SPEED: f32 = 200.0;
pub const NUMBER_OF_ENEMIES: usize = 4;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (spawn_player, spawn_enemies, spawn_camera))  // Spawning systems
        .add_systems(Update, (player_movement, constrain_player_movement))  // Player movement systems
        .add_systems(Update, (enemy_movement, update_enemy_direction))  // Enemy movement systems
        .add_systems(Update, enemy_hit_player)  // Collision systems
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
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {
    let window = window_query.get_single().unwrap();

    let half_enemy_size = ENEMY_SIZE / 2.0;
    let x_min = 0.0 + half_enemy_size;
    let x_max = window.width() - half_enemy_size;
    let y_min = 0.0 + half_enemy_size;
    let y_max = window.height() - half_enemy_size;

    // TODO: Take into account movement past the screen border when rebounding
    for (mut transform, mut enemy) in enemy_query.iter_mut() {
        let mut translation = transform.translation;
        let mut direction_changed = false;

        if translation.x < x_min || translation.x > x_max {
            enemy.direction.x *= -1.0;
            direction_changed = true;
        }

        if translation.y < y_min || translation.y > y_max {
            enemy.direction.y *= -1.0;
            direction_changed = true;
        }

        if direction_changed {
            // Hit the edge of the screen, clamp to screen space
            translation.x = translation.x.clamp(x_min, x_max);
            translation.y = translation.y.clamp(y_min, y_max);
            transform.translation = translation;

            // Play the audio
            let audio_file = if random() { "audio/footstep_grass_001.ogg" } else { "audio/footstep_snow_000.ogg" };
            commands.spawn(AudioBundle {
                source: asset_server.load(audio_file),
                ..default()
            });
        }
    }
}

pub fn enemy_hit_player(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform), With<Player>>,
    enemy_query: Query<&Transform, With<Enemy>>,
    asset_server: Res<AssetServer>
) {
    if let Ok((player_entity, player_transform)) = player_query.get_single_mut() {
        for enemy_transform in enemy_query.iter() {
            let distance = player_transform.translation.distance(enemy_transform.translation);

            if distance < (PLAYER_SIZE + ENEMY_SIZE) / 2.0 {
                println!("Enemy hit player! Game Over!");

                commands.spawn(AudioBundle {
                    source: asset_server.load("audio/explosionCrunch_000.ogg"),
                    ..default()
                });

                commands.entity(player_entity).despawn();
            }
        }
    }
}