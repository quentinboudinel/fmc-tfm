use std::fmt;
use std::path::Path;

use crate::core::TfmImage;
use eframe::egui::{self, Color32, Pos2, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colormap {
    Grayscale,
    Thermal,
}

impl Colormap {
    /// Maps a normalized intensity `t` in `[0, 1]` to a display color.
    pub fn apply(self, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Colormap::Grayscale => {
                let v = (t * 255.0).round() as u8;
                Color32::from_gray(v)
            }
            // Classic black -> red -> yellow -> white "hot" ramp used for
            // UT/NDT amplitude C-scans.
            Colormap::Thermal => {
                let r = (t * 3.0).clamp(0.0, 1.0);
                let g = (t * 3.0 - 1.0).clamp(0.0, 1.0);
                let b = (t * 3.0 - 2.0).clamp(0.0, 1.0);
                Color32::from_rgb(
                    (r * 255.0).round() as u8,
                    (g * 255.0).round() as u8,
                    (b * 255.0).round() as u8,
                )
            }
        }
    }
}

/// Converts a raw TFM amplitude into a normalized `[0, 1]` display intensity:
/// a dB scale relative to the image peak, with adjustable gain and dynamic
/// range, per SPECIFICATION.md 5.5.3.
pub fn normalize(amplitude: f32, max_abs: f32, dynamic_range_db: f32, gain_db: f32) -> f32 {
    if max_abs <= 0.0 {
        return 0.0;
    }
    let ratio = (amplitude.abs() / max_abs).max(1e-6);
    let db = 20.0 * ratio.log10() + gain_db;
    let clamped = db.clamp(-dynamic_range_db, 0.0);
    (clamped + dynamic_range_db) / dynamic_range_db
}

pub struct Heatmap {
    pub dynamic_range_db: f32,
    pub gain_db: f32,
    pub colormap: Colormap,
    texture: Option<egui::TextureHandle>,
    cached: Option<(u64, u32, u32, Colormap)>,
}

impl Default for Heatmap {
    fn default() -> Self {
        Self {
            dynamic_range_db: 40.0,
            gain_db: 0.0,
            colormap: Colormap::Thermal,
            texture: None,
            cached: None,
        }
    }
}

impl Heatmap {
    pub fn show_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Colormap:");
            egui::ComboBox::from_id_salt("colormap")
                .selected_text(format!("{:?}", self.colormap))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.colormap, Colormap::Thermal, "Thermal");
                    ui.selectable_value(&mut self.colormap, Colormap::Grayscale, "Grayscale");
                });
        });
        ui.add(
            egui::Slider::new(&mut self.dynamic_range_db, 6.0..=100.0).text("Dynamic range (dB)"),
        );
        ui.add(egui::Slider::new(&mut self.gain_db, -20.0..=40.0).text("Gain (dB)"));
    }

    /// Draws the reconstruction as a heatmap sized to fit `ui`'s available
    /// space while preserving the grid's aspect ratio, per SPECIFICATION.md
    /// 5.5.3. `generation` should change whenever `image` is replaced with a
    /// new reconstruction, so the display texture is only rebuilt when the
    /// data or display parameters actually change.
    ///
    /// `zoom`/`pan` are shared with `Canvas` (SPECIFICATION.md 5.6: views
    /// should be synchronized for pan/zoom) — passing the same state in from
    /// both panels keeps them in lockstep regardless of which one is dragged
    /// or scrolled.
    pub fn show_image(
        &mut self,
        ui: &mut egui::Ui,
        image: Option<&TfmImage>,
        generation: u64,
        zoom: &mut f32,
        pan: &mut egui::Vec2,
    ) -> egui::Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;

        let bg_color = ui.visuals().extreme_bg_color;
        painter.rect_filled(rect, 0.0, bg_color);

        let Some(image) = image else {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No reconstruction yet",
                egui::FontId::default(),
                ui.visuals().weak_text_color(),
            );
            return response;
        };

        let width_mm = image.grid.width_mm as f32;
        let depth_mm = image.grid.depth_mm as f32;
        handle_pan_zoom(&response, rect, width_mm, depth_mm, zoom, pan);

        self.ensure_texture(ui.ctx(), image, generation);
        if let Some(texture) = &self.texture {
            let top_left = world_to_screen(Pos2::ZERO, rect, width_mm, depth_mm, *zoom, *pan);
            let bottom_right = world_to_screen(
                Pos2::new(width_mm, depth_mm),
                rect,
                width_mm,
                depth_mm,
                *zoom,
                *pan,
            );
            let image_rect = Rect::from_two_pos(top_left, bottom_right);
            painter.image(
                texture.id(),
                image_rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        response
    }

    fn ensure_texture(&mut self, ctx: &egui::Context, image: &TfmImage, generation: u64) {
        let params_key = (
            generation,
            self.dynamic_range_db.to_bits(),
            self.gain_db.to_bits(),
            self.colormap,
        );
        if self.texture.is_some() && self.cached == Some(params_key) {
            return;
        }

        let (width, height, pixels) =
            render_pixels(image, self.dynamic_range_db, self.gain_db, self.colormap);
        let color_image = egui::ColorImage {
            size: [width, height],
            pixels,
        };
        self.texture =
            Some(ctx.load_texture("tfm_heatmap", color_image, egui::TextureOptions::LINEAR));
        self.cached = Some(params_key);
    }
}

