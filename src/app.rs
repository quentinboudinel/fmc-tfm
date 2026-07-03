use crate::core::Project;
use crate::ui::Canvas;
use eframe::egui;

pub struct App {
    pub canvas: Canvas,
    pub project: Project,
}

impl Default for App {
    fn default() -> Self {
        let project = Project::default();
        let canvas = Canvas::new(
            project.material.width_mm as f32,
            project.material.depth_mm as f32,
        );
        Self { canvas, project }
    }
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
                    ui.label(format!("{} defects", self.project.defects.len()));
                });

                ui.collapsing("Simulation", |ui| {
                    ui.label("Simulation controls placeholder");
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.canvas.show(ui, &self.project.defects);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_has_canvas_and_project() {
        let app = App::default();
        assert_eq!(app.canvas.width_mm, app.project.material.width_mm as f32);
        assert_eq!(app.canvas.depth_mm, app.project.material.depth_mm as f32);
        assert!(app.project.defects.is_empty());
    }
}
