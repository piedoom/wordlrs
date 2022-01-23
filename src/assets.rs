use crate::prelude::*;
use bevy::{asset::*, prelude::*, reflect::TypeUuid};
use bevy_asset_ron::RonAssetPlugin;
use rand::{seq::SliceRandom, thread_rng};

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
        self.handles
            .iter()
            .find(|x| assets.get_load_state(*x) == LoadState::Loading)
            .eq(&None)
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
    pub words: Vec<String>,
}

impl DictionaryAsset {
    pub fn contains(&self, pat: &str) -> bool {
        self.words.contains(&pat.to_string())
    }
    pub fn random(&mut self) -> &String {
        self.words.shuffle(&mut thread_rng());
        self.words.first().unwrap()
    }
}

fn load_assets_system(mut load_tracker: ResMut<LoadTracker>, assets: Res<AssetServer>) {
    paths::KEYBOARDS
        .iter()
        .map(|path| format!("keyboards/{path}.keyboard"))
        .chain(
            paths::DICTIONARIES
                .iter()
                .map(|path| format!("dictionaries/{path}.dict")),
        )
        .for_each(|path| {
            load_tracker.load(&path, &assets);
        });
}

fn check_loaded_system(
    mut state: ResMut<State<GameState>>,
    mut load_tracker: ResMut<LoadTracker>,
    mut current_word_list: ResMut<CurrentDictionaryResource>,
    mut events: EventWriter<GameEvent>,
    word_list_assets: Res<Assets<DictionaryAsset>>,
    assets: Res<AssetServer>,
) {
    if load_tracker.finished(&assets) {
        match load_tracker.stage {
            0 => {
                load_tracker.stage = 1;
            }
            1 => {
                // Set some defaults for now
                current_word_list.0 = word_list_assets.get_handle(
                    word_list_assets
                        .iter()
                        .find(|x| x.1.language == "english-us")
                        .map(|x| x.0)
                        .unwrap(),
                );
                load_tracker.stage = 2;
            }
            2 => {
                events.send(GameEvent::ChangeDictionary(current_word_list.0.clone()));
                load_tracker.stage = 3;
            }
            3 => {
                load_tracker.stage = 4;
            }
            4 => {
                state.replace(GameState::Main).ok();
            }
            _ => unreachable!(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, PartialEq, Default, Debug, Clone, Eq)]
#[uuid = "fccfcc12-4252-4fa8-adc4-78c5822269f8"]
pub struct KeyboardLayoutAsset {
    pub name: String,
    pub layout: Vec<Vec<char>>,
}
