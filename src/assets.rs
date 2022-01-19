use crate::prelude::*;
use bevy::{asset::*, prelude::*, reflect::TypeUuid};
use rand::{prelude::IteratorRandom, seq::SliceRandom, thread_rng};

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<WordListAsset>()
            .init_asset_loader::<WordListAssetLoader>()
            .init_resource::<LoadTracker>()
            .add_system_set(SystemSet::on_enter(GameState::Load).with_system(load_assets_system))
            .add_system_set(SystemSet::on_update(GameState::Load).with_system(check_loaded_system));
    }
}

#[derive(Default)]
pub struct LoadTracker(Vec<HandleUntyped>);

impl LoadTracker {
    pub fn finished(&self, assets: &AssetServer) -> bool {
        self.0
            .iter()
            .find(|x| assets.get_load_state(*x) == LoadState::Loading)
            .eq(&None)
    }

    pub fn load(&mut self, path: &str, assets: &AssetServer) {
        self.0.push(assets.load_untyped(path));
    }
}

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, PartialEq, Default, Debug, Clone, Eq)]
#[uuid = "fccfcc12-3456-4fa8-adc4-78c5822269f8"]
pub struct WordListAsset(pub Vec<String>);

impl From<&[u8]> for WordListAsset {
    fn from(bytes: &[u8]) -> Self {
        let string = std::str::from_utf8(bytes).unwrap();
        Self(
            string
                .split_whitespace()
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        )
    }
}

impl WordListAsset {
    pub fn contains(&self, pat: &str) -> bool {
        self.0.contains(&pat.to_string())
    }
    pub fn random(&mut self) -> &String {
        self.0.shuffle(&mut thread_rng());
        self.0.first().unwrap()
    }
}

#[derive(Default)]
pub struct WordListAssetLoader;

impl AssetLoader for WordListAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let asset = WordListAsset::from(bytes);
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["dict"]
    }
}

fn load_assets_system(mut load_tracker: ResMut<LoadTracker>, assets: Res<AssetServer>) {
    load_tracker.load("words.dict", &assets);
}

fn check_loaded_system(
    mut state: ResMut<State<GameState>>,
    load_tracker: Res<LoadTracker>,
    assets: Res<AssetServer>,
) {
    if load_tracker.finished(&assets) {
        state.replace(GameState::Main).ok();
    }
}
