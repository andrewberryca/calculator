#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::fs;
use std::path::PathBuf;

const MAX_HISTORY: usize = 10;

fn history_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_default();
    path.set_file_name("calc_history.txt");
    path
}

fn load_history() -> Vec<HistoryEntry> {
    let path = history_path();
    let Ok(contents) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    contents
        .lines()
        .filter_map(|line| {
            let (expr, result) = line.split_once('\t')?;
            Some(HistoryEntry {
                expression: expr.to_string(),
                result: result.to_string(),
            })
        })
        .collect()
}

fn save_history(history: &[HistoryEntry]) {
    let path = history_path();
    let content: String = history
        .iter()
        .map(|e| format!("{}\t{}", e.expression, e.result))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(&path, content);
}

const CALC_WIDTH: f32 = 320.0;
const HISTORY_WIDTH: f32 = 230.0;
const WINDOW_HEIGHT: f32 = 500.0;

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

struct HistoryEntry {
    expression: String,
    result: String,
}

struct CalcApp {
    display: String,
    expression: String,
    first_operand: Option<f64>,
    operator: Option<char>,
    waiting_for_second: bool,
    just_computed: bool,
    history: Vec<HistoryEntry>,
    show_history: bool,
}

impl CalcApp {
    fn new() -> Self {
        Self {
            display: "0".to_string(),
            expression: String::new(),
            first_operand: None,
            operator: None,
            waiting_for_second: false,
            just_computed: false,
            history: load_history(),
            show_history: false,
        }
    }

    fn add_history(&mut self, expression: String, result: String) {
        self.history.push(HistoryEntry { expression, result });
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }
        save_history(&self.history);
    }
}

impl CalcApp {
    fn input_digit(&mut self, d: char) {
        if self.just_computed {
            self.clear_state();
            self.just_computed = false;
        }
        if self.waiting_for_second {
            self.display = d.to_string();
            self.waiting_for_second = false;
        } else if self.display == "0" {
            self.display = d.to_string();
        } else {
            self.display.push(d);
        }
    }

    fn input_dot(&mut self) {
        if self.just_computed {
            self.clear_state();
            self.just_computed = false;
        }
        if self.waiting_for_second {
            self.display = "0.".to_string();
            self.waiting_for_second = false;
        } else if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    fn op_symbol(op: char) -> &'static str {
        match op {
            '+' => "+",
            '-' => "-",
            '*' => "\u{00D7}",
            '/' => "\u{00F7}",
            _ => "?",
        }
    }

    fn input_operator(&mut self, op: char) {
        if self.display == "Error" {
            return;
        }
        if let Ok(val) = self.display.parse::<f64>() {
            if self.first_operand.is_some() && !self.waiting_for_second {
                self.compute();
                if self.display == "Error" {
                    return;
                }
            }
            let current: f64 = self.display.parse().unwrap_or(val);
            self.expression = format!("{} {}", format_number(current), Self::op_symbol(op));
            self.first_operand = Some(current);
            self.operator = Some(op);
            self.waiting_for_second = true;
            self.just_computed = false;
        }
    }

    fn compute(&mut self) {
        if let (Some(a), Some(op)) = (self.first_operand, self.operator) {
            if let Ok(b) = self.display.parse::<f64>() {
                let expr = format!(
                    "{} {} {}",
                    format_number(a),
                    Self::op_symbol(op),
                    format_number(b)
                );
                self.expression = format!("{} =", expr);
                let result = match op {
                    '+' => Some(a + b),
                    '-' => Some(a - b),
                    '*' => Some(a * b),
                    '/' => {
                        if b == 0.0 {
                            None
                        } else {
                            Some(a / b)
                        }
                    }
                    _ => Some(0.0),
                };
                match result {
                    Some(r) => {
                        let result_str = format_number(r);
                        self.add_history(expr, result_str.clone());
                        self.display = result_str;
                    }
                    None => {
                        self.display = "Error".to_string();
                        self.expression = "Cannot divide by zero".to_string();
                    }
                }
                self.first_operand = None;
                self.operator = None;
                self.waiting_for_second = false;
                self.just_computed = true;
            }
        }
    }

    fn clear_state(&mut self) {
        self.display = "0".to_string();
        self.expression.clear();
        self.first_operand = None;
        self.operator = None;
        self.waiting_for_second = false;
        self.just_computed = false;
    }

    fn clear(&mut self) {
        self.clear_state();
    }

    fn clear_entry(&mut self) {
        self.display = "0".to_string();
    }

    fn backspace(&mut self) {
        if self.display == "Error" || self.just_computed {
            return;
        }
        if self.display.len() > 1 {
            self.display.pop();
        } else {
            self.display = "0".to_string();
        }
    }

    fn toggle_sign(&mut self) {
        if self.display == "Error" || self.display == "0" {
            return;
        }
        if self.display.starts_with('-') {
            self.display.remove(0);
        } else {
            self.display.insert(0, '-');
        }
    }

