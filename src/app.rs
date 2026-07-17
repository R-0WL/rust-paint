use crate::bmp::WriteBmp;
use crate::framebuffer::Framebuffer;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::f32::consts::TAU;

const CANVAS_W: usize = 450;
const CANVAS_H: usize = 450;
const BRUSH_SIZES: [i32; 5] = [1, 3, 6, 12, 20];

#[derive(PartialEq, Clone, Copy)]
enum Tool { Brush, Eraser }

pub struct PaintApp {
    fb: Framebuffer,
    tool: Tool,
    brush_size: usize,
    color: Color32,
    recent_colors: Vec<Color32>,
    texture: Option<egui::TextureHandle>,
    dirty: bool,
    last_point: Option<(i32, i32)>,
    status: String,
    picker_open: bool,
    picker_hsv: [f32; 3],
    hex_color: String,
}

impl PaintApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        Self::default()
    }

    fn remember_color(&mut self, color: Color32) {
        self.color = color;
        self.recent_colors.retain(|&c| c != color);
        self.recent_colors.insert(0, color);
        self.recent_colors.truncate(10);
    }

    fn export(&mut self) {
        let directory = match crate::export::reserve_export_directory(std::path::Path::new(".")) {
            Ok(directory) => directory,
            Err(err) => {
                self.status = format!("No se pudo crear la carpeta de exportación: {err}");
                return;
            }
        };
        let bmp_path = directory.join("out.bmp");
        let png_path = directory.join("out.png");
        // Try both formats even if one of the writes fails.
        let bmp = self.fb.render_buffer(&bmp_path.to_string_lossy());
        let png = self.fb.render_buffer(&png_path.to_string_lossy());
        self.status = match (bmp, png) {
            (Ok(()), Ok(())) => format!("Guardado en {}: out.bmp y out.png", directory.display()),
            (Err(b), Err(p)) => format!("Carpeta {} · Error BMP: {b}; PNG: {p}", directory.display()),
            (Err(e), Ok(())) => format!("{} guardado; error BMP: {e}", png_path.display()),
            (Ok(()), Err(e)) => format!("{} guardado; error PNG: {e}", bmp_path.display()),
        };
    }

    fn paint_segment(&mut self, from: (i32, i32), to: (i32, i32)) {
        let color = if self.tool == Tool::Brush { self.color } else { Color32::WHITE };
        self.fb.set_current_color(
            (u32::from(color.r()) << 16) | (u32::from(color.g()) << 8) | u32::from(color.b())
        );
        self.fb.thick_line(from.0, from.1, to.0, to.1, BRUSH_SIZES[self.brush_size]);
        self.dirty = true;
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.heading("Paint Rust");
            ui.separator();
            ui.selectable_value(&mut self.tool, Tool::Brush, "Pincel");
            ui.selectable_value(&mut self.tool, Tool::Eraser, "Borrador");
            ui.separator();
            ui.label("Grosor:");
            for (i, size) in BRUSH_SIZES.iter().enumerate() {
                ui.selectable_value(&mut self.brush_size, i, format!("{size} px"));
            }
            ui.separator();
            if ui.button("Guardar · S").clicked() { self.export(); }
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Colores:");
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::click());
            paint_wheel(ui.painter(), rect.center(), 15.0, 1.0);
            if response.on_hover_text("Abrir círculo cromático completo").clicked() {
                let hsv = egui::ecolor::Hsva::from(self.color);
                self.picker_hsv = [hsv.h, hsv.s, hsv.v];
                self.hex_color = color_hex(self.color);
                self.picker_open = true;
                self.last_point = None;
            }
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
            ui.painter().rect_filled(rect, 4.0, self.color);
            ui.label(color_hex(self.color));
            ui.separator();
            ui.label("Últimos 10:");
            for i in 0..10 {
                let color = self.recent_colors.get(i).copied();
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
                ui.painter().rect_filled(rect.shrink(1.0), 4.0, color.unwrap_or(Color32::from_gray(225)));
                let selected = color == Some(self.color);
                ui.painter().rect_stroke(rect.shrink(1.0), 4.0,
                    Stroke::new(if selected { 2.0 } else { 0.5 }, Color32::DARK_GRAY),
                    egui::StrokeKind::Inside);
                if let Some(c) = color {
                    if response.on_hover_text(color_hex(c)).clicked() { self.remember_color(c); }
                }
            }
        });
    }

    fn picker(&mut self, ctx: &egui::Context) {
        if !self.picker_open { return; }
        let mut open = true;
        let mut accept = false;
        let mut cancel = false;
        egui::Window::new("Círculo cromático")
            .open(&mut open).collapsible(false).resizable(false)
            .show(ctx, |ui| {
                ui.label("Elegí el tono y la saturación; ajustá el brillo abajo.");
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(260.0), Sense::click_and_drag());
                let radius = 122.0;
                paint_wheel(ui.painter(), rect.center(), radius, 1.0);
                if response.clicked() || response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let d = pos - rect.center();
                        // Initial clicks outside the disk are ignored; drags clamp at its edge.
                        if d.length() <= radius || response.dragged() {
                            self.picker_hsv[0] = (d.y.atan2(d.x) / TAU).rem_euclid(1.0);
                            self.picker_hsv[1] = (d.length() / radius).min(1.0);
                            self.hex_color = color_hex(hsv_color(self.picker_hsv));
                        }
                    }
                }
                let angle = self.picker_hsv[0] * TAU;
                let marker = rect.center() + Vec2::new(angle.cos(), angle.sin()) * self.picker_hsv[1] * radius;
                ui.painter().circle_stroke(marker, 6.0, Stroke::new(3.0, Color32::BLACK));
                ui.painter().circle_stroke(marker, 6.0, Stroke::new(1.5, Color32::WHITE));
                if ui.add(egui::Slider::new(&mut self.picker_hsv[2], 0.0..=1.0).text("Brillo")).changed() {
                    self.hex_color = color_hex(hsv_color(self.picker_hsv));
                }
                ui.horizontal(|ui| {
                    ui.label("HEX:");
                    let response = ui.add(egui::TextEdit::singleline(&mut self.hex_color).desired_width(90.0));
                    if response.changed() {
                        if let Some(color) = parse_hex(&self.hex_color) {
                            let hsv = egui::ecolor::Hsva::from(color);
                            self.picker_hsv = [hsv.h, hsv.s, hsv.v];
                        }
                    }
                    let (preview, _) = ui.allocate_exact_size(Vec2::new(52.0, 26.0), Sense::hover());
                    ui.painter().rect_filled(preview, 4.0, hsv_color(self.picker_hsv));
                });
                let valid = parse_hex(&self.hex_color).is_some();
                if !valid { ui.colored_label(Color32::RED, "Ingresá seis dígitos: #RRGGBB"); }
                ui.horizontal(|ui| {
                    accept = ui.add_enabled(valid, egui::Button::new("Usar color")).clicked();
                    cancel = ui.button("Cancelar").clicked();
                });
            });
        if accept { self.remember_color(hsv_color(self.picker_hsv)); }
        if accept || cancel || !open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.picker_open = false;
        }
    }

    fn canvas_ui(&mut self, ui: &mut egui::Ui, blocked: bool) -> Rect {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(CANVAS_W as f32, CANVAS_H as f32), Sense::click_and_drag());
        let (pressed, down) = ui.input(|i| (i.pointer.primary_pressed(), i.pointer.primary_down()));
        if blocked || !down { self.last_point = None; }
        if !blocked && response.contains_pointer() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let point = ((pos.x - rect.left()).floor() as i32, (pos.y - rect.top()).floor() as i32);
                if pressed {
                    self.paint_segment(point, point);
                    self.last_point = Some(point);
                } else if down {
                    if let Some(previous) = self.last_point {
                        self.paint_segment(previous, point);
                        self.last_point = Some(point);
                    }
                }
            }
        } else {
            // Leaving the canvas ends a segment, so returning never draws a connecting line.
            self.last_point = None;
        }
        if self.dirty || self.texture.is_none() {
            let image = egui::ColorImage::from_rgb([CANVAS_W, CANVAS_H], &self.fb.to_rgb_bytes());
            if let Some(texture) = &mut self.texture {
                texture.set(image, egui::TextureOptions::NEAREST);
            } else {
                self.texture = Some(ui.ctx().load_texture("canvas", image, egui::TextureOptions::NEAREST));
            }
            self.dirty = false;
        }
        if let Some(texture) = &self.texture {
            ui.painter().image(texture.id(), rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        }
        ui.painter().rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::GRAY), egui::StrokeKind::Outside);
        if !blocked && response.contains_pointer() {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                let p = ui.painter().with_clip_rect(rect);
                let radius = (BRUSH_SIZES[self.brush_size] as f32 / 2.0).max(1.0);
                p.circle_stroke(pos, radius, Stroke::new(2.0, Color32::WHITE));
                p.circle_stroke(pos, radius, Stroke::new(1.0, Color32::BLACK));
            }
        }
        rect
    }

    fn cursor(&self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() { return; }
        if let Some(pos) = ctx.pointer_hover_pos().filter(|p| ctx.screen_rect().contains(*p)) {
            ctx.set_cursor_icon(egui::CursorIcon::None);
            let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("tool-cursor")));
            draw_tool_icon(&painter, pos + Vec2::new(12.0, 14.0), self.tool, self.color);
            // The icon is offset from the actual drawing point.
            painter.line_segment([pos - Vec2::new(3.0, 0.0), pos + Vec2::new(3.0, 0.0)], Stroke::new(1.0, Color32::GRAY));
            painter.line_segment([pos - Vec2::new(0.0, 3.0), pos + Vec2::new(0.0, 3.0)], Stroke::new(1.0, Color32::GRAY));
        }
    }
}