/// Renders every pixel of `image` through the normalize/colormap pipeline,
/// shared between the live texture (`Heatmap::ensure_texture`) and PNG export.
fn render_pixels(
    image: &TfmImage,
    dynamic_range_db: f32,
    gain_db: f32,
    colormap: Colormap,
) -> (usize, usize, Vec<Color32>) {
    let max_abs = image.max_abs();
    let width = image.grid.res_x;
    let height = image.grid.res_z;
    let mut pixels = Vec::with_capacity(width * height);
    for iz in 0..height {
        for ix in 0..width {
            let amp = image.get(ix, iz);
            let t = normalize(amp, max_abs, dynamic_range_db, gain_db);
            pixels.push(colormap.apply(t));
        }
    }
    (width, height, pixels)
}

#[derive(Debug)]
pub enum PngExportError {
    InvalidDimensions,
    Encode(image::ImageError),
}

impl fmt::Display for PngExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PngExportError::InvalidDimensions => write!(f, "invalid image dimensions"),
            PngExportError::Encode(e) => write!(f, "PNG encode error: {e}"),
        }
    }
}

impl std::error::Error for PngExportError {}

impl From<image::ImageError> for PngExportError {
    fn from(e: image::ImageError) -> Self {
        PngExportError::Encode(e)
    }
}

/// Renders `tfm_image` through the same normalize/colormap pipeline as the
/// live heatmap and saves it as a PNG, per SPECIFICATION.md 5.7.3.
pub fn export_png<P: AsRef<Path>>(
    path: P,
    tfm_image: &TfmImage,
    dynamic_range_db: f32,
    gain_db: f32,
    colormap: Colormap,
) -> Result<(), PngExportError> {
    let (width, height, pixels) = render_pixels(tfm_image, dynamic_range_db, gain_db, colormap);
    let mut buf = Vec::with_capacity(width * height * 3);
    for color in pixels {
        buf.push(color.r());
        buf.push(color.g());
        buf.push(color.b());
    }
    let img = image::RgbImage::from_raw(width as u32, height as u32, buf)
        .ok_or(PngExportError::InvalidDimensions)?;
    img.save(path)?;
    Ok(())
}

const ZOOM_MIN: f32 = 0.1;
const ZOOM_MAX: f32 = 10.0;
const ZOOM_SPEED: f32 = 0.001;

