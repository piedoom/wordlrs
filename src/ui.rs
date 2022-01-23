pub mod colors {
    use super::*;
    pub const GREEN: Color32 = Color32::from_rgb(28, 142, 62);
    pub const ORANGE: Color32 = Color32::from_rgb(170, 103, 13);
    pub const GRAY: Color32 = Color32::from_rgb(83, 96, 100);
    pub const DARK_GRAY: Color32 = Color32::from_rgb(10, 10, 15);
}
use colors::*;
use crate::prelude::*;
use bevy_egui::{
    egui::{
        self,
        epaint::{RectShape, TextStyle},
        Color32, ComboBox, Sense, Widget, TextBuffer,
    },
    EguiContext,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Main).with_system(main_ui_system))
            .add_system_set(SystemSet::on_update(GameState::Menu).with_system(menu_ui_system))
            .add_system_set(SystemSet::on_update(GameState::Win).with_system(win_ui_system))
            .add_system_set(SystemSet::on_update(GameState::Loss).with_system(loss_ui_system));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn main_ui_system(
    mut state: ResMut<State<GameState>>,
    mut keyboard_events: EventWriter<ReceivedCharacter>,
    windows: Res<Windows>,
    ctx: ResMut<EguiContext>,
    current: Res<CurrentInputResource>,
    layout: Res<CurrentLayoutResource>,
    layouts: Res<Assets<KeyboardLayoutAsset>>,
    history: Res<HistoryResource>,
    word: Res<CurrentWordResource>,
) {
    egui::containers::Area::new("info")
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0f32, 32f32))
        .show(ctx.ctx(), |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::new(16f32, 8f32);
            ui.vertical(|ui| {
                history.get_guesses().iter().for_each(|guess| {
                    ui.add(WordLineWidget {
                        length: guess.0.len(),
                        size: 48f32,
                        contents: &guess.0,
                    });
                });
                ui.add(WordLineWidget {
                    length: word.chars().count(),
                    size: 48f32,
                    contents: &current
                        .contents()
                        .iter()
                        .map(|x| (*x, GuessState::None))
                        .collect(),
                });
            })
        });

    // button to change game settings
    egui::containers::Area::new("menu")
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::ZERO)
        .show(ctx.ctx(), |ui| {
            if ui.button("Settings").clicked() {
                state.push(GameState::Menu).ok();
            }
        });

    egui::containers::Area::new("keyboard")
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0f32, -32f32))
        .show(ctx.ctx(), |ui| {
            if let Some(layout) = layouts.get(layout.0.clone()) {
                
                ui.add(KeyboardWidget {
                    layout: layout
                        .layout
                        .iter()
                        .map(|x| x.as_slice())
                        .collect::<Vec<&[char]>>()
                        .as_slice(),
                    onclick: &mut |char| {
                        keyboard_events.send(ReceivedCharacter {
                            id: windows.get_primary().unwrap().id(),
                            char,
                        })
                    },
                    history: &history,
                    key_size: egui::Vec2::splat(48f32),
                    key_spacing: egui::Vec2::splat(8f32),
                });
            }
        });
}

