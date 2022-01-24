use crate::prelude::*;
use bevy::{asset::*, prelude::*, reflect::TypeUuid};
use bevy_asset_ron::RonAssetPlugin;
use rand::{prelude::IteratorRandom, thread_rng};

pub mod paths {
    pub const KEYBOARDS: &[&str] = &["qwerty", "ЙЦУКЕН"];
    pub const DICTIONARIES: &[&str] = &["english", "русский"];
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadTracker>()
            .add_asset::<DictionaryAsset>()
            .add_asset::<KeyboardLayoutAsset>()
            .add_plugin(RonAssetPlugin::<DictionaryAsset>::new(&["dict"]))
            .add_plugin(RonAssetPlugin::<KeyboardLayoutAsset>::new(&["keyboard"]))
            .add_system_set(SystemSet::on_enter(GameState::Load).with_system(load_assets_system))
            .add_system_set(SystemSet::on_update(GameState::Load).with_system(check_loaded_system));
    }
}

#[derive(Default)]
pub struct LoadTracker {
    pub stage: usize,
    pub handles: Vec<HandleUntyped>,
}

impl LoadTracker {
    pub fn finished(&self, assets: &AssetServer) -> bool {
        !self
            .handles
            .iter()
            .any(|x| assets.get_load_state(x) == LoadState::Loading)
    }

    pub fn load(&mut self, path: &str, assets: &AssetServer) {
        self.handles.push(assets.load_untyped(path));
    }
}

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, PartialEq, Default, Debug, Clone, Eq)]
#[uuid = "fccfcc12-3456-4fa8-adc4-78c5822269f8"]
pub struct DictionaryAsset {
    pub name: String,
    pub language: String,
    pub keyboards: Vec<String>,
    words: Vec<String>,
}

impl DictionaryAsset {
    pub fn contains(&self, pat: &str) -> bool {
        self.words.contains(&pat.to_string())
    }
    pub fn random(&self, length: usize) -> Option<&String> {
        self.words
            .iter()
            .filter(|x| x.chars().count() == length)
            .choose(&mut thread_rng())
    }
}

fn load_assets_system(mut load_tracker: ResMut<LoadTracker>, assets: Res<AssetServer>) {
    paths::KEYBOARDS
        .iter()
        .map(|path| format!("./keyboards/{path}.keyboard"))
        .chain(
            paths::DICTIONARIES
                .iter()
                .map(|path| format!("./dictionaries/{path}.dict")),
        )
        .for_each(|path| {
            load_tracker.load(&path, &assets);
        });
}

fn check_loaded_system(
    mut state: ResMut<State<GameState>>,
    load_tracker: ResMut<LoadTracker>,
    assets: Res<AssetServer>,
    dictionaries: Res<Assets<DictionaryAsset>>,
) {
    if load_tracker.finished(&assets) {
        // get handle to english dictionary and a random word
        if let Some((dictionary, word)) =
            dictionaries
                .iter()
                .find_map(|(h, a)| match a.language == "english-us" {
                    true => Some((dictionaries.get_handle(h), a.random(5))),
                    false => None,
                })
        {
            // if dictionary is loaded, set the state to main
            state
                .replace(GameState::Main(GameOptions {
                    word: word.unwrap().to_string(),
                    dictionary,
                    settings: Default::default(),
                }))
                .ok();
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, PartialEq, Default, Debug, Clone, Eq)]
#[uuid = "fccfcc12-4252-4fa8-adc4-78c5822269f8"]
pub struct KeyboardLayoutAsset {
    pub name: String,
    pub layout: Vec<Vec<char>>,
}
