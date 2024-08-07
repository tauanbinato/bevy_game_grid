use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use bevy::asset::ron;
use serde::Deserialize;
use thiserror::Error;
use crate::schedule::InLoadGridSet;
use crate::state::GameState;


#[non_exhaustive]
#[derive(Debug, Error)]
enum BlobAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct LevelAssetBlob {
    bytes: Vec<u8>,
}

#[derive(Default)]
struct BlobAssetLoader;
impl AssetLoader for BlobAssetLoader {
    type Asset = LevelAssetBlob;
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

        Ok(LevelAssetBlob { bytes })
    }
}

#[derive(Resource, Default)]
struct AssetStore {
    blob: Handle<LevelAssetBlob>,
}

pub struct AssetLoaderPlugin;
impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetStore>()
            .init_asset::<LevelAssetBlob>()
            .init_asset_loader::<BlobAssetLoader>()
            .add_systems(PreStartup, setup)
            .add_systems(Update, print_on_load.run_if(in_state(GameState::LoadingAssets)));
    }
}

fn setup(mut state: ResMut<AssetStore>, asset_server: Res<AssetServer>) {

    // Will use BlobAssetLoader instead of CustomAssetLoader thanks to type inference
    state.blob = asset_server.load("data/level.json");
}

fn print_on_load(
    mut state: ResMut<AssetStore>,
    blob_assets: Res<Assets<LevelAssetBlob>>,
    mut next_state: ResMut<NextState<GameState>>
) {
    let blob = blob_assets.get(&state.blob);

    if blob.is_none() {
        info!("Blob Not Ready");
        return;
    }
    info!("Level Blob Loaded, Size: {:?} Bytes", blob.unwrap().bytes.len());
    next_state.set(GameState::BuildingGrid);

}