impl Default for PaintApp {
    fn default() -> Self {
        let mut fb = Framebuffer::new(CANVAS_W, CANVAS_H);
        fb.set_background_color(0xFFFFFF);
        fb.clear();
        Self {
            fb, tool: Tool::Brush, brush_size: 2, color: Color32::BLACK,
            recent_colors: vec![Color32::BLACK], texture: None, dirty: true,
            last_point: None, status: "Listo · S guarda BMP y PNG".into(),
            picker_open: false, picker_hsv: [0.0, 0.0, 0.0], hex_color: "#000000".into(),
        }
    }
}

impl eframe::App for PaintApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_enabled_ui(!self.picker_open, |ui| self.toolbar(ui));
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(&self.status);
                ui.separator();
                ui.label(format!("450 × 450 px · {} · {} px", if self.tool == Tool::Brush { "Pincel" } else { "Borrador" }, BRUSH_SIZES[self.brush_size]));
            });
        });
        let blocked = self.picker_open;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Lienzo");
            ui.add_space(8.0);
            egui::ScrollArea::both().show(ui, |ui| self.canvas_ui(ui, blocked));
        });
        self.picker(ctx);
        // Do not export when S is typed inside the HEX input or the picker is open.
        if !ctx.wants_keyboard_input() && !blocked && !self.picker_open
            && ctx.input(|i| i.key_pressed(egui::Key::S)) {
            self.export();
        }
        self.cursor(ctx);
    }
}