    fn percent(&mut self) {
        if let Ok(val) = self.display.parse::<f64>() {
            self.display = format_number(val / 100.0);
        }
    }
}

fn format_number(n: f64) -> String {
    if n.is_nan() || n.is_infinite() {
        return "Error".to_string();
    }
    if n == n.floor() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.10}", n);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
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
        if self.show_history {
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
                                self.show_history = false;
                            }
                            if !self.history.is_empty() {
                                if ui.add(
                                    egui::Button::new(egui::RichText::new("Clear").size(12.0).color(text_gray))
                                        .fill(op_bg)
                                        .rounding(4.0),
                                ).clicked() {
                                    self.history.clear();
                                    save_history(&self.history);
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

                    if self.history.is_empty() {
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
                                for entry in self.history.iter().rev() {
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
                    let label = if self.show_history { "History <<" } else { "History >>" };
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new(label).size(12.0).color(text_gray),
                        )
                        .fill(egui::Color32::TRANSPARENT),
                    ).clicked() {
                        self.show_history = !self.show_history;
                    }
                });

                // Expression line (right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(
                        egui::RichText::new(&self.expression)
                            .size(14.0)
                            .color(text_gray),
                    );
                });

                // Main display (right-aligned, large)
                let display_size = if self.display.len() > 12 {
                    24.0
                } else if self.display.len() > 8 {
                    32.0
                } else {
                    46.0
                };
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(
                        egui::RichText::new(&self.display)
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
                if make_btn(ui, "%", btn, op_bg, text_white) { self.percent(); }
                if make_btn(ui, "CE", btn, op_bg, text_white) { self.clear_entry(); }
                if make_btn(ui, "C", btn, op_bg, text_white) { self.clear(); }
                if make_btn(ui, "DEL", btn, op_bg, text_white) { self.backspace(); }
            });

            // Row 2: 1/x  x²  √x  ÷
            ui.horizontal(|ui| {
                if make_btn(ui, "1/x", btn, op_bg, text_white) {
                    if let Ok(val) = self.display.parse::<f64>() {
                        if val == 0.0 {
                            self.display = "Error".to_string();
                            self.expression = "Cannot divide by zero".to_string();
                        } else {
                            let result = format_number(1.0 / val);
                            let expr = format!("1/({})", format_number(val));
                            self.add_history(expr.clone(), result.clone());
                            self.expression = expr;
                            self.display = result;
                            self.just_computed = true;
                        }
                    }
                }
                if make_btn(ui, "x\u{00B2}", btn, op_bg, text_white) {
                    if let Ok(val) = self.display.parse::<f64>() {
                        let result = format_number(val * val);
                        let expr = format!("sqr({})", format_number(val));
                        self.add_history(expr.clone(), result.clone());
                        self.expression = expr;
                        self.display = result;
                        self.just_computed = true;
                    }
                }
                if make_btn(ui, "\u{221A}x", btn, op_bg, text_white) {
                    if let Ok(val) = self.display.parse::<f64>() {
                        if val < 0.0 {
                            self.display = "Error".to_string();
                            self.expression = "Invalid input".to_string();
                        } else {
                            let result = format_number(val.sqrt());
                            let expr = format!("\u{221A}({})", format_number(val));
                            self.add_history(expr.clone(), result.clone());
                            self.expression = expr;
                            self.display = result;
                            self.just_computed = true;
                        }
                    }
                }
                if make_btn(ui, "\u{00F7}", btn, op_bg, text_white) { self.input_operator('/'); }
            });

            // Row 3: 7  8  9  ×
            ui.horizontal(|ui| {
                for d in ['7', '8', '9'] {
                    if make_btn(ui, &d.to_string(), btn, num_bg, text_white) { self.input_digit(d); }
                }
                if make_btn(ui, "\u{00D7}", btn, op_bg, text_white) { self.input_operator('*'); }
            });

            // Row 4: 4  5  6  −
            ui.horizontal(|ui| {
                for d in ['4', '5', '6'] {
                    if make_btn(ui, &d.to_string(), btn, num_bg, text_white) { self.input_digit(d); }
                }
                if make_btn(ui, "\u{2212}", btn, op_bg, text_white) { self.input_operator('-'); }
            });

            // Row 5: 1  2  3  +
            ui.horizontal(|ui| {
                for d in ['1', '2', '3'] {
                    if make_btn(ui, &d.to_string(), btn, num_bg, text_white) { self.input_digit(d); }
                }
                if make_btn(ui, "+", btn, op_bg, text_white) { self.input_operator('+'); }
            });

            // Row 6: ±  0  .  =
            ui.horizontal(|ui| {
                if make_btn(ui, "+/-", btn, op_bg, text_white) { self.toggle_sign(); }
                if make_btn(ui, "0", btn, num_bg, text_white) { self.input_digit('0'); }
                if make_btn(ui, ".", btn, num_bg, text_white) { self.input_dot(); }
                if make_btn(ui, "=", btn, eq_bg, text_dark) { self.compute(); }
            });
        });
    }
}
