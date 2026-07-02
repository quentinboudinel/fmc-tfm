use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

const GRID_SPACING_MM: f32 = 10.0;
const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 10.0;
const ZOOM_SPEED: f32 = 0.001;
const PROBE_HEIGHT_MM: f32 = 3.0;

#[derive(Clone, Debug)]
pub struct Canvas {
    pub width_mm: f32,
    pub depth_mm: f32,
    pub zoom: f32,
    pub pan: Vec2,
    pub num_elements: usize,
    pub pitch_mm: f32,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width_mm: 100.0,
            depth_mm: 50.0,
            zoom: 1.0,
            pan: Vec2::ZERO,
            num_elements: 64,
            pitch_mm: 0.5,
        }
    }
}

impl Canvas {
    pub fn new(width_mm: f32, depth_mm: f32) -> Self {
        Self {
            width_mm,
            depth_mm,
            ..Default::default()
        }
    }

    pub fn world_to_screen(&self, world: Pos2, screen_rect: Rect) -> Pos2 {
        let scale = self.pixels_per_mm(screen_rect);
        let x = screen_rect.left() + (world.x * scale * self.zoom) + self.pan.x;
        let y = screen_rect.top() + (world.y * scale * self.zoom) + self.pan.y;
        Pos2::new(x, y)
    }

    pub fn screen_to_world(&self, screen: Pos2, screen_rect: Rect) -> Pos2 {
        let scale = self.pixels_per_mm(screen_rect);
        let x = (screen.x - screen_rect.left() - self.pan.x) / (scale * self.zoom);
        let y = (screen.y - screen_rect.top() - self.pan.y) / (scale * self.zoom);
        Pos2::new(x, y)
    }

    fn pixels_per_mm(&self, screen_rect: Rect) -> f32 {
        let scale_x = screen_rect.width() / self.width_mm;
        let scale_y = screen_rect.height() / self.depth_mm;
        scale_x.min(scale_y)
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;

        self.handle_input(&response, rect);

        let bg_color = ui.visuals().extreme_bg_color;
        painter.rect_filled(rect, 0.0, bg_color);

        self.draw_grid(&painter, rect);
        self.draw_material_boundary(&painter, rect);
        self.draw_probe(&painter, rect);

        response
    }

    fn handle_input(&mut self, response: &egui::Response, rect: Rect) {
        if response.dragged_by(egui::PointerButton::Middle) {
            self.pan += response.drag_delta();
        }

        if response.hovered() {
            let scroll = response.ctx.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                if let Some(pointer) = response.ctx.input(|i| i.pointer.hover_pos()) {
                    let world_before = self.screen_to_world(pointer, rect);
                    let zoom_delta = scroll * ZOOM_SPEED * self.zoom;
                    self.zoom = (self.zoom + zoom_delta).clamp(ZOOM_MIN, ZOOM_MAX);
                    let world_after = self.screen_to_world(pointer, rect);
                    let scale = self.pixels_per_mm(rect) * self.zoom;
                    self.pan.x += (world_after.x - world_before.x) * scale;
                    self.pan.y += (world_after.y - world_before.y) * scale;
                }
            }
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, rect: Rect) {
        let grid_color = Color32::from_gray(60);
        let stroke = Stroke::new(0.5, grid_color);

        let mut x = 0.0;
        while x <= self.width_mm {
            let p1 = self.world_to_screen(Pos2::new(x, 0.0), rect);
            let p2 = self.world_to_screen(Pos2::new(x, self.depth_mm), rect);
            if p1.x >= rect.left() && p1.x <= rect.right() {
                painter.line_segment([p1, p2], stroke);
            }
            x += GRID_SPACING_MM;
        }

        let mut y = 0.0;
        while y <= self.depth_mm {
            let p1 = self.world_to_screen(Pos2::new(0.0, y), rect);
            let p2 = self.world_to_screen(Pos2::new(self.width_mm, y), rect);
            if p1.y >= rect.top() && p1.y <= rect.bottom() {
                painter.line_segment([p1, p2], stroke);
            }
            y += GRID_SPACING_MM;
        }
    }

    fn draw_material_boundary(&self, painter: &egui::Painter, rect: Rect) {
        let top_left = self.world_to_screen(Pos2::new(0.0, 0.0), rect);
        let bottom_right = self.world_to_screen(Pos2::new(self.width_mm, self.depth_mm), rect);
        let material_rect = Rect::from_two_pos(top_left, bottom_right);

        let fill = Color32::from_rgba_unmultiplied(100, 100, 150, 40);
        painter.rect_filled(material_rect, 0.0, fill);

        let border = Stroke::new(2.0, Color32::from_rgb(150, 150, 200));
        painter.rect_stroke(material_rect, 0.0, border);
    }

