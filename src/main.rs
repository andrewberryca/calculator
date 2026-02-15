#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use eframe::egui::{Color32, Pos2, Stroke};
use calculator::CalcApp as LibCalcApp;
use calculator::save_history;
use rodio::{OutputStream, Sink, Decoder};

const CALC_WIDTH: f32 = 320.0;
const HISTORY_WIDTH: f32 = 230.0;
const WINDOW_HEIGHT: f32 = 500.0;
const BLAZE_DURATION: f32 = 20.0;

/// Embedded 420 audio clip (first 20 seconds)
const BLAZE_AUDIO: &[u8] = include_bytes!("../assets/blaze_mono.wav");

/// Play the embedded audio clip on a background thread
fn play_blaze_melody() {
    std::thread::spawn(|| {
        let Ok((_stream, handle)) = OutputStream::try_default() else { return };
        let Ok(sink) = Sink::try_new(&handle) else { return };
        let cursor = std::io::Cursor::new(BLAZE_AUDIO);
        let Ok(source) = Decoder::new(cursor) else { return };
        sink.append(source);
        sink.sleep_until_end();
    });
}

/// Convert HSV (h: 0-360, s: 0-1, v: 0-1) to Color32
fn hsv_to_color32(h: f32, s: f32, v: f32, a: u8) -> Color32 {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    Color32::from_rgba_unmultiplied(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
        a,
    )
}

/// Draw a single serrated cannabis leaflet (realistic jagged edges + pointed tip)
fn serrated_leaflet_points(
    cx: f32, cy: f32, angle: f32, length: f32, width: f32, teeth: usize,
) -> Vec<Pos2> {
    let mut pts = Vec::new();
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let segs = teeth * 2 + 1;

    let transform = |along: f32, across: f32| -> Pos2 {
        Pos2::new(
            cx + along * cos_a - across * sin_a,
            cy + along * sin_a + across * cos_a,
        )
    };

    // Envelope: widest around 35% of length, tapers to sharp tip
    let envelope = |t: f32| -> f32 {
        let rise = (t / 0.35).min(1.0);
        let fall = if t > 0.35 { ((1.0 - t) / 0.65).max(0.0) } else { 1.0 };
        width * rise * fall.powf(0.6)
    };

    // Right side (positive across) with serrations
    for i in 0..=segs {
        let t = i as f32 / segs as f32;
        let base_w = envelope(t);
        // Serration: every other segment pokes outward
        let serration = if i % 2 == 1 && t > 0.1 && t < 0.92 {
            base_w * 0.25
        } else {
            0.0
        };
        pts.push(transform(t * length, base_w + serration));
    }
    // Tip
    pts.push(transform(length, 0.0));
    // Left side (negative across) with serrations — reverse
    for i in (0..=segs).rev() {
        let t = i as f32 / segs as f32;
        let base_w = envelope(t);
        let serration = if i % 2 == 1 && t > 0.1 && t < 0.92 {
            base_w * 0.25
        } else {
            0.0
        };
        pts.push(transform(t * length, -(base_w + serration)));
    }
    pts
}

/// Draw central vein line along a leaflet
fn draw_leaflet_vein(
    painter: &egui::Painter, cx: f32, cy: f32, angle: f32, length: f32,
    color: Color32,
) {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let p0 = Pos2::new(cx, cy);
    let p1 = Pos2::new(cx + length * cos_a, cy + length * sin_a);
    painter.line_segment([p0, p1], Stroke::new(1.5, color));
}

