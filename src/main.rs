#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use calculator::CalcApp as LibCalcApp;
use calculator::save_history;

const CALC_WIDTH: f32 = 320.0;
const HISTORY_WIDTH: f32 = 230.0;
const WINDOW_HEIGHT: f32 = 500.0;

// Wrapper type to implement eframe::App for CalcApp
struct CalcApp {
    inner: LibCalcApp,
}

impl CalcApp {
    fn new() -> Self {
        Self {
            inner: LibCalcApp::new(),
        }
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([CALC_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([CALC_WIDTH, WINDOW_HEIGHT])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Calculator",
        options,
        Box::new(|_cc| Ok(Box::new(CalcApp::new()))),
    )
}

impl eframe::App for CalcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let bg = egui::Color32::from_rgb(32, 32, 32);
        let num_bg = egui::Color32::from_rgb(59, 59, 59);
        let op_bg = egui::Color32::from_rgb(50, 50, 50);
        let eq_bg = egui::Color32::from_rgb(118, 185, 237);
        let text_white = egui::Color32::WHITE;
        let text_gray = egui::Color32::from_rgb(150, 150, 150);
        let text_dark = egui::Color32::from_rgb(32, 32, 32);
        let history_bg = egui::Color32::from_rgb(40, 40, 40);
        let divider_color = egui::Color32::from_rgb(55, 55, 55);

        // --- Right side panel: History (collapsible) ---
        if self.inner.show_history {
            let target_w = CALC_WIDTH + HISTORY_WIDTH;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(target_w, WINDOW_HEIGHT)));

            let side_frame = egui::Frame::default()
                .fill(bg)
                .inner_margin(egui::Margin::same(8.0));