    fn draw_probe(&self, painter: &egui::Painter, rect: Rect) {
        let probe_width = self.num_elements as f32 * self.pitch_mm;
        let probe_start = (self.width_mm - probe_width) / 2.0;

        let top_left = self.world_to_screen(Pos2::new(probe_start, -PROBE_HEIGHT_MM), rect);
        let bottom_right = self.world_to_screen(Pos2::new(probe_start + probe_width, 0.0), rect);
        let probe_rect = Rect::from_two_pos(top_left, bottom_right);

        let fill = Color32::from_rgb(80, 120, 80);
        painter.rect_filled(probe_rect, 2.0, fill);

        let element_color = Color32::from_rgb(200, 200, 100);
        for i in 0..self.num_elements {
            let x = probe_start + (i as f32 + 0.5) * self.pitch_mm;
            let p1 = self.world_to_screen(Pos2::new(x, -PROBE_HEIGHT_MM * 0.8), rect);
            let p2 = self.world_to_screen(Pos2::new(x, -PROBE_HEIGHT_MM * 0.2), rect);
            painter.line_segment([p1, p2], Stroke::new(1.0, element_color));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rect() -> Rect {
        Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(400.0, 200.0))
    }

    #[test]
    fn world_origin_maps_to_screen_topleft() {
        let canvas = Canvas::new(100.0, 50.0);
        let rect = test_rect();
        let screen = canvas.world_to_screen(Pos2::new(0.0, 0.0), rect);
        assert_eq!(screen.x, rect.left());
        assert_eq!(screen.y, rect.top());
    }

    #[test]
    fn screen_to_world_roundtrip() {
        let canvas = Canvas::new(100.0, 50.0);
        let rect = test_rect();
        let world = Pos2::new(25.0, 12.5);
        let screen = canvas.world_to_screen(world, rect);
        let back = canvas.screen_to_world(screen, rect);
        assert!((back.x - world.x).abs() < 0.001);
        assert!((back.y - world.y).abs() < 0.001);
    }

    #[test]
    fn zoom_scales_correctly() {
        let mut canvas = Canvas::new(100.0, 50.0);
        let rect = test_rect();
        let world = Pos2::new(10.0, 10.0);

        let screen1 = canvas.world_to_screen(world, rect);
        canvas.zoom = 2.0;
        let screen2 = canvas.world_to_screen(world, rect);

        let dx1 = screen1.x - rect.left();
        let dx2 = screen2.x - rect.left();
        assert!((dx2 - dx1 * 2.0).abs() < 0.001);
    }

    #[test]
    fn pan_offsets_correctly() {
        let mut canvas = Canvas::new(100.0, 50.0);
        let rect = test_rect();
        let world = Pos2::new(10.0, 10.0);

        let screen1 = canvas.world_to_screen(world, rect);
        canvas.pan = Vec2::new(50.0, 30.0);
        let screen2 = canvas.world_to_screen(world, rect);

        assert!((screen2.x - screen1.x - 50.0).abs() < 0.001);
        assert!((screen2.y - screen1.y - 30.0).abs() < 0.001);
    }

    #[test]
    fn pixels_per_mm_respects_aspect_ratio() {
        let canvas = Canvas::new(100.0, 50.0);
        let wide_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 200.0));
        let tall_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 400.0));

        let scale_wide = canvas.pixels_per_mm(wide_rect);
        let scale_tall = canvas.pixels_per_mm(tall_rect);

        assert_eq!(scale_wide, 4.0);
        assert_eq!(scale_tall, 2.0);
    }

    #[test]
    fn zoom_clamps_to_valid_range() {
        let mut canvas = Canvas::default();

        canvas.zoom = 0.01;
        canvas.zoom = canvas.zoom.clamp(super::ZOOM_MIN, super::ZOOM_MAX);
        assert_eq!(canvas.zoom, super::ZOOM_MIN);

        canvas.zoom = 100.0;
        canvas.zoom = canvas.zoom.clamp(super::ZOOM_MIN, super::ZOOM_MAX);
        assert_eq!(canvas.zoom, super::ZOOM_MAX);
    }

    #[test]
    fn default_has_probe_config() {
        let canvas = Canvas::default();
        assert_eq!(canvas.num_elements, 64);
        assert_eq!(canvas.pitch_mm, 0.5);
    }
}