/// Same shape as `Canvas::pixels_per_mm`/`world_to_screen`/`screen_to_world`,
/// parameterized instead of tied to a `Canvas` instance, so the heatmap can
/// share the exact same zoom/pan behavior without depending on `Canvas`.
fn pixels_per_mm(rect: Rect, width_mm: f32, depth_mm: f32) -> f32 {
    (rect.width() / width_mm).min(rect.height() / depth_mm)
}

fn world_to_screen(
    world: Pos2,
    rect: Rect,
    width_mm: f32,
    depth_mm: f32,
    zoom: f32,
    pan: egui::Vec2,
) -> Pos2 {
    let scale = pixels_per_mm(rect, width_mm, depth_mm);
    Pos2::new(
        rect.left() + world.x * scale * zoom + pan.x,
        rect.top() + world.y * scale * zoom + pan.y,
    )
}

fn screen_to_world(
    screen: Pos2,
    rect: Rect,
    width_mm: f32,
    depth_mm: f32,
    zoom: f32,
    pan: egui::Vec2,
) -> Pos2 {
    let scale = pixels_per_mm(rect, width_mm, depth_mm);
    Pos2::new(
        (screen.x - rect.left() - pan.x) / (scale * zoom),
        (screen.y - rect.top() - pan.y) / (scale * zoom),
    )
}