#[allow(clippy::too_many_arguments)]
pub fn menu_ui_system(
    mut current_dictionary: ResMut<CurrentDictionaryResource>,
    mut current_settings: ResMut<CurrentSettingsResource>,
    mut state: ResMut<State<GameState>>,
    mut events: EventWriter<GameEvent>,
    current_word: Res<CurrentWordResource>,
    ctx: ResMut<EguiContext>,
    dictionaries: Res<Assets<DictionaryAsset>>,
) {
    egui::containers::Window::new("Menu")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx.ctx(), |ui| {
            let current_dict_name = &dictionaries.get(current_dictionary.0.clone()).unwrap().name;
            ui.vertical(|ui| {
                ui.heading(current_word.0.clone());
                ComboBox::from_label("Dictionary")
                    .selected_text(current_dict_name)
                    .show_ui(ui, |ui| {
                        dictionaries.iter().for_each(|(handle, dict)| {
                            ui.selectable_value(
                                &mut current_dictionary.0,
                                dictionaries.get_handle(handle),
                                &dict.name,
                            );
                        });
                    });

                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut current_settings.word_length)
                            .speed(0.2)
                            .clamp_range(0.0..=16f32)
                            .fixed_decimals(0)
                            .prefix("Length: ")
                            .suffix(" characters"),
                    );
                    ui.add(
                        egui::DragValue::new(&mut current_settings.guesses)
                            .speed(0.2)
                            .clamp_range(0.0..=12f32)
                            .fixed_decimals(0)
                            .prefix("Guesses: "),
                    );
                });
                if ui.button("Go back").clicked() {
                    state.pop().ok();
                }
                if ui.button("Start game").clicked() {
                    events.send(GameEvent::ChangeDictionary(current_dictionary.0.clone()));
                    state.replace(GameState::Main).ok();
                }
            });
        });
}

fn win_ui_system(ctx: ResMut<EguiContext>, word: Res<CurrentWordResource>, mut events: EventWriter<GameEvent>, mut history: ResMut<HistoryResource>, mut state: ResMut<State<GameState>>){
    egui::containers::Window::new("Win")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx.ctx(), |ui| {
            ui.label("Win");
            ui.label(format!("The word was: {}", word.0));
            if ui.button("New game").clicked() {
                events.send(GameEvent::RandomizeWord);
                history.clear();
                state.replace(GameState::Main).ok();
            }
        });
}
fn loss_ui_system(ctx: ResMut<EguiContext>, word: Res<CurrentWordResource>, mut events: EventWriter<GameEvent>, mut history: ResMut<HistoryResource>, mut state: ResMut<State<GameState>>){
    egui::containers::Window::new("Loss")
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx.ctx(), |ui| {
            ui.label("Loss");
            if ui.button("Retry").clicked() {
                history.clear();
                state.replace(GameState::Main).ok();
            }
            if ui.button("New game").clicked() {
                events.send(GameEvent::RandomizeWord);
                history.clear();
                state.replace(GameState::Main).ok();
            }
        });
}

pub struct WordBlockWidget<'a> {
    pub character: Option<&'a char>,
    pub state: GuessState,
    pub size: f32,
}

impl<'a> Widget for WordBlockWidget<'a> {
    fn ui(self, ui: &mut bevy_egui::egui::Ui) -> bevy_egui::egui::Response {
        let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(self.size), Sense::hover());

        let (fill_color, stroke_color) = match self.state {
            GuessState::Misplaced => (ORANGE, Color32::TRANSPARENT),
            GuessState::None | GuessState::Missing => (Color32::TRANSPARENT, GRAY),
            GuessState::Correct => (GREEN, Color32::TRANSPARENT),
        };

        ui.painter().add(RectShape {
            rect,
            corner_radius: 3f32,
            fill: fill_color,
            stroke: egui::Stroke::new(2f32, stroke_color),
        });
        if let Some(character) = self.character {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("{character}"),
                TextStyle::Heading,
                Color32::WHITE,
            );
        }

        response
    }
}

pub struct WordLineWidget<'a> {
    pub contents: &'a Vec<(char, GuessState)>,
    pub length: usize,
    pub size: f32,
}

impl<'a> Widget for WordLineWidget<'a> {
    fn ui(self, ui: &mut bevy_egui::egui::Ui) -> bevy_egui::egui::Response {
        let (_, response) = ui.allocate_at_least(egui::Vec2::splat(0f32), Sense::hover());
        ui.horizontal(|ui| {
            for x in 0..self.length {
                let contents = self.contents.get(x);
                ui.add(WordBlockWidget {
                    character: contents.map(|(c, _)| c),
                    state: contents.map(|(_, s)| *s).unwrap_or(GuessState::None),
                    size: self.size,
                });
            }
        });

        response
    }
}

