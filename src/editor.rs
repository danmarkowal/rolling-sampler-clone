use std::{fmt::Display, ops::RangeInclusive, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}};

use crossbeam::atomic::AtomicCell;
use nih_plug::prelude::*;
use nih_plug_egui::{EguiState, egui::{self, *}, resizable_window::ResizableWindow};
use serde::{Deserialize, Serialize};
use triple_buffer;

use crate::{RollingSamplerCloneParams, buffer_size::{Note, BufferSizeUnit}};

pub(crate) struct EditorState {
    pub waveform_buffer_output: Arc<Mutex<triple_buffer::Output<Vec<f32>>>>
}

pub(crate) struct Theme {
    bg_color: Color32,
    fg_color_primary: Color32,
    fg_color_secondary: Color32,
    text_color: Color32
}

#[derive(Enum, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum ThemeType {
    #[default]
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "light")]
    Light
}

impl Display for ThemeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeType::Dark => write!(f, "Dark"),
            ThemeType::Light => write!(f, "Light")
        }
    }
}

impl ThemeType {
    pub const fn theme(self) -> Theme {
        match self {
            ThemeType::Light => Theme {
                bg_color: Color32::from_rgb(224, 224, 224),
                fg_color_primary: Color32::from_rgb(240, 240, 240),
                fg_color_secondary: Color32::from_rgb(209, 209, 209),
                text_color: Color32::from_rgb(46, 46, 46),
            },
            ThemeType::Dark => Theme {
                bg_color: Color32::from_rgb(15, 15, 15),
                fg_color_primary: Color32::from_rgb(31, 31, 31),
                fg_color_secondary: Color32::from_rgb(46, 46, 46),
                text_color: Color32::from_rgb(209, 209, 209),
            },
        }
    }

    pub const fn egui_theme(self) -> egui::Theme {
        match self {
            ThemeType::Light => egui::Theme::Light,
            ThemeType::Dark => egui::Theme::Dark
        }
    }
}

const ACCENT_COLOR: Color32 = Color32::from_rgb(0, 157, 255);
const SECONDS_RANGE: RangeInclusive<f32> = 0.0..=60.0;
const NOTE_VALUES: [Note; 6] = [
    Note(1, 4),
    Note(1, 2),
    Note(1, 1),
    Note(2, 1),
    Note(4, 1),
    Note(8, 1),
];

pub(crate) fn build_ui(ctx: &Context) {
    // add custom font(s)
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert("lato".to_owned(),
    Arc::new(FontData::from_static(include_bytes!("assets/fonts/Lato-Regular.ttf"))));

    fonts.families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "lato".to_owned());

    ctx.set_fonts(fonts);
}

pub(crate) fn update_ui(ctx: &Context, setter: &ParamSetter, state: &EditorState, egui_state: &EguiState, params: &RollingSamplerCloneParams) {
    ResizableWindow::new("res-wind")
        .min_size(Vec2::new(600.0, 120.0))
        .show(ctx, egui_state, |ui| {
            let theme_type = params.theme_type.clone().load();
            let factory = UiFactory { 
                theme: theme_type.theme()
            };

            // We may need to change visuals later instead if we want to change the accent color
            ctx.set_theme(theme_type.egui_theme());

            Frame::new()
                .fill(factory.theme.bg_color)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height());

                    ui.vertical(|ui| {
                        // We don't want an extra gap between the train and the platform
                        ui.spacing_mut().item_spacing.y = 0.0;

                        factory.top_bar(ui, params, setter);
                        factory.waveform_view(ui, state);
                    });
                });
        });
}

struct UiFactory {
    theme: Theme
}

