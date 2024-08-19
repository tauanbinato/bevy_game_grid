use crate::state::GameState;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Level {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub world: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct StructuresData {
    pub structures: Vec<Vec<String>>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
enum BlobAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct AssetBlob {
    pub bytes: Vec<u8>,
}

#[derive(Default)]
struct BlobAssetLoader;
impl AssetLoader for BlobAssetLoader {
    type Asset = AssetBlob;
    type Settings = ();
    type Error = BlobAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading Blob...");
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        Ok(AssetBlob { bytes })
    }
}

#[derive(Resource, Default)]
pub struct AssetStore {
    pub level_blob: Handle<AssetBlob>,
    pub structures_blob: Handle<AssetBlob>,
}

pub struct AssetLoaderPlugin;
impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetStore>()
            .init_asset::<AssetBlob>()
            .init_asset_loader::<BlobAssetLoader>()
            .add_systems(PreStartup, setup)
            .add_systems(Update, print_on_load.run_if(in_state(GameState::LoadingAssets)));
    }
}

fn setup(mut state: ResMut<AssetStore>, asset_server: Res<AssetServer>) {
    // Will use BlobAssetLoader instead of CustomAssetLoader thanks to type inference
    state.level_blob = asset_server.load("data/level.json");

    state.structures_blob = asset_server.load("data/structures.json");
}

fn print_on_load(
    state: ResMut<AssetStore>,
    blob_assets: Res<Assets<AssetBlob>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let level_blob = blob_assets.get(&state.level_blob);
    let structures_blob = blob_assets.get(&state.structures_blob);

    if level_blob.is_none() && structures_blob.is_none() {
        info!("Blobs Not Ready");
        return;
    }
    info!("Level Blob Loaded, Size: {:?} Bytes", level_blob.unwrap().bytes.len());
    info!("Structures Blob Loaded, Size: {:?} Bytes", structures_blob.unwrap().bytes.len());

    next_state.set(GameState::BuildingGrid);
}
