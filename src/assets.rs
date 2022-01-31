use std::ops::Deref;

use crate::prelude::*;
use bevy::{asset::*, prelude::*, reflect::TypeUuid};
use bevy_asset_ron::RonAssetPlugin;
use rand::{
    prelude::{IteratorRandom, SliceRandom},
    thread_rng,
};

pub mod paths {
    pub const KEYBOARDS: &[&str] = &["qwerty", "ЙЦУКЕН"];
    pub const LANGUAGES: &[&str] = &["english-us", "русский"];
    pub const LISTS: &[&str] = &["english-us-classic", "русский"];
}

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadTracker>()
            .init_resource::<LanguagesResource>()
            .add_asset::<DictionaryAsset>()
            .add_asset::<KeyboardLayoutAsset>()
            .add_asset::<WordListAsset>()
            .add_asset::<LanguageAsset>()
            .add_plugin(RonAssetPlugin::<LanguageAsset>::new(&["lang"]))
            .add_plugin(RonAssetPlugin::<KeyboardLayoutAsset>::new(&["keyboard"]))
            .init_asset_loader::<DictionaryAssetLoader>()
            .init_asset_loader::<WordListAssetLoader>()
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

fn load_assets_system(mut load_tracker: ResMut<LoadTracker>, assets: Res<AssetServer>) {
    paths::KEYBOARDS
        .iter()
        .map(|path| format!("./keyboards/{path}.keyboard"))
        .chain(
            paths::LANGUAGES
                .iter()
                .map(|path| format!("./languages/{path}.lang")),
        )
        .chain(
            paths::LANGUAGES
                .iter()
                .map(|path| format!("./dictionaries/{path}.dict")),
        )
        .chain(
            paths::LISTS
                .iter()
                .map(|path| format!("./lists/{path}.list")),
        )
        .for_each(|path| {
            load_tracker.load(&path, &assets);
        });
}

fn check_loaded_system(
    mut state: ResMut<State<GameState>>,
    mut languages: ResMut<LanguagesResource>,
    load_tracker: ResMut<LoadTracker>,
    assets: Res<AssetServer>,
    dictionaries: Res<Assets<DictionaryAsset>>,
    language_assets: Res<Assets<LanguageAsset>>,
    keyboards: Res<Assets<KeyboardLayoutAsset>>,
    wordlists: Res<Assets<WordListAsset>>,
) {
    if load_tracker.finished(&assets) {
        // build the languages resource
        *languages = LanguagesResource(
            language_assets
                .iter()
                .map(|(lang_handle, l)| {
                    // get actual handles
                    let name = l.name.clone();
                    let keyboards = keyboards
                        .iter()
                        .filter_map(|(kh, k)| {
                            if l.keyboards.contains(&k.name) {
                                Some(keyboards.get_handle(kh))
                            } else {
                                None
                            }
                        })
                        .collect();
                    let wordlists = wordlists
                        .iter()
                        .filter_map(|(wh, w)| {
                            let path = assets.get_handle_path(wh).unwrap();
                            let file_name = path.path().file_stem().unwrap().to_str().unwrap();
                            if l.wordlists.contains(&String::from(file_name)) {
                                Some(wordlists.get_handle(wh))
                            } else {
                                None
                            }
                        })
                        .collect();

                    let language_path = assets.get_handle_path(lang_handle).unwrap();
                    let language_asset_file_name =
                        language_path.path().file_stem().unwrap().to_str().unwrap();

                    let dictionary = dictionaries
                        .iter()
                        .find_map(|(dh, d)| {
                            let dict_path = assets.get_handle_path(dh).unwrap();
                            let dict_asset_file_name =
                                dict_path.path().file_stem().unwrap().to_str().unwrap();
                            if language_asset_file_name == dict_asset_file_name {
                                Some(dictionaries.get_handle(dh))
                            } else {
                                None
                            }
                        })
                        .unwrap();

                    Language {
                        name,
                        keyboards,
                        wordlists,
                        dictionary,
                    }
                })
                .collect(),
        );

        // get english language as default
        if let Some(english) = languages.iter().find(|x| x.name == "english-us") {
            // Set the state to main with some default settings
            state
                .replace(GameState::Main(GameOptions {
                    word: english.get_random_word(&wordlists, 5),
                    language: english.clone(),
                    settings: Settings {
                        ..Default::default()
                    },
                }))
                .ok();
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, PartialEq, Default, Debug, Clone, Eq)]
#[uuid = "fccfcc12-345c-4fa8-adc4-78c5822269f8"]
pub struct DictionaryAsset(Vec<String>);

impl Deref for DictionaryAsset {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, PartialEq, Default, Debug, Clone, Eq)]
#[uuid = "fccfcc12-4252-4fa8-adc4-78c5822269c9"]
pub struct WordListAsset(Vec<String>);

impl Deref for WordListAsset {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, PartialEq, Default, Debug, Clone, Eq)]
#[uuid = "fccfcc12-4252-4fa8-adc4-78c5822269f8"]
pub struct KeyboardLayoutAsset {
    pub name: String,
    pub layout: Vec<Vec<char>>,
}

// Constructed from the language asset but with real handles
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Language {
    pub name: String,
    pub keyboards: Vec<Handle<KeyboardLayoutAsset>>,
    pub wordlists: Vec<Handle<WordListAsset>>,
    pub dictionary: Handle<DictionaryAsset>,
}

impl Language {
    pub fn get_random_word(&self, wordlists: &Assets<WordListAsset>, length: usize) -> String {
        // TODO: support multiple lists
        wordlists
            .get(self.wordlists.first().unwrap())
            .unwrap()
            .0
            .iter()
            .filter(|x| x.chars().count() == length)
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }
    pub fn is_in_dictionary(&self, dictionaries: &Assets<DictionaryAsset>, word: &str) -> bool {
        let dictionary = dictionaries.get(self.dictionary.clone()).unwrap();
        dictionary.contains(&word.to_string())
    }
}

#[derive(Default)]
pub struct LanguagesResource(Vec<Language>);

impl Deref for LanguagesResource {
    type Target = Vec<Language>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(serde::Deserialize, serde::Serialize, TypeUuid, PartialEq, Default, Debug, Clone, Eq)]
#[uuid = "fccfcc12-4252-4fa8-adc4-78c5822269f9"]
pub struct LanguageAsset {
    pub name: String,
    pub keyboards: Vec<String>,
    pub wordlists: Vec<String>,
}

#[derive(Default)]
struct DictionaryAssetLoader;

impl AssetLoader for DictionaryAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let input = String::from_utf8(bytes.to_vec())?;
            let asset = DictionaryAsset(input.split_whitespace().map(String::from).collect());
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["dict"]
    }
}

#[derive(Default)]
struct WordListAssetLoader;

impl AssetLoader for WordListAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let input = String::from_utf8(bytes.to_vec())?;
            let asset = WordListAsset(input.split_whitespace().map(String::from).collect());
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["list"]
    }
}
