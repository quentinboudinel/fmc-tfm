use eframe::egui::{self, Pos2, Rect, Vec2};

#[derive(Clone, Debug)]
pub struct Canvas {
    pub width_mm: f32,
    pub depth_mm: f32,
    pub zoom: f32,
    pub pan: Vec2,
}

impl Default for Canvas {
    fn default() -> Self {
        Self {
            width_mm: 100.0,
            depth_mm: 50.0,
            zoom: 1.0,
            pan: Vec2::ZERO,
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
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::drag());
        let rect = response.rect;

        let bg_color = ui.visuals().extreme_bg_color;
        painter.rect_filled(rect, 0.0, bg_color);

        let border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, border_color));

        response
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
}
