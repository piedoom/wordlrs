pub mod assets;
pub mod ui;

use std::ops::Deref;

use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadState, LoadedAsset},
    input::keyboard::KeyboardInput,
    prelude::*,
    reflect::TypeUuid,
    ui::UiPlugin,
};
use prelude::assets::{AssetPlugin, WordListAsset, WordListAssetLoader};
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
            .init_resource::<CurrentInput>()
            .init_resource::<CurrentWord>()
            .init_resource::<History>()
            .add_event::<GameEvent>()
            .add_system_set(SystemSet::on_enter(GameState::Main).with_system(game_setup_system))
            .add_system_set(
                SystemSet::on_update(GameState::Main)
                    .with_system(capture_input_system)
                    .with_system(process_game_events_system),
            )
            .add_plugin(AssetPlugin)
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
pub struct GameSettings {
    pub word: String,
    pub max_guesses: usize,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            word: Default::default(),
            max_guesses: 6,
        }
    }
}

/// Keeps track of current input (guess)
#[derive(Default)]
pub struct CurrentInput(Vec<char>);

impl CurrentInput {
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
pub struct History(Vec<Guess>);

pub struct Guess(Vec<(char, Option<GuessState>)>);

#[derive(Copy, Clone)]
pub enum GuessState {
    Misplaced,
    Correct,
}

impl ToString for CurrentInput {
    fn to_string(&self) -> String {
        self.0.clone().into_iter().collect()
    }
}

#[derive(Default)]
pub struct CurrentWord(pub String);

impl Deref for CurrentWord {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn game_setup_system(mut cmd: Commands, words: Res<Assets<WordListAsset>>) {
    // Reset input resource
    cmd.insert_resource(CurrentWord(get_random_word(&words, 5)));
    cmd.insert_resource(CurrentInput::default());
}

fn capture_input_system(
    mut keyboard_events: EventReader<ReceivedCharacter>,
    mut current: ResMut<CurrentInput>,
    mut events: EventWriter<GameEvent>,
    word: Res<CurrentWord>,
) {
    for event in keyboard_events.iter() {
        // add typed letters
        if event.char.is_alphabetic() {
            current.push(event.char, word.len());
        } else if event.char == '\u{8}' {
            current.backspace()
        } else if event.char == '\r' {
            events.send(GameEvent::Guess(current.to_string()));
        }
    }
}

fn get_random_word(words: &Assets<WordListAsset>, length: usize) -> String {
    words
        .iter()
        .next()
        .unwrap()
        .1
         .0
        .iter()
        .filter(|a| a.chars().count() == length)
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone()
}

pub enum GameEvent {
    Guess(String),
}

fn process_game_events_system(
    mut events: EventReader<GameEvent>,
    mut history: ResMut<History>,
    mut current: ResMut<CurrentInput>,
    words: Res<Assets<WordListAsset>>,
    word: Res<CurrentWord>,
) {
    events.iter().for_each(|event| match event {
        GameEvent::Guess(guess) => {
            // build a guess and add it to the history (if valid)
            // Proceed if guess is correct length
            if guess.chars().count() == word.chars().count() {
                // proceed if guess is in dictionary
                if words.iter().next().unwrap().1 .0.contains(guess) {
                    // Clone the word and use it as a way to keep track of letters
                    let mut letters: Vec<char> = word.clone().chars().collect();

                    // loop over guess for comparison to find correct ones
                    let guess: Vec<(char, Option<GuessState>)> = guess
                        .chars()
                        .zip(word.chars())
                        .enumerate()
                        .map(|(i, (guess_char, word_char))| {
                            if guess_char == word_char {
                                // remove correct characters from checking pool
                                if let Some(c) = letters.get_mut(i) {
                                    *c = ' '
                                }
                                (guess_char, Some(GuessState::Correct))
                            } else {
                                // not checked at this stage
                                (guess_char, None)
                            }
                        })
                        .collect::<Vec<(char, Option<GuessState>)>>()
                        .iter()
                        .map(|(c, state)| {
                            if letters.contains(c) {
                                // remove correct characters from checking pool
                                let pos = letters.iter_mut().position(|x| *x == *c).unwrap();
                                if let Some(c) = letters.get_mut(pos) {
                                    *c = ' '
                                }
                                (*c, Some(GuessState::Misplaced))
                            } else {
                                (*c, *state)
                            }
                        })
                        .collect();

                    // Add guess to history
                    history.0.push(Guess(guess));
                    // reset current input
                    current.reset();
                }
            }
        }
    });
}
