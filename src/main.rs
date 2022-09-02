#![allow(unused)]
use std::collections::HashSet;

use bevy::{
    ecs::{entity, system::Insert},
    math::{vec2, Vec3Swizzles},
    prelude::*,
    sprite::collide_aabb::collide,
};
use components::{
    Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, FromEnemy, FromPlayer, Laser, Movable,
    Player, SpriteSize, Velocity, Background, EnemyT2,
};
use enemy::EnemyPlugin;
use player::PlayerPlugin;

/* #region constlar */
const PLAYER_LASER: &str = "laserBlue15.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 57.);
const PLAYER_SPRITE: &str = "playerShip2_blue.png";
const PLAYER_SIZE: (f32, f32) = (112., 75.);
const PLAYER_RESPAWN_DELAY : f32 = 2.;

const ENEMY_SPRITE: &str = "enemyGreen3.png";
const ENEMY_SIZE: (f32, f32) = (103., 84.);
const ENEMYT2_SPRITE: &str = "enemyRed1.png";
const ENEMYT2_SIZE:(f32,f32) = (93.,84.);
const ENEMY_LASER: &str = "laserGreen07.png";
const ENEMY_LASER_SIZE: (f32, f32) = (9., 57.); 
const ENEMYT2_LASER: &str = "laserRed08.png";
const ENEMYT2_LASER_SIZE: (f32,f32) = (48.,46.);
const ENEMY_MAX: u32 = 4;
const ENEMYT2_MAX: u32 = 2;

const BACKGROUND_SPRITE: &str = "desert-backgorund-looped.png";
const BACKGROUND_HEIGHT: f32= 608.;

const EXPLOSION_SHEET: &str = "exp2_0.png";
const EXPLOSION_LEN: usize = 16;

const FORMATION_MEMBERS_MAX: u32 = 2;
const SPRITE_SCALE: f32 = 0.5;
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 500.;
/* #endregion */

//mod
mod components;
mod enemy;
mod player;

/* #region  structlar */
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}
struct GameTextures {
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    enemy_t2 : Handle<Image>,
    enemy_t2_laser : Handle<Image>,
    explosion: Handle<TextureAtlas>,
    background: Handle<Image>,
}
struct EnemyCount(u32,u32);
struct PlayerState {
    on: bool,       //alive
    last_shot: f64, //-1 if not shot
}
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: -1.,
        }
    }
}
impl PlayerState {
    pub fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time;
    }
    pub fn spawned(&mut self) {
        self.on = true;
        self.last_shot = -1.;
    }
}
/* #endregion */

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "ilk oyun".to_string(),
            width: 598.0,
            height: 676.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system(setup_system)
        .add_startup_system(background_spawn)
        .add_system(movable_system)
        .add_system(backgorund_movement)
        .add_system(player_laser_hit_enemy_system)
        .add_system(explosion_to_spawn_system)
        .add_system(explosion_animation_system)
        .add_system(enemy_laser_hit_player_system)
        .run();
}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut windows: ResMut<Windows>,
) {
    //kamera
    commands.spawn_bundle(Camera2dBundle::default());
    //windows
    let window = windows.get_primary_mut().unwrap();
    let (win_w, win_h) = (window.width(), window.height());
    //winsize
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);
    //Patlama
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4);
    let explosion = texture_atlases.add(texture_atlas);
    //GameTextures
    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER),
        explosion,
        background: asset_server.load(BACKGROUND_SPRITE),
        enemy_t2: asset_server.load(ENEMYT2_SPRITE),
        enemy_t2_laser: asset_server.load(ENEMYT2_LASER)
    };
    commands.insert_resource(game_textures);
    commands.insert_resource(EnemyCount(0,0));
}

fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        translation.y += velocity.y * TIME_STEP * BASE_SPEED;
        if translation.x > win_size.w / 2. + 50. {
            translation.x = -(win_size.w / 2. + 50.);
        }
        if translation.x < -(win_size.w / 2. + 50.) {
            translation.x = (win_size.w / 2. + 50.);
        }
        if movable.auto_despawn {
            const MARGIN: f32 = 1000.;
            if translation.y > win_size.h / 2. + MARGIN
                || translation.y < -win_size.h / 2. - MARGIN
                || translation.x > win_size.w / 2. + MARGIN
                || translation.x < -win_size.w / 2. - MARGIN
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn enemy_laser_hit_player_system(
    mut commands: Commands,
    mut player_state : ResMut<PlayerState>,
    time : Res<Time>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
) {
    if let Ok((player_entity, player_tf, player_size)) = player_query.get_single() {
        let player_scale = Vec2::from(player_tf.scale.xy());
        for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
            let laser_scale = Vec2::from(laser_tf.scale.xy());

            //determine if collision
            let collsision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                player_tf.translation,
                player_size.0 * player_scale,
            );
            if let Some(_) = collsision {
                commands.entity(player_entity).despawn();
                player_state.shot(time.seconds_since_startup());
                commands.entity(laser_entity).despawn();
                commands
                    .spawn()
                    .insert(ExplosionToSpawn(player_tf.translation.clone()));
                break;
            }
        }
    }
}

fn player_laser_hit_enemy_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
    enemyt2_query: Query<(Entity, &Transform, &SpriteSize), With<EnemyT2>>,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();
    for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
        let laser_scale = Vec2::from(laser_tf.scale.xy());
        if despawned_entities.contains(&laser_entity) {
            continue;
        }

        for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter() {
            let enemy_scale = Vec2::from(enemy_tf.scale.xy());
            if despawned_entities.contains(&laser_entity)
                || despawned_entities.contains(&enemy_entity)
            {
                continue;
            }
            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                enemy_tf.translation,
                enemy_size.0 * enemy_scale,
            );
            if let Some(_) = collision {
                commands.entity(enemy_entity).despawn();
                despawned_entities.insert(enemy_entity);
                enemy_count.0 -= 1;

                commands.entity(laser_entity).despawn();
                despawned_entities.insert(laser_entity);
                commands
                    .spawn()
                    .insert(ExplosionToSpawn(enemy_tf.translation.clone()));
            }
        }
        for (enemy_entity, enemy_tf, enemy_size) in enemyt2_query.iter() {
            let enemy_scale = Vec2::from(enemy_tf.scale.xy());
            if despawned_entities.contains(&laser_entity)
                || despawned_entities.contains(&enemy_entity)
            {
                continue;
            }
            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                enemy_tf.translation,
                enemy_size.0 * enemy_scale,
            );
            if let Some(_) = collision {
                commands.entity(enemy_entity).despawn();
                despawned_entities.insert(enemy_entity);
                enemy_count.1 -= 1;

                commands.entity(laser_entity).despawn();
                despawned_entities.insert(laser_entity);
                commands
                    .spawn()
                    .insert(ExplosionToSpawn(enemy_tf.translation.clone()));
            }
        }
    }
}

fn explosion_to_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    query: Query<(Entity, &ExplosionToSpawn)>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        // spawn the explosion sprite
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_textures.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(ExplosionTimer::default());

        // despawn the explosionToSpawn
        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1;
            if sprite.index >= EXPLOSION_LEN {
                commands.entity(entity).despawn();
            }
        }
    }
}


fn background_spawn(mut commands: Commands,asset_server: Res<AssetServer>){
    let background_sprite :Handle<Image> = asset_server.load(BACKGROUND_SPRITE);
    let bk_y = BACKGROUND_HEIGHT / 2. * 2.36 ;
    let mut spawn_background = |bk:f32| {
        commands
        .spawn_bundle(SpriteBundle {
            texture : background_sprite.clone(),
            transform : Transform {
                translation : Vec3::new(0.,bk,0.),
                scale : Vec3::new(2.36, 2.36, 1.),
                ..Default::default()
            }, 
            ..Default::default()
        })
        .insert(Background)
        .insert(Movable {auto_despawn :false})
        .insert(Velocity {x:0.,y:-0.2});};
    spawn_background(bk_y);
    spawn_background(-bk_y);   
}
fn backgorund_movement(mut commands: Commands,mut query: Query<&mut Transform,With<Background>>,win_size: Res<WinSize>){
    for mut transform in query.iter_mut(){
        let mut translation = &mut transform.translation;
        if translation.y < - BACKGROUND_HEIGHT * 2.36 / 2.0 -win_size.h / 2.0{
            translation.y = 2.0 * BACKGROUND_HEIGHT * 2.36 - win_size.h * 1.65;
        }
    }
}