fn handle_pan_zoom(
    response: &egui::Response,
    rect: Rect,
    width_mm: f32,
    depth_mm: f32,
    zoom: &mut f32,
    pan: &mut egui::Vec2,
) {
    if response.dragged_by(egui::PointerButton::Middle) {
        *pan += response.drag_delta();
    }

    if response.hovered() {
        let scroll = response.ctx.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            if let Some(pointer) = response.ctx.input(|i| i.pointer.hover_pos()) {
                let world_before = screen_to_world(pointer, rect, width_mm, depth_mm, *zoom, *pan);
                let zoom_delta = scroll * ZOOM_SPEED * *zoom;
                *zoom = (*zoom + zoom_delta).clamp(ZOOM_MIN, ZOOM_MAX);
                let world_after = screen_to_world(pointer, rect, width_mm, depth_mm, *zoom, *pan);
                let scale = pixels_per_mm(rect, width_mm, depth_mm) * *zoom;
                pan.x += (world_after.x - world_before.x) * scale;
                pan.y += (world_after.y - world_before.y) * scale;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_peak_amplitude_is_near_full_scale() {
        let t = normalize(10.0, 10.0, 40.0, 0.0);
        assert!((t - 1.0).abs() < 1e-4);
    }

    #[test]
    fn normalize_zero_amplitude_is_floor() {
        let t = normalize(0.0, 10.0, 40.0, 0.0);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn normalize_zero_max_abs_is_floor() {
        let t = normalize(5.0, 0.0, 40.0, 0.0);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn normalize_is_symmetric_in_sign() {
        let pos = normalize(3.0, 10.0, 40.0, 0.0);
        let neg = normalize(-3.0, 10.0, 40.0, 0.0);
        assert_eq!(pos, neg);
    }

    #[test]
    fn normalize_gain_shifts_toward_full_scale() {
        let base = normalize(1.0, 10.0, 40.0, 0.0);
        let boosted = normalize(1.0, 10.0, 40.0, 20.0);
        assert!(boosted > base);
    }

    #[test]
    fn normalize_output_is_always_in_unit_range() {
        for amp in [0.0, 0.001, 1.0, 10.0, 1000.0] {
            let t = normalize(amp, 10.0, 40.0, 0.0);
            assert!((0.0..=1.0).contains(&t), "t={t} out of range for amp={amp}");
        }
    }

    #[test]
    fn grayscale_colormap_endpoints() {
        assert_eq!(Colormap::Grayscale.apply(0.0), Color32::from_gray(0));
        assert_eq!(Colormap::Grayscale.apply(1.0), Color32::from_gray(255));
    }

    #[test]
    fn thermal_colormap_endpoints() {
        assert_eq!(Colormap::Thermal.apply(0.0), Color32::from_rgb(0, 0, 0));
        assert_eq!(
            Colormap::Thermal.apply(1.0),
            Color32::from_rgb(255, 255, 255)
        );
    }

    #[test]
    fn heatmap_defaults_are_reasonable() {
        let heatmap = Heatmap::default();
        assert_eq!(heatmap.colormap, Colormap::Thermal);
        assert!(heatmap.dynamic_range_db > 0.0);
    }

    #[test]
    fn world_origin_maps_to_screen_topleft() {
        let rect = Rect::from_min_size(Pos2::new(100.0, 50.0), egui::Vec2::new(400.0, 200.0));
        let screen = world_to_screen(Pos2::ZERO, rect, 100.0, 50.0, 1.0, egui::Vec2::ZERO);
        assert_eq!(screen.x, rect.left());
        assert_eq!(screen.y, rect.top());
    }

    #[test]
    fn screen_to_world_roundtrip() {
        let rect = Rect::from_min_size(Pos2::new(100.0, 50.0), egui::Vec2::new(400.0, 200.0));
        let world = Pos2::new(25.0, 12.5);
        let screen = world_to_screen(world, rect, 100.0, 50.0, 1.5, egui::Vec2::new(10.0, -5.0));
        let back = screen_to_world(screen, rect, 100.0, 50.0, 1.5, egui::Vec2::new(10.0, -5.0));
        assert!((back.x - world.x).abs() < 0.001);
        assert!((back.y - world.y).abs() < 0.001);
    }

    #[test]
    fn zoom_scales_distance_from_origin() {
        let rect = Rect::from_min_size(Pos2::new(100.0, 50.0), egui::Vec2::new(400.0, 200.0));
        let world = Pos2::new(10.0, 10.0);
        let at_1x = world_to_screen(world, rect, 100.0, 50.0, 1.0, egui::Vec2::ZERO);
        let at_2x = world_to_screen(world, rect, 100.0, 50.0, 2.0, egui::Vec2::ZERO);
        let d1 = at_1x.x - rect.left();
        let d2 = at_2x.x - rect.left();
        assert!((d2 - d1 * 2.0).abs() < 0.001);
    }

    fn sample_tfm_image() -> TfmImage {
        use crate::core::TfmGrid;
        let grid = TfmGrid::new(10.0, 10.0, 4, 3);
        let values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0];
        TfmImage::new(grid, values)
    }

    fn temp_png_path(name: &str) -> std::path::PathBuf {
        tempfile::Builder::new()
            .prefix(name)
            .suffix(".png")
            .tempfile()
            .unwrap()
            .into_temp_path()
            .to_path_buf()
    }

    #[test]
    fn export_png_writes_readable_file_with_matching_dimensions() {
        let path = temp_png_path("heatmap_export");
        let tfm_image = sample_tfm_image();

        export_png(&path, &tfm_image, 40.0, 0.0, Colormap::Thermal).unwrap();

        let decoded = image::open(&path).unwrap();
        assert_eq!(decoded.width(), 4);
        assert_eq!(decoded.height(), 3);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn export_png_brightest_pixel_matches_colormap_peak() {
        let path = temp_png_path("heatmap_export_peak");
        let tfm_image = sample_tfm_image();

        export_png(&path, &tfm_image, 40.0, 0.0, Colormap::Grayscale).unwrap();

        let decoded = image::open(&path).unwrap().to_rgb8();
        // The last value (11.0) is the image's max_abs, so it should map to
        // full-scale white under the grayscale colormap.
        let brightest = decoded.get_pixel(3, 2);
        assert_eq!(*brightest, image::Rgb([255, 255, 255]));

        std::fs::remove_file(&path).ok();
    }
}
