use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy_common_assets::json::JsonAssetPlugin;

#[derive(Resource)]
struct LevelHandle(Handle<Level>);

#[derive(serde::Deserialize, Asset, TypePath)]
struct Level {
    positions: Vec<[f32; 3]>,
}

pub struct AssetLoaderPlugin;
impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<Level>::new(&["level.json"]))
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let level = LevelHandle(asset_server.load("level.json"));
    commands.insert_resource(level);
    info!("Loaded level.json successfully!");
}
