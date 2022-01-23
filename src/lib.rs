pub mod assets;
pub mod ui;

use std::ops::Deref;

use bevy::{app::Events, prelude::*, utils::HashMap};
use bevy_egui::egui::Color32;
use prelude::{
    assets::{AssetPlugin, DictionaryAsset, KeyboardLayoutAsset},
    ui::colors::*,
};
use rand::prelude::IteratorRandom;

pub mod prelude {
    pub use super::*;
    pub use assets;
    pub use ui;
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameState::Load)
            .init_resource::<CurrentDictionaryResource>()
            .init_resource::<CurrentInputResource>()
            .init_resource::<CurrentWordResource>()
            .init_resource::<HistoryResource>()
            .init_resource::<CurrentLayoutResource>()
            .init_resource::<CurrentSettingsResource>()
            .add_event::<GameEvent>()
            .add_plugin(AssetPlugin)
            .add_system(process_game_events_system)
            .add_system_set(SystemSet::on_enter(GameState::Main).with_system(game_setup_system))
            .add_system_set(SystemSet::on_update(GameState::Main).with_system(capture_input_system))
            .add_plugin(ui::UiPlugin);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Load,
    Main,
    Menu,
}

#[derive(Debug)]
pub struct CurrentSettingsResource {
    pub max_size: usize,
    pub length: usize,
}

impl Default for CurrentSettingsResource {
    fn default() -> Self {
        Self {
            length: 5,
            max_size: 5,
        }
    }
}

/// Keeps track of current input (guess)
#[derive(Default)]
pub struct CurrentInputResource(Vec<char>);

impl CurrentInputResource {
    pub fn contents(&self) -> &Vec<char> {
        &self.0
    }

    pub fn push(&mut self, character: char, max: usize) {
        if self.contents().len() < max {
            self.0.push(character);
        }
    }

    pub fn backspace(&mut self) {
        self.0.pop();
    }

    pub fn reset(&mut self) {
        self.0.truncate(0);
    }
}

#[derive(Default)]
pub struct CurrentLayoutResource(Handle<KeyboardLayoutAsset>);

#[derive(Default)]
pub struct HistoryResource {
    guesses: Vec<Guess>,
    guessed_char: HashMap<char, GuessState>,
}

impl HistoryResource {
    pub fn clear(&mut self) {
        self.guesses = Default::default();
        self.guessed_char = Default::default();
    }

    pub fn get_guesses(&self) -> &[Guess] {
        &self.guesses
    }
    /// Add a guess to the history
    pub fn guess(&mut self, guess: Guess) {
        // ascertain which chars are guessed by using an empy hashmap
        guess.get().iter().for_each(|(ch, guess_state)| {
            // push guess char to the hashmap. we don't need to do this
            // but it seems useless to calculate every frame

            // when inserting, we must ensure that there are some rules.
            // Correct chars are never re-inserted.
            // Misplaced chars can only be corrected
            if let Some(state) = self.guessed_char.get(ch) {
                match state {
                    GuessState::None => {
                        self.guessed_char.insert(*ch, *guess_state);
                    }
                    GuessState::Missing => {
                        if *guess_state == GuessState::Misplaced
                            || *guess_state == GuessState::Correct
                        {
                            self.guessed_char.insert(*ch, *guess_state);
                        }
                    }
                    GuessState::Misplaced => {
                        if *guess_state == GuessState::Correct {
                            self.guessed_char.insert(*ch, *guess_state);
                        }
                    }
                    GuessState::Correct => (),
                }
            } else {
                self.guessed_char.insert(*ch, *guess_state);
            }
        });
        // push regular guess to the history
        self.guesses.push(guess);
    }

    /// Get a reference to the history resource's guessed char.
    pub fn guessed_chars(&self) -> &HashMap<char, GuessState> {
        &self.guessed_char
    }
}

pub struct Guess(Vec<(char, GuessState)>);

impl Guess {
    pub fn get_chars(&self) -> Vec<char> {
        self.0.iter().map(|(c, _)| *c).collect()
    }

