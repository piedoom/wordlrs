pub mod assets;
pub mod ui;

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use bevy::{prelude::*, utils::HashMap};
use bevy_egui::egui::Color32;
use prelude::{
    assets::{AssetPlugin, DictionaryAsset, KeyboardLayoutAsset},
    ui::colors::*,
};

pub mod prelude {
    pub use super::*;
    pub use assets;
    pub use ui;
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state(GameState::Load)
            .init_resource::<CurrentInputResource>()
            .init_resource::<HistoryResource>()
            .init_resource::<Settings>()
            .add_event::<GameEvent>()
            .add_plugin(AssetPlugin)
            .add_system(process_game_events_system)
            .add_system_set(SystemSet::on_enter(GameState::main()).with_system(game_setup_system))
            .add_system_set(
                SystemSet::on_update(GameState::main()).with_system(capture_input_system),
            )
            .add_plugin(ui::UiPlugin);
    }
}

#[allow(clippy::derive_hash_xor_eq)]
#[derive(Debug, Clone, Eq, Hash)]
pub enum GameState {
    Load,
    Main {
        settings: Settings,
        word: String,
        dictionary: Handle<DictionaryAsset>,
    },
    Menu {
        settings: Settings,
        word: String,
        dictionary: Handle<DictionaryAsset>,
    },
    Win {
        settings: Settings,
        word: String,
        dictionary: Handle<DictionaryAsset>,
    },
    Loss {
        settings: Settings,
        word: String,
        dictionary: Handle<DictionaryAsset>,
    },
}

impl PartialEq for GameState {
    /// Set a custom equality method that only compares the enum variant,
    /// ignoring any attached data.
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl GameState {
    #[inline(always)]
    pub fn load() -> Self {
        Self::Load
    }

    #[inline(always)]
    pub fn main() -> Self {
        Self::Main {
            settings: Default::default(),
            word: Default::default(),
            dictionary: Default::default(),
        }
    }

    #[inline(always)]
    pub fn menu() -> Self {
        Self::Menu {
            settings: Default::default(),
            word: Default::default(),
            dictionary: Default::default(),
        }
    }

    #[inline(always)]
    pub fn win() -> Self {
        Self::Win {
            settings: Default::default(),
            word: Default::default(),
            dictionary: Default::default(),
        }
    }

    #[inline(always)]
    pub fn loss() -> Self {
        Self::Loss {
            settings: Default::default(),
            word: Default::default(),
            dictionary: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Settings {
    pub word_length: usize,
    pub max_attempts: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            word_length: 5,
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
pub struct HistoryResource {
    guesses: Vec<Guess>,
    guessed_char: HashMap<char, GuessState>,
}

impl HistoryResource {
    pub fn share_string(&self, word: &str, settings: &Settings) -> String {
        // Hash the word so it isn't given away since we don't have an ID
        let mut hasher = DefaultHasher::new();
        word.hash(&mut hasher);
        let hash = hasher.finish();

        // get number of attempts
        let attempt = self.guesses.len();
        let max_attempts = settings.max_attempts;

        let blocks = self.guesses.iter().fold(String::default(), |acc, x| {
            acc + &x.0.iter().fold(String::default(), |acc, (_, state)| {
                acc + match state {
                    GuessState::None | GuessState::Missing => "⬜",
                    GuessState::Misplaced => "🟨",
                    GuessState::Correct => "🟩",
                }
            }) + "\n"
        });

        format!("wordlrs {hash} {attempt}/{max_attempts}\n{blocks}")
    }

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

    pub fn correct(&self) -> bool {
        !self.0.iter().any(|(_, s)| s != &GuessState::Correct)
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

fn game_setup_system() {
    // nothing to do yet
}

fn capture_input_system(
    mut keyboard_events: EventReader<ReceivedCharacter>,
    #[cfg(target_arch = "wasm32")] keyboard_input: Res<Input<KeyCode>>,
    mut current: ResMut<CurrentInputResource>,
    mut events: EventWriter<GameEvent>,
    state: Res<State<GameState>>,
) {
    if let GameState::Main { word, .. } = state.current() {
        for event in keyboard_events.iter() {
            println!("{}", event.char);
            // add typed letters
            if event.char.is_alphabetic() {
                current.push(event.char, word.chars().count());
            } else if event.char == '\u{8}' {
                current.backspace()
            } else if event.char == '\r' || event.char == '\n' {
                events.send(GameEvent::Guess(current.to_string()));
            }
        }
        // backspace and enter don't work with chars on web
        #[cfg(target_arch = "wasm32")]
        keyboard_input.get_just_pressed().for_each(|k| match k {
            KeyCode::Back => current.backspace(),
            KeyCode::Return => events.send(GameEvent::Guess(current.to_string())),
            _ => (),
        })
    }
}

pub enum GameEvent {
    Guess(String),
}

#[allow(clippy::too_many_arguments)]
fn process_game_events_system(
    mut state: ResMut<State<GameState>>,
    mut events: EventReader<GameEvent>,
    mut history: ResMut<HistoryResource>,
    mut current_input: ResMut<CurrentInputResource>,
    current_settings: Res<Settings>,
    dictionaries: Res<Assets<DictionaryAsset>>,
) {
    if let GameState::Main {
        settings,
        word,
        dictionary,
    } = state.current().clone()
    {
        events.iter().for_each(|event| match event {
            GameEvent::Guess(guess) => {
                // build a guess and add it to the history (if valid)
                // Proceed if guess is correct length
                if guess.chars().count() == word.chars().count() {
                    // proceed if guess is in dictionary
                    if dictionaries
                        .get(dictionary.clone())
                        .unwrap()
                        .contains(guess)
                    {
                        // Clone the word and use it as a way to keep track of letters
                        let mut letters: Vec<char> = word.clone().chars().collect();

                        // loop over guess for comparison to find correct ones
                        let guess: Vec<(char, GuessState)> = guess
                            .chars()
                            .zip(word.chars())
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
                                        let pos =
                                            letters.iter_mut().position(|x| *x == *c).unwrap();
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

                        // check for win/loss state
                        let guesses = history.get_guesses();
                        if let Some(guess) = guesses.last() {
                            // check if correct
                            if guess.correct() {
                                state
                                    .push(GameState::Win {
                                        settings: settings.clone(),
                                        word: word.clone(),
                                        dictionary: dictionary.clone(),
                                    })
                                    .ok();
                            } else {
                                // check if loss
                                if guesses.len() >= current_settings.max_attempts {
                                    state
                                        .push(GameState::Loss {
                                            settings: settings.clone(),
                                            word: word.clone(),
                                            dictionary: dictionary.clone(),
                                        })
                                        .ok();
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

// TODO: could be cool to treat each guess as a sequencer in a playback drone  like the tonal grid thing
