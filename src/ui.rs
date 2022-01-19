const GREEN: Color32 = Color32::from_rgb(28, 142, 62);
const ORANGE: Color32 = Color32::from_rgb(170, 103, 13);
const GRAY: Color32 = Color32::from_rgb(83, 96, 100);
const DARK_GRAY: Color32 = Color32::from_rgb(10, 10, 15);

use crate::prelude::*;
use bevy_egui::{
    egui::{
        self,
        epaint::{RectShape, TextStyle},
        Color32, Label, Sense, Shape, Widget,
    },
    EguiContext,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::Main).with_system(main_ui_system));
    }
}

pub fn main_ui_system(
    ctx: ResMut<EguiContext>,
    state: Res<State<GameState>>,
    current: Res<CurrentInput>,
    history: Res<History>,
    word: Res<CurrentWord>,
) {
    egui::containers::Area::new("info")
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0f32, 32f32))
        .show(ctx.ctx(), |ui| {
            ui.spacing_mut().item_spacing = egui::Vec2::new(16f32, 8f32);
            ui.vertical(|ui| {
                history.0.iter().for_each(|guess| {
                    ui.add(WordLineWidget {
                        length: guess.0.len(),
                        size: 48f32,
                        contents: &guess.0,
                    });
                });
                ui.add(WordLineWidget {
                    length: word.len(),
                    size: 48f32,
                    contents: &current
                        .contents()
                        .iter()
                        .map(|x| (*x, None::<GuessState>))
                        .collect(),
                });
            })
        });

    egui::containers::Area::new("keyboard")
        .anchor(egui::Align2::CENTER_BOTTOM, egui::Vec2::new(0f32, -200f32))
        .show(ctx.ctx(), |ui| {
            ui.add(KeyboardWidget::Qwerty);
        });
}

pub struct WordBlockWidget<'a> {
    pub character: Option<&'a char>,
    pub state: Option<GuessState>,
    pub size: f32,
}

impl<'a> Widget for WordBlockWidget<'a> {
    fn ui(self, ui: &mut bevy_egui::egui::Ui) -> bevy_egui::egui::Response {
        let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(self.size), Sense::hover());

        let (fill_color, stroke_color) = match self.state {
            Some(state) => match state {
                GuessState::Misplaced => (ORANGE, Color32::TRANSPARENT),
                GuessState::Correct => (GREEN, Color32::TRANSPARENT),
            },
            None => (Color32::TRANSPARENT, GRAY),
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
    pub contents: &'a Vec<(char, Option<GuessState>)>,
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
                    state: contents.map(|(_, s)| *s).unwrap_or(None),
                    size: self.size,
                });
            }
        });

        response
    }
}

pub enum KeyboardWidget {
    Qwerty,
}

pub struct KeyWidget {
    character: char,
    /// The key has already been guessed and is not a part of the word
    invalid: bool,
    size: f32,
}

impl Widget for KeyWidget {
    fn ui(self, ui: &mut bevy_egui::egui::Ui) -> bevy_egui::egui::Response {
        let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(self.size), Sense::hover());

        let (fill_color, stroke_color) = match self.invalid {
            true => (DARK_GRAY, GRAY),
            false => (DARK_GRAY, Color32::WHITE),
        };
        ui.painter().add(RectShape {
            rect,
            corner_radius: 1f32,
            fill: fill_color,
            stroke: egui::Stroke::new(1f32, stroke_color),
        });
        let character = self.character;
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{character}"),
            TextStyle::Button,
            Color32::WHITE,
        );

        response
    }
}

impl Widget for KeyboardWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            KeyboardWidget::Qwerty => {
                let keys = [
                    vec!['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
                    vec!['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'],
                    vec!['z', 'x', 'c', 'v', 'b', 'n', 'm'],
                ];
                ui.spacing_mut().item_spacing = egui::Vec2::new(16f32, 8f32);
                ui.vertical(|ui| {
                    for line in keys {
                        ui.horizontal(|ui| {
                            for key in line {
                                ui.add(KeyWidget {
                                    character: key,
                                    invalid: false,
                                    size: 24f32,
                                });
                            }
                        });
                    }
                })
                .response
            }
        }
    }
}