/// Draw the 420 Easter egg: rainbow background + realistic cannabis leaf + smoke
fn draw_blaze_overlay(ui: &mut egui::Ui, t: f32) {
    let rect = ui.max_rect();
    let painter = ui.painter();
    let pi = std::f32::consts::PI;

    // Fade: ramp up 0.5s, hold, fade out last 1.5s
    let alpha = if t < 0.5 {
        t / 0.5
    } else if t > BLAZE_DURATION - 1.5 {
        (BLAZE_DURATION - t) / 1.5
    } else {
        1.0
    }.clamp(0.0, 1.0);

    let a = (alpha * 200.0) as u8;

    // --- Rainbow background stripes that scroll ---
    let stripe_count = 14;
    let stripe_h = rect.height() / stripe_count as f32;
    for i in 0..stripe_count {
        let hue = ((i as f32 / stripe_count as f32) * 360.0 + t * 120.0) % 360.0;
        let color = hsv_to_color32(hue, 0.8, 0.9, a / 2);
        let y0 = rect.top() + i as f32 * stripe_h;
        let stripe_rect = egui::Rect::from_min_size(
            Pos2::new(rect.left(), y0),
            egui::vec2(rect.width(), stripe_h + 1.0),
        );
        painter.rect_filled(stripe_rect, 0.0, color);
    }

    // --- Cannabis leaf ---
    let cx = rect.center().x;
    let cy = rect.center().y - 10.0;
    let base_size = 100.0 + 15.0 * (t * 2.0).sin(); // much bigger
    let rotation = t * 0.25; // gentle sway

    // 7 leaflets: angles relative to straight up, lengths, widths
    // Real cannabis: center longest, pairs get shorter and wider-angled
    let leaflets: [(f32, f32, f32); 7] = [
        (-pi / 2.0,            1.0,  1.0),   // center (straight up)
        (-pi / 2.0 - 0.40,     0.88, 0.85),  // upper-left
        (-pi / 2.0 + 0.40,     0.88, 0.85),  // upper-right
        (-pi / 2.0 - 0.82,     0.72, 0.72),  // mid-left
        (-pi / 2.0 + 0.82,     0.72, 0.72),  // mid-right
        (-pi / 2.0 - 1.28,     0.48, 0.55),  // lower-left
        (-pi / 2.0 + 1.28,     0.48, 0.55),  // lower-right
    ];

    // Pulsing bright green
    let green_pulse = 0.7 + 0.3 * (t * 4.0).sin();
    let leaf_green = Color32::from_rgba_unmultiplied(
        (20.0 + 30.0 * green_pulse) as u8,
        (180.0 + 60.0 * green_pulse) as u8,
        (15.0 + 25.0 * green_pulse) as u8,
        (alpha * 240.0) as u8,
    );
    let dark_green = Color32::from_rgba_unmultiplied(
        15, (120.0 * green_pulse) as u8, 10, (alpha * 200.0) as u8,
    );
    let vein_color = Color32::from_rgba_unmultiplied(
        30, (100.0 + 40.0 * green_pulse) as u8, 20, (alpha * 180.0) as u8,
    );

    // Draw each leaflet: dark outline layer, main green, rainbow shimmer, vein
    for (i, &(angle, len_f, wid_f)) in leaflets.iter().enumerate() {
        let a_rot = angle + rotation;
        let length = base_size * len_f;
        let width = base_size * 0.13 * wid_f;
        let teeth = 6 + (len_f * 4.0) as usize; // more teeth on longer leaflets

        // Dark green outline/shadow (slightly larger)
        let pts_shadow = serrated_leaflet_points(cx, cy, a_rot, length * 1.03, width * 1.15, teeth);
        painter.add(egui::Shape::convex_polygon(pts_shadow, dark_green, Stroke::NONE));

        // Main bright green leaflet
        let pts = serrated_leaflet_points(cx, cy, a_rot, length, width, teeth);
        painter.add(egui::Shape::convex_polygon(pts, leaf_green, Stroke::NONE));

        // Rainbow shimmer (inner, semi-transparent)
        let hue = ((i as f32 / 7.0) * 360.0 + t * 200.0) % 360.0;
        let shimmer = hsv_to_color32(hue, 1.0, 1.0, (alpha * 70.0) as u8);
        let pts_inner = serrated_leaflet_points(cx, cy, a_rot, length * 0.85, width * 0.5, teeth);
        painter.add(egui::Shape::convex_polygon(pts_inner, shimmer, Stroke::NONE));

        // Central vein
        draw_leaflet_vein(painter, cx, cy, a_rot, length * 0.92, vein_color);
    }

    // --- Short stem ---
    let stem_angle = pi / 2.0 + rotation; // pointing down
    let stem_len = base_size * 0.35; // short stem
    let stem_start = Pos2::new(cx, cy);
    let stem_end = Pos2::new(
        cx + stem_len * stem_angle.cos(),
        cy + stem_len * stem_angle.sin(),
    );
    let stem_color = Color32::from_rgba_unmultiplied(
        30, (140.0 * green_pulse) as u8, 20, (alpha * 220.0) as u8,
    );
    painter.line_segment([stem_start, stem_end], Stroke::new(4.0, stem_color));

    // --- Floating "~ 420 ~" text ---
    let text_hue = (t * 150.0) % 360.0;
    let text_color = hsv_to_color32(text_hue, 1.0, 1.0, (alpha * 255.0) as u8);
    let text_y = cy + base_size * 0.55 + 8.0 * (t * 3.0).sin();
    painter.text(
        Pos2::new(cx, text_y),
        egui::Align2::CENTER_CENTER,
        "~ 420 ~",
        egui::FontId::proportional(40.0),
        text_color,
    );

    // --- Lots of smoke particles ---
    // Multiple layers: big slow wisps + small fast particles
    let smoke_configs: &[(usize, f32, f32, f32, f32)] = &[
        // (count, speed, spread_x, max_size, base_alpha)
        (12, 30.0, 80.0,  10.0, 90.0),   // big slow wisps
        (18, 55.0, 120.0,  5.0, 60.0),   // medium particles
        (10, 80.0, 50.0,   3.0, 40.0),   // small fast sparks
    ];
    for &(count, speed, spread, max_size, base_a) in smoke_configs {
        for i in 0..count {
            let seed = i as f32 * 2.37;
            let phase = seed + t * 1.2;
            let drift_x = (phase * 0.7 + seed).sin() * spread;
            let rise = (t * speed + i as f32 * (rect.height() / count as f32)) % rect.height();
            let px = cx + drift_x;
            let py = rect.bottom() - rise;
            let life = rise / rect.height(); // 0 at bottom, 1 at top
            let size = max_size * (0.3 + 0.7 * (seed * 3.1).sin().abs()) * (1.0 - life * 0.5);
            let fade = (1.0 - life).powf(0.8);
            let smoke_a = (alpha * base_a * fade) as u8;
            // Slight color tint: warm gray to white
            let gray = 180 + (40.0 * (seed * 1.7).sin().abs()) as u8;
            painter.circle_filled(
                Pos2::new(px, py),
                size,
                Color32::from_rgba_unmultiplied(gray, gray, gray.saturating_sub(10), smoke_a),
            );
        }
    }
}

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

        // --- 420 Easter egg animation ---
        let blaze_t = if self.inner.blaze_it {
            if let Some(start) = self.inner.blaze_start {
                // Play the melody once on first frame
                if !self.inner.blaze_sound_played {
                    self.inner.blaze_sound_played = true;
                    play_blaze_melody();
                }
                let elapsed = start.elapsed().as_secs_f32();
                if elapsed > BLAZE_DURATION {
                    self.inner.blaze_it = false;
                    self.inner.blaze_start = None;
                    self.inner.blaze_sound_played = false;
                    None
                } else {
                    ctx.request_repaint();
                    Some(elapsed)
                }
            } else {
                None
            }
        } else {
            None
        };

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
                // Square root button with proper math notation: small raised ² + √x
                let sqrt_clicked = {
                    let mut job = egui::text::LayoutJob::default();
                    job.append("2", 0.0, egui::TextFormat {
                        font_id: egui::FontId::proportional(12.0),
                        color: text_white,
                        valign: egui::Align::Min,
                        ..Default::default()
                    });
                    job.append("\u{221A}x", 0.0, egui::TextFormat {
                        font_id: egui::FontId::proportional(font_size),
                        color: text_white,
                        ..Default::default()
                    });
                    ui.add(
                        egui::Button::new(job)
                            .fill(op_bg)
                            .rounding(4.0)
                            .min_size(btn),
                    ).clicked()
                };
                if sqrt_clicked {
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

            // Draw 420 overlay on top of everything
            if let Some(t) = blaze_t {
                draw_blaze_overlay(ui, t);
            }
        });
    }
}
