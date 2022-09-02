#![allow(unused)]
use bevy::{prelude::*, math::vec2};
use components::{Velocity, Player, Movable};
use player::PlayerPlugin;

//constlar
const PLAYER_LASER:&str = "laserBlue15.png";
const PLAYER_LASER_SİZE:(f32,f32)=(9.,57.);
const PLAYER_SPRITE: &str = "playerShip2_blue.png";
const PLAYER_SIZE :(f32,f32)= (112.,75.); 
const SPRITE_SCALE: f32= 0.5;
const TIME_STEP:f32=1./60.;
const BASE_SPEED:f32 = 500.;

//mod
mod player;
mod components;

//structlar
pub struct  WinSize {
    pub w : f32,
    pub h : f32
}
struct GameTextures {
    player:Handle<Image>,
    player_laser:Handle<Image>,
}



fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "ilk oyun".to_string(),
            width: 598.0,
            height:676.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_system)
        .add_system(movable_system)
        .run();
}

fn setup_system(
    mut commands : Commands,
    asset_server:Res<AssetServer>,
    mut windows: ResMut<Windows>
) {fn player_movement_system(mut query: Query<(&Velocity,&mut Transform),With<Player>>){
    for (velocity, mut transform) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x = velocity.x * TIME_STEP * BASE_SPEED;
        translation.y = velocity.y * TIME_STEP * BASE_SPEED;
    }
}
    //kamera
    commands.spawn_bundle(Camera2dBundle::default());
    //windows
    let window = windows.get_primary_mut().unwrap();
    let (win_w,win_h) = (window.width(),window.height());
    //winsize
    let win_size = WinSize { w: win_w, h: win_h };
	commands.insert_resource(win_size);
    //GameTextures
    let game_textures = GameTextures{
        player : asset_server.load(PLAYER_SPRITE),
        player_laser : asset_server.load(PLAYER_LASER),
    };
    commands.insert_resource(game_textures);
}


fn movable_system(
    mut commands:Commands,
    win_size : Res<WinSize>,
    mut query: Query<(Entity ,&Velocity,&mut Transform,&Movable)>){
    for (entity,velocity, mut transform ,movable) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        translation.y += velocity.y * TIME_STEP * BASE_SPEED;
    }
}