fn color_hex(c: Color32) -> String { format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b()) }

fn parse_hex(text: &str) -> Option<Color32> {
    let text = text.trim();
    let value = text.strip_prefix('#').unwrap_or(text);
    if value.len() != 6 || !value.bytes().all(|c| c.is_ascii_hexdigit()) { return None; }
    let rgb = u32::from_str_radix(value, 16).ok()?;
    Some(Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8))
}

fn hsv_color(hsv: [f32; 3]) -> Color32 {
    egui::ecolor::Hsva::new(hsv[0], hsv[1], hsv[2], 1.0).into()
}

fn paint_wheel(painter: &egui::Painter, center: Pos2, radius: f32, value: f32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, hsv_color([0.0, 0.0, value]));
    for i in 0..=96 {
        let hue = i as f32 / 96.0;
        let angle = hue * TAU;
        mesh.colored_vertex(center + Vec2::new(angle.cos(), angle.sin()) * radius,
            hsv_color([hue, 1.0, value]));
        if i > 0 { mesh.add_triangle(0, i, i + 1); }
    }
    painter.add(egui::Shape::mesh(mesh));
}

fn draw_tool_icon(p: &egui::Painter, origin: Pos2, tool: Tool, color: Color32) {
    let v = |x, y| origin + Vec2::new(x, y);
    let outline = Stroke::new(1.3, Color32::from_gray(35));
    match tool {
        Tool::Brush => {
            p.line_segment([v(5.0, 14.0), v(18.0, -1.0)], Stroke::new(7.0, Color32::WHITE));
            p.line_segment([v(5.0, 14.0), v(18.0, -1.0)], Stroke::new(4.0, Color32::from_rgb(139, 88, 45)));
            p.add(egui::Shape::convex_polygon(vec![v(0.0, 22.0), v(1.0, 13.0), v(6.0, 10.0), v(10.0, 15.0), v(6.0, 20.0)], color, outline));
            p.line_segment([v(4.0, 12.0), v(8.0, 16.0)], Stroke::new(3.0, Color32::LIGHT_GRAY));
        }
        Tool::Eraser => {
            p.add(egui::Shape::convex_polygon(vec![v(0.0, 12.0), v(11.0, 0.0), v(23.0, 10.0), v(12.0, 22.0)], Color32::from_rgb(247, 156, 181), outline));
            p.add(egui::Shape::convex_polygon(vec![v(0.0, 12.0), v(5.0, 7.0), v(17.0, 17.0), v(12.0, 22.0)], Color32::WHITE, outline));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_keeps_ten_unique_colors_in_most_recent_order() {
        let mut app = PaintApp::default();
        for i in 1..=12 { app.remember_color(Color32::from_rgb(i, 0, 0)); }
        app.remember_color(Color32::from_rgb(7, 0, 0));
        assert_eq!(app.recent_colors.len(), 10);
        assert_eq!(app.recent_colors[0], Color32::from_rgb(7, 0, 0));
        assert_eq!(app.recent_colors[1], Color32::from_rgb(12, 0, 0));
        assert_eq!(app.recent_colors.iter().filter(|&&c| c == app.color).count(), 1);
    }

    #[test]
    fn hex_accepts_rgb_and_rejects_invalid_input() {
        assert_eq!(parse_hex("#12abEF"), Some(Color32::from_rgb(0x12, 0xAB, 0xEF)));
        for bad in ["#123", "#GG0000", "#12345678", "ñ12345"] { assert!(parse_hex(bad).is_none()); }
    }

    #[test]
    fn eraser_does_not_change_brush_color_or_history() {
        let mut app = PaintApp::default();
        app.remember_color(Color32::RED);
        app.paint_segment((10, 10), (20, 20));
        assert_eq!(app.fb.buffer[10 * CANVAS_W + 10], 0xFF0000);
        app.tool = Tool::Eraser;
        app.paint_segment((10, 10), (20, 20));
        assert_eq!(app.fb.buffer[10 * CANVAS_W + 10], 0xFFFFFF);
        assert_eq!(app.color, Color32::RED);
        assert_eq!(app.recent_colors[0], Color32::RED);
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;

    fn frame(app: &mut PaintApp, ctx: &egui::Context, events: Vec<egui::Event>, blocked: bool) -> Rect {
        let mut canvas = Rect::NOTHING;
        let _ = ctx.run(egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(600.0))),
            events,
            ..Default::default()
        }, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                canvas = app.canvas_ui(ui, blocked);
            });
        });
        canvas
    }

    fn button(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos, button: egui::PointerButton::Primary, pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }

    #[test]
    fn pointer_click_draws_a_dot_and_separate_strokes_do_not_connect() {
        let mut app = PaintApp::default();
        app.brush_size = 0;
        let ctx = egui::Context::default();
        let rect = frame(&mut app, &ctx, vec![], false);
        let a = rect.min + Vec2::new(20.5, 20.5);
        let b = rect.min + Vec2::new(80.5, 20.5);
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(a), button(a, true)], false);
        frame(&mut app, &ctx, vec![button(a, false)], false);
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(b), button(b, true)], false);
        frame(&mut app, &ctx, vec![button(b, false)], false);
        assert_eq!(app.fb.buffer[20 * CANVAS_W + 20], 0);
        assert_eq!(app.fb.buffer[20 * CANVAS_W + 80], 0);
        assert_eq!(app.fb.buffer[20 * CANVAS_W + 50], 0xFFFFFF);
    }

    #[test]
    fn picker_blocks_canvas_input() {
        let mut app = PaintApp::default();
        let ctx = egui::Context::default();
        let rect = frame(&mut app, &ctx, vec![], true);
        let pos = rect.min + Vec2::splat(30.0);
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(pos), button(pos, true)], true);
        assert!(app.fb.buffer.iter().all(|&p| p == 0xFFFFFF));
    }
}
