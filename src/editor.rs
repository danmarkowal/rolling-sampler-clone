use std::{fmt::Display, sync::Arc};

use nih_plug::prelude::*;
use nih_plug_egui::{EguiState, egui::{self, *}, resizable_window::ResizableWindow};

use crate::RollingSamplerCloneParams;

pub(crate) struct Theme {
    bg_color: Color32,
    fg_color_primary: Color32,
    fg_color_secondary: Color32,
    text_color: Color32
}

#[derive(Enum, PartialEq)]
pub(crate) enum ThemeType {
    Dark,
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

pub(crate) fn update_ui(ctx: &Context, egui_state: &EguiState, params: &RollingSamplerCloneParams, setter: &ParamSetter) {
    ResizableWindow::new("res-wind")
        .min_size(Vec2::new(400.0, 100.0))
        .show(ctx, egui_state, |ui| {
            let factory = UiFactory { 
                theme: params.theme_type.value().theme()
            };

            // We may need to change visuals later instead if we want to change the accent color
            ctx.set_theme(params.theme_type.value().egui_theme());

            Frame::new()
                .fill(factory.theme.bg_color)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height());

                    ui.vertical(|ui| {
                        // We don't want an extra gap between the train and the platform
                        ui.spacing_mut().item_spacing.y = 0.0;

                        factory.top_bar(ui, params, setter);
                        factory.waveform_view(ui);
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
                    ui.label(RichText::new("Rolling Sampler Clone")
                        .color(self.theme.text_color)
                        .strong()
                        .size(12.0));
            
                    ui.separator();
            
                    ui.label(RichText::new("Buffer Size")
                        .color(self.theme.text_color)
                        .size(10.0));

                    ui.add(DragValue::new(&mut 22050).range(0..=44100));

                    ui.add(Button::new("Clear"));

                    ui.separator();

                    ui.menu_button("⚙", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(self.text("Theme"));

                            let mut selected_theme = params.theme_type.value();
                            ComboBox::from_id_salt("theme")
                                .selected_text(self.text(selected_theme.to_string().as_str()))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut selected_theme, ThemeType::Dark,
                                        self.text(ThemeType::Dark.to_string().as_str()));
                                    ui.selectable_value(&mut selected_theme, ThemeType::Light,
                                        self.text(ThemeType::Light.to_string().as_str()));
                                });

                            if selected_theme != params.theme_type.value() {
                                setter.begin_set_parameter(&params.theme_type);
                                setter.set_parameter(&params.theme_type, selected_theme);
                                setter.end_set_parameter(&params.theme_type);
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label(self.text("Clear on Play"));
                            ui.add(Checkbox::without_text(&mut false));
                        });

                        ui.horizontal(|ui| {
                            ui.label(self.text("Truncate Silence"));
                            ui.add(Checkbox::without_text(&mut true))
                        });
                    });
                });
            });
    }
    
    fn waveform_view(&self, ui: &mut Ui) {
        Frame::new()
            .fill(self.theme.fg_color_primary)
            .outer_margin(Margin::same(8))
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());

                let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::empty());
                let start_pos = Vec2::new(response.rect.min.x, (response.rect.min.y + response.rect.max.y) / 2.0);

                let width = response.rect.max.x - response.rect.min.x;
                let max_amplitude = (response.rect.max.y - response.rect.min.y) / 2.0 - 8.0;

                // self.draw_waveform(&res, painter, start_pos, Vec2::new(width, max_amplitude), samples, Stroke::new(1.0, ACCENT_COLOR));
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
            .size(10.0)
            .color(self.theme.text_color)
    }
}