impl UiFactory {
    fn top_bar(&self, ui: &mut Ui, params: &RollingSamplerCloneParams, setter: &ParamSetter) {
        Frame::new()
            .fill(self.theme.fg_color_primary)
            .inner_margin(Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(16.0);

                ui.horizontal(|ui| {
                    ui.label(self.text("Rolling Sampler Clone").strong());
            
                    ui.separator();
            
                    self.buffer_size_picker(ui, params);

                    ui.add(Button::new("Clear Buffer"));

                    ui.separator();

                    // This is kinda hacky but nothing else worked
                    // Also for some reason it causes the waveform view to change size???
                    ui.add_space(ui.available_width() - 20.0);

                    ui.menu_button("⚙", |ui| {
                        self.theme_picker(ui, params.theme_type.clone());
                        self.checkbox(ui, "Reset on Play", params.clear_on_play.clone()); 
                        self.checkbox(ui, "Trim Silence", params.trim_silence.clone());
                    });
                });
            });
    }

    fn buffer_size_picker(&self, ui: &mut Ui, params: &RollingSamplerCloneParams) {
        ui.horizontal(|ui| {
            ui.label(self.text("Buffer Size"));

            let buffer_size = params.buffer_size.clone();

            match buffer_size.unit.clone().load() {
                BufferSizeUnit::Seconds => self.seconds_picker(ui, buffer_size.seconds.clone()),
                BufferSizeUnit::Notes => self.notes_picker(ui, buffer_size.notes.clone()),
            }
            
            self.buffer_size_unit_picker(ui, buffer_size.unit.clone());
        });
    }

    fn notes_picker(&self, ui: &mut Ui, cell: Arc<AtomicCell<Note>>) {
        let mut notes = cell.load();
        
        ComboBox::from_id_salt("notes")
            .selected_text(self.text(notes.to_string().as_str()))
            .show_ui(ui, |ui| {
                for value in NOTE_VALUES.iter() {
                   ui.selectable_value(&mut notes, *value, self.text(value.to_string().as_str()));
                }
            });

        cell.store(notes);
    }

    fn seconds_picker(&self, ui: &mut Ui, cell: Arc<AtomicF32>) {
        let mut seconds = cell.load(Ordering::Acquire);
        ui.add(DragValue::new(&mut seconds)
            .range(SECONDS_RANGE)
            .fixed_decimals(1)
            .speed(0.01));
        cell.store(seconds, Ordering::Release);
    }

    fn buffer_size_unit_picker(&self, ui: &mut Ui, cell: Arc<AtomicCell<BufferSizeUnit>>) {
        let mut selected = cell.load();

        ComboBox::from_id_salt("buffer-size-unit")
            .selected_text(self.text(selected.to_string().as_str()))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut selected, BufferSizeUnit::Seconds,
                    self.text(BufferSizeUnit::Seconds.to_string().as_str()));
                ui.selectable_value(&mut selected, BufferSizeUnit::Notes,
                    self.text(BufferSizeUnit::Notes.to_string().as_str()));
            });
        
        cell.store(selected);
    }

    fn theme_picker(&self, ui: &mut Ui, cell: Arc<AtomicCell<ThemeType>>) {
        ui.horizontal(|ui| {
            ui.label(self.text("Theme"));

            // Cache this instead of cloning multiple times
            let mut selected = cell.load();

            ComboBox::from_id_salt("theme")
                .selected_text(self.text(selected.to_string().as_str()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, ThemeType::Dark,
                        self.text(ThemeType::Dark.to_string().as_str()));
                    ui.selectable_value(&mut selected, ThemeType::Light,
                        self.text(ThemeType::Light.to_string().as_str()));
                });
            
            cell.store(selected);
        });
    }

    fn checkbox(&self, ui: &mut Ui, label_text: &str, cell: Arc<AtomicBool>) {
        ui.horizontal(|ui| {
            ui.add(Label::new(self.text(label_text)));

            let mut selected = cell.load(Ordering::Acquire);
            ui.add(Checkbox::without_text(&mut selected));
            cell.store(selected, Ordering::Release);
        });
    }
    
    fn waveform_view(&self, ui: &mut Ui, state: &EditorState) {
        Frame::new()
            .fill(self.theme.fg_color_primary)
            .outer_margin(Margin::same(8))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());

                let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
                let start_pos = Vec2::new(response.rect.min.x, (response.rect.min.y + response.rect.max.y) / 2.0);

                let width = response.rect.max.x - response.rect.min.x;
                let max_amplitude = (response.rect.max.y - response.rect.min.y) / 2.0 - 8.0;

                let mut buffer = state.waveform_buffer_output.lock().unwrap();
                self.draw_waveform(&response, &painter, start_pos, Vec2::new(width, max_amplitude), buffer.read(), Stroke::new(1.0, ACCENT_COLOR));
            });
    }

    /// Draws a waveform from a collection of samples
    /// Start pos refers to the left-hand position of the equilibrium line
    /// Size refers to (width, amplitude)
    fn draw_waveform(&self, response: &Response, painter: &Painter, start_pos: Vec2, size: Vec2, samples: &Vec<f32>, stroke: Stroke) {
        // First and last elements are equal to the equilibrium position at their respective x coordinates
        let mut vertices: Vec<Pos2> = Vec::new();
        // Start pos
        vertices.push(start_pos.to_pos2()); 

        // Sample positions
        for (i, sample) in samples.iter().enumerate() {
            let t = (i as f32) / ((samples.len() - 1) as f32);
            let x = emath::lerp(start_pos.x..=(start_pos.x + size.x), t);
            vertices.push(Vec2::new(x, start_pos.y).to_pos2());
        }

        // End pos
        vertices.push((start_pos + Vec2::new(size.x, 0.0)).to_pos2());

        painter.line(vertices, stroke);
    }

    fn text(&self, text: &str) -> RichText {
        RichText::new(text)
            .size(12.0)
            .color(self.theme.text_color)
    }
}
