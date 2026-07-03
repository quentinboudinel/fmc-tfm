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
    pub fn show_image(
        &mut self,
        ui: &mut egui::Ui,
        image: Option<&TfmImage>,
        generation: u64,
    ) -> egui::Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
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

        self.ensure_texture(ui.ctx(), image, generation);
        if let Some(texture) = &self.texture {
            let aspect = image.grid.width_mm / image.grid.depth_mm;
            let image_rect = fit_aspect(rect, aspect as f32);
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

        let max_abs = image.max_abs();
        let width = image.grid.res_x;
        let height = image.grid.res_z;
        let mut pixels = Vec::with_capacity(width * height);
        for iz in 0..height {
            for ix in 0..width {
                let amp = image.get(ix, iz);
                let t = normalize(amp, max_abs, self.dynamic_range_db, self.gain_db);
                pixels.push(self.colormap.apply(t));
            }
        }
        let color_image = egui::ColorImage {
            size: [width, height],
            pixels,
        };
        self.texture =
            Some(ctx.load_texture("tfm_heatmap", color_image, egui::TextureOptions::LINEAR));
        self.cached = Some(params_key);
    }
}

fn fit_aspect(rect: Rect, aspect: f32) -> Rect {
    let available_aspect = rect.width() / rect.height();
    let (w, h) = if available_aspect > aspect {
        (rect.height() * aspect, rect.height())
    } else {
        (rect.width(), rect.width() / aspect)
    };
    Rect::from_center_size(rect.center(), egui::Vec2::new(w, h))
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
    fn fit_aspect_preserves_wide_grid_within_tall_rect() {
        let rect = Rect::from_min_size(Pos2::ZERO, egui::Vec2::new(200.0, 200.0));
        let fitted = fit_aspect(rect, 2.0);
        assert!((fitted.width() / fitted.height() - 2.0).abs() < 1e-4);
        assert!(fitted.width() <= rect.width() + 1e-4);
        assert!(fitted.height() <= rect.height() + 1e-4);
    }
}