pub struct KeyboardWidget<'a, F>
where
    F: FnMut(char),
{
    layout: &'a [&'a [char]],
    onclick: &'a mut F,
    history: &'a HistoryResource,
    key_size: egui::Vec2,
    key_spacing: egui::Vec2,
}

impl<'a, F> Widget for KeyboardWidget<'a, F>
where
    F: FnMut(char),
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Get the UI dimensions of a keyboard line
        let get_line_size = |length: usize| -> egui::Vec2 {
            egui::Vec2::new(
                // multiply the total number of keys in a row times the key size...
                (self.key_size.x * (length as f32)) + 
                // and also add some spacing, subtracting 1 from the length to acccount for
                // how things get laid out.
                    (self.key_spacing.x * (length - 1) as f32),
                // The Y size is the same, plus our spacing in one direction.
                self.key_size.y + self.key_spacing.y,
            )
        };

        // find the size of each line
        let line_sizes: Vec<egui::Vec2> = self
            .layout
            .iter()
            .map(|line| {
                // calculate the total width of the line, including margin
                get_line_size(line.len())
            })
            .collect();
            

        // allocate a rect that is the size required. X is based off of
        // the longest line's length
        let mut line_widths_sorted = line_sizes.clone();
        line_widths_sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        // Get total width of keyboard
        let max_x = line_widths_sorted.last().unwrap().x;
        // Get total height of keyboard
        let max_y = line_sizes.iter().fold(0f32, |acc, v| acc + v.y);

        let (resp, _) = ui.allocate_painter(egui::Vec2::new(max_x, max_y), Sense::hover());
        for (i_line, line) in self.layout.iter().enumerate() {
            // For rows other than the longest, find the width and subtract from max_x
            // and div by two to find the offset
            let line_size = line_sizes[i_line];
            let line_offset = (max_x - line_size.x) / 2f32;
            for (i_char, character) in line.iter().enumerate() {
                // get the rect where the key will reside
                let rect = egui::Rect::from_min_max(
                    resp.rect.left_top()
                        + egui::Vec2::new(
                                // Calculate the starting position based on the size
                            (i_char as f32 * self.key_size.x) + 
                            // add offset to the line to compensate for smaller lines being centered
                            line_offset + 
                            // add spacing in-between characters
                            (i_char as f32 * self.key_spacing.x),
                            // Same for the Y direction but a bit simpler
                            (i_line as f32 * self.key_size.y) + (i_line as f32 * self.key_spacing.y),
                        ),
                    resp.rect.left_top()
                        + egui::Vec2::new(
                            ((i_char + 1) as f32 * self.key_size.x) + line_offset + (i_char as f32 * self.key_spacing.x),
                            ((i_line + 1) as f32 * self.key_size.y) + (i_line as f32 * self.key_spacing.y),
                        )
                    );

                let key = ui.add(KeyWidget { character, state: self.history.guessed_chars().get(character).unwrap_or(&GuessState::None), rect: &rect });
                if key.clicked() {
                    (self.onclick)(*character);
                }
            }
        }
        resp
    }
}

pub struct KeyLineWidget<'a> {
    pub characters: &'a [char],
    pub size: f32,
}

pub struct KeyWidget<'a>
{
    pub character: &'a char,
    pub state: &'a GuessState,
    pub rect: &'a egui::Rect,

}

impl<'a> Widget for KeyWidget<'a> where{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let resp = ui.allocate_rect(*self.rect, Sense::click());
        let (fill_color, stroke_color, text_color) = self.state.colors();
        ui.painter().rect(
            resp.rect,
            3f32,
            fill_color,
            egui::Stroke::new(1f32, stroke_color),
        );
        ui.painter().text(
            resp.rect.center(),
            egui::Align2::CENTER_CENTER,
            self.character.to_string(),
            TextStyle::Heading,
            text_color,
        );
    
        resp
    }
}
