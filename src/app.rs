use crate::ui::Canvas;
use eframe::egui;

#[derive(Default)]
pub struct App {
    pub canvas: Canvas,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("control_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("FMC-TFM");
                ui.separator();

                ui.collapsing("Material", |ui| {
                    ui.label("Material settings placeholder");
                });

                ui.collapsing("Probe", |ui| {
                    ui.label("Probe settings placeholder");
                });

                ui.collapsing("Defects", |ui| {
                    ui.label("Defect list placeholder");
                });

                ui.collapsing("Simulation", |ui| {
                    ui.label("Simulation controls placeholder");
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.canvas.show(ui);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_has_canvas() {
        let app = App::default();
        assert_eq!(app.canvas.width_mm, 100.0);
        assert_eq!(app.canvas.depth_mm, 50.0);
    }
}