    pub fn get(&self) -> &[(char, GuessState)] {
        &self.0
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GuessState {
    None,
    Missing,
    Misplaced,
    Correct,
}

impl GuessState {
    /// Returns a fill, stroke, and text color
    pub fn colors(&self) -> (Color32, Color32, Color32) {
        match self {
            GuessState::None => (DARK_GRAY, GRAY, Color32::WHITE),
            GuessState::Missing => (DARK_GRAY, GRAY, GRAY),
            GuessState::Misplaced => (ORANGE, Color32::TRANSPARENT, Color32::WHITE),
            GuessState::Correct => (GREEN, Color32::TRANSPARENT, Color32::WHITE),
        }
    }
}

impl ToString for CurrentInputResource {
    fn to_string(&self) -> String {
        self.0.clone().into_iter().collect()
    }
}

#[derive(Default)]
pub struct CurrentDictionaryResource(pub Handle<DictionaryAsset>);

#[derive(Default)]
pub struct CurrentWordResource(pub String);

impl Deref for CurrentWordResource {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn game_setup_system(mut events: EventWriter<GameEvent>) {
    // Reset input resource
    events.send(GameEvent::RandomizeWord);
}

fn capture_input_system(
    mut keyboard_events: EventReader<ReceivedCharacter>,
    mut current: ResMut<CurrentInputResource>,
    mut events: EventWriter<GameEvent>,
    word: Res<CurrentWordResource>,
) {
    for event in keyboard_events.iter() {
        // add typed letters
        if event.char.is_alphabetic() {
            current.push(event.char, word.chars().count());
        } else if event.char == '\u{8}' {
            current.backspace()
        } else if event.char == '\r' {
            events.send(GameEvent::Guess(current.to_string()));
        }
    }
}

pub enum GameEvent {
    Guess(String),
    ChangeDictionary(Handle<DictionaryAsset>),
    RandomizeWord,
}

#[allow(clippy::too_many_arguments)]
fn process_game_events_system(
    mut events: ResMut<Events<GameEvent>>,
    mut history: ResMut<HistoryResource>,
    mut current_input: ResMut<CurrentInputResource>,
    mut current_word: ResMut<CurrentWordResource>,
    mut current_dictionary: ResMut<CurrentDictionaryResource>,
    mut current_layout: ResMut<CurrentLayoutResource>,
    current_settings: Res<CurrentSettingsResource>,
    layouts: Res<Assets<KeyboardLayoutAsset>>,
    dictionaries: Res<Assets<DictionaryAsset>>,
) {
    let mut send_events = vec![];
    events.drain().for_each(|event| match event {
        GameEvent::Guess(guess) => {
            // build a guess and add it to the history (if valid)
            // Proceed if guess is correct length
            if guess.chars().count() == current_word.chars().count() {
                // proceed if guess is in dictionary
                if dictionaries.iter().next().unwrap().1.words.contains(&guess) {
                    // Clone the word and use it as a way to keep track of letters
                    let mut letters: Vec<char> = current_word.clone().chars().collect();

                    // loop over guess for comparison to find correct ones
                    let guess: Vec<(char, GuessState)> = guess
                        .chars()
                        .zip(current_word.chars())
                        .enumerate()
                        .map(|(i, (guess_char, word_char))| {
                            if guess_char == word_char {
                                // remove correct characters from checking pool
                                if let Some(c) = letters.get_mut(i) {
                                    *c = ' ';
                                }
                                (guess_char, GuessState::Correct)
                            } else {
                                // not checked at this stage, set to missing first
                                (guess_char, GuessState::None)
                            }
                        })
                        .collect::<Vec<(char, GuessState)>>()
                        .iter()
                        .map(|(c, state)| {
                            if *state == GuessState::None {
                                if letters.contains(c) {
                                    // remove misplaced characters from checking pool
                                    let pos = letters.iter_mut().position(|x| *x == *c).unwrap();
                                    if let Some(c) = letters.get_mut(pos) {
                                        *c = ' '
                                    }
                                    (*c, GuessState::Misplaced)
                                } else {
                                    (*c, GuessState::Missing)
                                }
                            } else {
                                (*c, *state)
                            }
                        })
                        .collect();

                    // Add guess to history
                    history.guess(Guess(guess));
                    // reset current input
                    current_input.reset();
                }
            }
        }
        GameEvent::ChangeDictionary(new_dictionary) => {
            // clear all history (reset the game)
            history.clear();

            // Change the current dictionary
            current_dictionary.0 = new_dictionary.clone();

            // get the dict asset
            let dictionary = dictionaries.get(new_dictionary).unwrap();

            // also change the current keyboard layout to reflect the dictionary capability
            // TODO: for now just select the first available layout. Should let users choose later.
            current_layout.0 = layouts.get_handle(
                layouts
                    .iter()
                    .find(|x| &x.1.name == dictionary.keyboards.first().unwrap())
                    .map(|x| x.0)
                    .unwrap(),
            );
            send_events.push(GameEvent::RandomizeWord);
        }
        GameEvent::RandomizeWord => {
            // get a new word
            current_word.0 = dictionaries
                .get(current_dictionary.0.clone())
                .unwrap()
                .words
                .iter()
                // TODO: customize size
                .filter(|a| a.chars().count() == current_settings.max_size)
                .choose(&mut rand::thread_rng())
                .unwrap()
                .clone();

            // reset input
            *current_input = CurrentInputResource::default();
        }
    });
    for event in send_events {
        events.send(event);
    }
}

// TODO: could be cool to treat each guess as a sequencer in a playback drone  like the tonal grid thing