            egui::SidePanel::right("history_panel")
                .exact_width(HISTORY_WIDTH - 10.0)
                .frame(side_frame)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("History")
                                .size(16.0)
                                .color(text_white)
                                .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(
                                egui::Button::new(egui::RichText::new("<<").size(14.0).color(text_gray))
                                    .fill(egui::Color32::TRANSPARENT),
                            ).clicked() {
                                self.inner.show_history = false;
                            }
                            if !self.inner.history.is_empty() {
                                if ui.add(
                                    egui::Button::new(egui::RichText::new("Clear").size(12.0).color(text_gray))
                                        .fill(op_bg)
                                        .rounding(4.0),
                                ).clicked() {
                                    self.inner.history.clear();
                                    save_history(&self.inner.history);
                                }
                            }
                        });
                    });
                    ui.add_space(4.0);

                    let rect = ui.available_rect_before_wrap();
                    ui.painter().line_segment(
                        [
                            egui::pos2(rect.left(), rect.top()),
                            egui::pos2(rect.right(), rect.top()),
                        ],
                        egui::Stroke::new(1.0, divider_color),
                    );
                    ui.add_space(6.0);

                    if self.inner.history.is_empty() {
                        ui.add_space(30.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new("No history yet")
                                    .size(13.0)
                                    .color(text_gray),
                            );
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for entry in self.inner.history.iter().rev() {
                                    egui::Frame::default()
                                        .fill(history_bg)
                                        .rounding(4.0)
                                        .inner_margin(egui::Margin::same(8.0))
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Min),
                                                |ui| {
                                                    ui.label(
                                                        egui::RichText::new(&entry.expression)
                                                            .size(12.0)
                                                            .color(text_gray),
                                                    );
                                                },
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Min),
                                                |ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "= {}",
                                                            &entry.result
                                                        ))
                                                        .size(16.0)
                                                        .color(text_white)
                                                        .strong(),
                                                    );
                                                },
                                            );
                                        });
                                    ui.add_space(2.0);
                                }
                            });
                    }
                });
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(CALC_WIDTH, WINDOW_HEIGHT)));
        }

        // --- Left side: Calculator ---
        let panel_frame = egui::Frame::default()
            .fill(bg)
            .inner_margin(egui::Margin::same(4.0));

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            // Display area
            ui.vertical(|ui| {
                // History toggle button row
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    let label = if self.inner.show_history { "History <<" } else { "History >>" };
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new(label).size(12.0).color(text_gray),
                        )
                        .fill(egui::Color32::TRANSPARENT),
                    ).clicked() {
                        self.inner.show_history = !self.inner.show_history;
                    }
                });

                // Expression line (right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(
                        egui::RichText::new(&self.inner.expression)
                            .size(14.0)
                            .color(text_gray),
                    );
                });

                // Main display (right-aligned, large)
                let display_size = if self.inner.display.len() > 12 {
                    24.0
                } else if self.inner.display.len() > 8 {
                    32.0
                } else {
                    46.0
                };
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(
                        egui::RichText::new(&self.inner.display)
                            .size(display_size)
                            .color(text_white)
                            .strong(),
                    );
                });

                ui.add_space(8.0);
            });

            // --- Button grid ---
            let spacing = 2.0;
            ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

            let btn_w = (ui.available_width() - 3.0 * spacing) / 4.0;
            let btn_h = (ui.available_height() - 5.0 * spacing) / 6.0;
            let btn = egui::vec2(btn_w, btn_h);
            let font_size = 20.0;

            let make_btn = |ui: &mut egui::Ui,
                            text: &str,
                            size: egui::Vec2,
                            fill: egui::Color32,
                            text_color: egui::Color32|
             -> bool {
                ui.add(
                    egui::Button::new(
                        egui::RichText::new(text).size(font_size).color(text_color),
                    )
                    .fill(fill)
                    .rounding(4.0)
                    .min_size(size),
                )
                .clicked()
            };

            // Row 1: %  CE  C  DEL
            ui.horizontal(|ui| {
                if make_btn(ui, "%", btn, op_bg, text_white) { self.inner.percent(); }
                if make_btn(ui, "CE", btn, op_bg, text_white) { self.inner.clear_entry(); }
                if make_btn(ui, "C", btn, op_bg, text_white) { self.inner.clear(); }
                if make_btn(ui, "DEL", btn, op_bg, text_white) { self.inner.backspace(); }
            });

            // Row 2: 1/x  x²  √x  ÷
            ui.horizontal(|ui| {
                if make_btn(ui, "1/x", btn, op_bg, text_white) {
                    if let Ok(val) = self.inner.display.parse::<f64>() {
                        if val == 0.0 {
                            self.inner.display = "Error".to_string();
                            self.inner.expression = "Cannot divide by zero".to_string();
                        } else {
                            let result = calculator::format_number(1.0 / val);
                            let expr = format!("1/({})", calculator::format_number(val));
                            self.inner.add_history(expr.clone(), result.clone());
                            self.inner.expression = expr;
                            self.inner.display = result;
                            self.inner.just_computed = true;
                        }
                    }
                }
                if make_btn(ui, "x\u{00B2}", btn, op_bg, text_white) {
                    if let Ok(val) = self.inner.display.parse::<f64>() {
                        let result = calculator::format_number(val * val);
                        let expr = format!("sqr({})", calculator::format_number(val));
                        self.inner.add_history(expr.clone(), result.clone());
                        self.inner.expression = expr;
                        self.inner.display = result;
                        self.inner.just_computed = true;
                    }
                }
                if make_btn(ui, "\u{221A}x", btn, op_bg, text_white) {
                    if let Ok(val) = self.inner.display.parse::<f64>() {
                        if val < 0.0 {
                            self.inner.display = "Error".to_string();
                            self.inner.expression = "Invalid input".to_string();
                        } else {
                            let result = calculator::format_number(val.sqrt());
                            let expr = format!("\u{221A}({})", calculator::format_number(val));
                            self.inner.add_history(expr.clone(), result.clone());
                            self.inner.expression = expr;
                            self.inner.display = result;
                            self.inner.just_computed = true;
                        }
                    }
                }
                if make_btn(ui, "\u{00F7}", btn, op_bg, text_white) { self.inner.input_operator('/'); }
            });

            // Row 3: 7  8  9  ×
            ui.horizontal(|ui| {
                for d in ['7', '8', '9'] {
                    if make_btn(ui, &d.to_string(), btn, num_bg, text_white) { self.inner.input_digit(d); }
                }
                if make_btn(ui, "\u{00D7}", btn, op_bg, text_white) { self.inner.input_operator('*'); }
            });

            // Row 4: 4  5  6  −
            ui.horizontal(|ui| {
                for d in ['4', '5', '6'] {
                    if make_btn(ui, &d.to_string(), btn, num_bg, text_white) { self.inner.input_digit(d); }
                }
                if make_btn(ui, "\u{2212}", btn, op_bg, text_white) { self.inner.input_operator('-'); }
            });

            // Row 5: 1  2  3  +
            ui.horizontal(|ui| {
                for d in ['1', '2', '3'] {
                    if make_btn(ui, &d.to_string(), btn, num_bg, text_white) { self.inner.input_digit(d); }
                }
                if make_btn(ui, "+", btn, op_bg, text_white) { self.inner.input_operator('+'); }
            });

            // Row 6: ±  0  .  =
            ui.horizontal(|ui| {
                if make_btn(ui, "+/-", btn, op_bg, text_white) { self.inner.toggle_sign(); }
                if make_btn(ui, "0", btn, num_bg, text_white) { self.inner.input_digit('0'); }
                if make_btn(ui, ".", btn, num_bg, text_white) { self.inner.input_dot(); }
                if make_btn(ui, "=", btn, eq_bg, text_dark) { self.inner.compute(); }
            });
        });
    }
}
