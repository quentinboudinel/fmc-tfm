use crate::core::{
    AddDefect, CommandHistory, Defect, FmcData, FmcSimulator, MaterialPreset, MoveDefect,
    PointReflector, Project, RemoveDefect, TfmGrid, TfmImage, TfmReconstructor,
};
use crate::ui::{Canvas, Heatmap};
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolMode {
    #[default]
    Select,
    AddPoint,
    AddCrack,
    AddVoid,
}

pub struct App {
    pub canvas: Canvas,
    pub project: Project,
    pub history: CommandHistory,
    pub tool_mode: ToolMode,
    pub selected_defect: Option<usize>,
    pub heatmap: Heatmap,
    pub tfm_image: Option<TfmImage>,
    /// Bumped every time `tfm_image` is replaced, so `Heatmap` knows when it
    /// must rebuild its cached texture instead of reusing the last one.
    pub tfm_generation: u64,
    /// FMC data from the last `run_simulation()`, kept around so "Export FMC"
    /// doesn't need to re-simulate.
    pub fmc_data: Option<FmcData>,
    /// FMC data loaded via "Import FMC", shown (metadata first) before the
    /// user chooses to reconstruct it, per SPECIFICATION.md 5.7.2.
    pub imported_fmc: Option<FmcData>,
    /// Last save/load/export/import outcome, shown in the control panel.
    pub status_message: Option<String>,
    drag_start_pos: Option<(f64, f64)>,
}

impl Default for App {
    fn default() -> Self {
        let project = Project::default();
        let canvas = Canvas::new(
            project.material.width_mm as f32,
            project.material.depth_mm as f32,
        );
        Self {
            canvas,
            project,
            history: CommandHistory::default(),
            tool_mode: ToolMode::Select,
            selected_defect: None,
            heatmap: Heatmap::default(),
            tfm_image: None,
            tfm_generation: 0,
            fmc_data: None,
            imported_fmc: None,
            status_message: None,
            drag_start_pos: None,
        }
    }
}

impl App {
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let mut save = false;
        let mut open = false;
        ctx.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Z) {
                if i.modifiers.shift {
                    self.history.redo(&mut self.project);
                } else {
                    self.history.undo(&mut self.project);
                }
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::Y) {
                self.history.redo(&mut self.project);
            }
            if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                if let Some(idx) = self.selected_defect {
                    if idx < self.project.defects.len() {
                        let cmd = RemoveDefect::new(idx);
                        self.history.execute(Box::new(cmd), &mut self.project);
                        self.selected_defect = None;
                    }
                }
            }
            if i.key_pressed(egui::Key::Escape) {
                self.tool_mode = ToolMode::Select;
                self.selected_defect = None;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                save = true;
            }
            if i.modifiers.ctrl && i.key_pressed(egui::Key::O) {
                open = true;
            }
        });
        if save {
            self.save_project_dialog();
        }
        if open {
            self.open_project_dialog();
        }
    }

    fn defect_at_position(&self, x: f64, y: f64, tolerance: f64) -> Option<usize> {
        for (i, defect) in self.project.defects.iter().enumerate() {
            let (dx, dy) = defect.position();
            let dist = ((x - dx).powi(2) + (y - dy).powi(2)).sqrt();
            if dist < tolerance {
                return Some(i);
            }
        }
        None
    }

    fn save_project_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Project", &["json"])
            .set_file_name("project.json")
            .save_file()
        else {
            return;
        };
        self.status_message = Some(match crate::io::save_project(&path, &self.project) {
            Ok(()) => format!("Saved project to {}", path.display()),
            Err(e) => format!("Failed to save project: {e}"),
        });
    }

    fn open_project_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Project", &["json"])
            .pick_file()
        else {
            return;
        };
        match crate::io::load_project(&path) {
            Ok(project) => {
                self.canvas = Canvas::new(
                    project.material.width_mm as f32,
                    project.material.depth_mm as f32,
                );
                self.project = project;
                self.history = CommandHistory::default();
                self.selected_defect = None;
                self.tfm_image = None;
                self.status_message = Some(format!("Loaded project from {}", path.display()));
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to load project: {e}"));
            }
        }
    }

    /// Runs the FMC simulation over the current project, then reconstructs a
    /// TFM image over the material's full extent at the default resolution.
    fn run_simulation(&mut self) {
        let simulator = FmcSimulator::default();
        let fmc = simulator.simulate(
            &self.project.material,
            &self.project.probe,
            &self.project.defects,
        );
        let grid = TfmGrid::from_fmc(&fmc);
        self.tfm_image = Some(TfmReconstructor::reconstruct(&fmc, grid));
        self.tfm_generation += 1;
        self.fmc_data = Some(fmc);
    }

    fn export_fmc_dialog(&mut self) {
        let Some(fmc) = &self.fmc_data else {
            self.status_message = Some("Run Simulate before exporting FMC data.".to_string());
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("FMC data", &["h5"])
            .set_file_name("fmc_data.h5")
            .save_file()
        else {
            return;
        };
        self.status_message = Some(match crate::io::write_fmc_file(&path, fmc) {
            Ok(()) => format!("Exported FMC data to {}", path.display()),
            Err(e) => format!("Failed to export FMC data: {e}"),
        });
    }

    fn import_fmc_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("FMC data", &["h5"])
            .pick_file()
        else {
            return;
        };
        match crate::io::read_fmc_file(&path) {
            Ok(fmc) => {
                self.status_message = Some(format!(
                    "Loaded {} ({} elements, {} samples @ {:.1} MHz). Review metadata, then reconstruct.",
                    path.display(),
                    fmc.metadata.num_elements,
                    fmc.metadata.num_samples,
                    fmc.metadata.sample_rate_mhz,
                ));
                self.imported_fmc = Some(fmc);
            }
            Err(e) => {
                self.imported_fmc = None;
                self.status_message = Some(format!("Failed to import FMC data: {e}"));
            }
        }
    }

    fn reconstruct_imported_fmc(&mut self) {
        if let Some(fmc) = &self.imported_fmc {
            let grid = TfmGrid::from_fmc(fmc);
            self.tfm_image = Some(TfmReconstructor::reconstruct(fmc, grid));
            self.tfm_generation += 1;
        }
    }

    fn export_png_dialog(&mut self) {
        let Some(tfm_image) = &self.tfm_image else {
            self.status_message = Some("Run Simulate before exporting an image.".to_string());
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name("reconstruction.png")
            .save_file()
        else {
            return;
        };
        let result = crate::ui::export_png(
            &path,
            tfm_image,
            self.heatmap.dynamic_range_db,
            self.heatmap.gain_db,
            self.heatmap.colormap,
        );
        self.status_message = Some(match result {
            Ok(()) => format!("Exported reconstruction image to {}", path.display()),
            Err(e) => format!("Failed to export image: {e}"),
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);

        egui::SidePanel::right("control_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("FMC-TFM");
                ui.separator();

                egui::CollapsingHeader::new("File")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Save Project").clicked() {
                                self.save_project_dialog();
                            }
                            if ui.button("Open Project").clicked() {
                                self.open_project_dialog();
                            }
                        });
                        ui.horizontal(|ui| {
                            if ui.button("Export FMC").clicked() {
                                self.export_fmc_dialog();
                            }
                            if ui.button("Import FMC").clicked() {
                                self.import_fmc_dialog();
                            }
                        });
                        if let Some(fmc) = &self.imported_fmc {
                            let metadata = fmc.metadata.clone();
                            let mut reconstruct_clicked = false;
                            ui.group(|ui| {
                                ui.label("Imported FMC metadata:");
                                ui.label(format!("Elements: {}", metadata.num_elements));
                                ui.label(format!("Pitch: {:.2} mm", metadata.pitch_mm));
                                ui.label(format!(
                                    "Frequency: {:.1} MHz",
                                    metadata.center_frequency_mhz
                                ));
                                ui.label(format!(
                                    "Sample rate: {:.1} MHz, {} samples",
                                    metadata.sample_rate_mhz, metadata.num_samples
                                ));
                                ui.label(format!(
                                    "Material: {:.0} m/s, {:.0}x{:.0} mm",
                                    metadata.material_velocity_mps,
                                    metadata.material_width_mm,
                                    metadata.material_depth_mm
                                ));
                                if ui.button("Reconstruct").clicked() {
                                    reconstruct_clicked = true;
                                }
                            });
                            if reconstruct_clicked {
                                self.reconstruct_imported_fmc();
                            }
                        }
                    });

                egui::CollapsingHeader::new("Material")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.show_material_controls(ui);
                    });

                egui::CollapsingHeader::new("Probe")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.show_probe_controls(ui);
                    });

                egui::CollapsingHeader::new("Tools")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.show_tool_buttons(ui);
                    });

                egui::CollapsingHeader::new("Defects")
                    .default_open(true)
                    .show(ui, |ui| {
                        self.show_defect_list(ui);
                    });

                egui::CollapsingHeader::new("Reconstruction")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Simulate").clicked() {
                                self.run_simulation();
                            }
                            if ui.button("Export PNG").clicked() {
                                self.export_png_dialog();
                            }
                        });
                        self.heatmap.show_controls(ui);
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Undo:");
                    ui.add_enabled(self.history.can_undo(), egui::Button::new("⟲"))
                        .clicked()
                        .then(|| self.history.undo(&mut self.project));
                    ui.add_enabled(self.history.can_redo(), egui::Button::new("⟳"))
                        .clicked()
                        .then(|| self.history.redo(&mut self.project));
                });

                if let Some(message) = &self.status_message {
                    ui.separator();
                    ui.label(message);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                let response =
                    self.canvas
                        .show(&mut columns[0], &self.project.defects, self.selected_defect);
                self.handle_canvas_interaction(&response);

                self.heatmap.show_image(
                    &mut columns[1],
                    self.tfm_image.as_ref(),
                    self.tfm_generation,
                    &mut self.canvas.zoom,
                    &mut self.canvas.pan,
                );
            });
        });
    }
}

impl App {
    fn show_material_controls(&mut self, ui: &mut egui::Ui) {
        let mut preset_changed = false;
        let current_preset = self.current_material_preset();

        egui::ComboBox::from_label("Preset")
            .selected_text(format!("{:?}", current_preset))
            .show_ui(ui, |ui| {
                for preset in [
                    MaterialPreset::Steel,
                    MaterialPreset::Aluminum,
                    MaterialPreset::Copper,
                    MaterialPreset::Water,
                    MaterialPreset::Acrylic,
                ] {
                    if ui
                        .selectable_value(
                            &mut preset_changed,
                            current_preset != preset,
                            format!("{:?}", preset),
                        )
                        .clicked()
                    {
                        self.project.material = crate::core::Material::from_preset(
                            preset,
                            self.project.material.width_mm,
                            self.project.material.depth_mm,
                        );
                    }
                }
            });

        ui.horizontal(|ui| {
            ui.label("Velocity:");
            ui.label(format!("{:.0} m/s", self.project.material.velocity_mps));
        });
    }

    fn current_material_preset(&self) -> MaterialPreset {
        match self.project.material.velocity_mps as i32 {
            5900 => MaterialPreset::Steel,
            6300 => MaterialPreset::Aluminum,
            4700 => MaterialPreset::Copper,
            1480 => MaterialPreset::Water,
            2700 => MaterialPreset::Acrylic,
            _ => MaterialPreset::Steel,
        }
    }

    fn show_probe_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Elements:");
            ui.label(format!("{}", self.project.probe.num_elements));
        });
        ui.horizontal(|ui| {
            ui.label("Pitch:");
            ui.label(format!("{:.2} mm", self.project.probe.pitch_mm));
        });
        ui.horizontal(|ui| {
            ui.label("Frequency:");
            ui.label(format!(
                "{:.1} MHz",
                self.project.probe.center_frequency_mhz
            ));
        });
    }

    fn show_tool_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tool_mode, ToolMode::Select, "Select");
            ui.selectable_value(&mut self.tool_mode, ToolMode::AddPoint, "Point");
            ui.selectable_value(&mut self.tool_mode, ToolMode::AddCrack, "Crack");
            ui.selectable_value(&mut self.tool_mode, ToolMode::AddVoid, "Void");
        });
    }

    fn show_defect_list(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("{} defects", self.project.defects.len()));

        let mut to_delete = None;
        for (i, defect) in self.project.defects.iter().enumerate() {
            let (x, y) = defect.position();
            let label = match defect {
                Defect::PointReflector(_) => format!("Point ({:.1}, {:.1})", x, y),
                Defect::Crack(_) => format!("Crack ({:.1}, {:.1})", x, y),
                Defect::Void(_) => format!("Void ({:.1}, {:.1})", x, y),
                Defect::Porosity(_) => format!("Porosity ({:.1}, {:.1})", x, y),
                Defect::PlanarDefect(_) => format!("Planar ({:.1}, {:.1})", x, y),
            };

            ui.horizontal(|ui| {
                let selected = self.selected_defect == Some(i);
                if ui.selectable_label(selected, label).clicked() {
                    self.selected_defect = Some(i);
                }
                if ui.small_button("×").clicked() {
                    to_delete = Some(i);
                }
            });
        }

        if let Some(idx) = to_delete {
            let cmd = RemoveDefect::new(idx);
            self.history.execute(Box::new(cmd), &mut self.project);
            if self.selected_defect == Some(idx) {
                self.selected_defect = None;
            }
        }
    }

    fn handle_canvas_interaction(&mut self, response: &egui::Response) {
        let rect = response.rect;

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let world = self.canvas.screen_to_world(pos, rect);
                let x = world.x as f64;
                let y = world.y as f64;

                match self.tool_mode {
                    ToolMode::Select => {
                        self.selected_defect = self.defect_at_position(x, y, 3.0);
                    }
                    ToolMode::AddPoint => {
                        let defect = Defect::PointReflector(PointReflector {
                            x,
                            y,
                            amplitude: 1.0,
                        });
                        let cmd = AddDefect::new(defect);
                        self.history.execute(Box::new(cmd), &mut self.project);
                    }
                    ToolMode::AddCrack => {
                        let defect = Defect::Crack(crate::core::Crack {
                            x,
                            y,
                            length: 5.0,
                            angle: 0.0,
                            amplitude: 1.0,
                        });
                        let cmd = AddDefect::new(defect);
                        self.history.execute(Box::new(cmd), &mut self.project);
                    }
                    ToolMode::AddVoid => {
                        let defect = Defect::Void(crate::core::Void {
                            x,
                            y,
                            radius: 2.0,
                            amplitude: 1.0,
                        });
                        let cmd = AddDefect::new(defect);
                        self.history.execute(Box::new(cmd), &mut self.project);
                    }
                }
            }
        }

        if self.tool_mode == ToolMode::Select {
            if response.drag_started_by(egui::PointerButton::Primary) {
                if let Some(pos) = response.interact_pointer_pos() {
                    let world = self.canvas.screen_to_world(pos, rect);
                    if let Some(idx) = self.defect_at_position(world.x as f64, world.y as f64, 3.0)
                    {
                        self.selected_defect = Some(idx);
                        self.drag_start_pos = Some(self.project.defects[idx].position());
                    }
                }
            }

            if response.dragged_by(egui::PointerButton::Primary) {
                if let (Some(idx), Some(_start)) = (self.selected_defect, self.drag_start_pos) {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let world = self.canvas.screen_to_world(pos, rect);
                        self.project.defects[idx].set_position(world.x as f64, world.y as f64);
                    }
                }
            }

            if response.drag_stopped_by(egui::PointerButton::Primary) {
                if let (Some(idx), Some(old_pos)) = (self.selected_defect, self.drag_start_pos) {
                    let new_pos = self.project.defects[idx].position();
                    if old_pos != new_pos {
                        self.project.defects[idx].set_position(old_pos.0, old_pos.1);
                        let cmd = MoveDefect::new(idx, old_pos, new_pos);
                        self.history.execute(Box::new(cmd), &mut self.project);
                    }
                    self.drag_start_pos = None;
                }
            }
        }
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

    #[test]
    fn app_default_tool_is_select() {
        let app = App::default();
        assert_eq!(app.tool_mode, ToolMode::Select);
        assert!(app.selected_defect.is_none());
    }

    #[test]
    fn defect_at_position_finds_nearby() {
        let mut app = App::default();
        app.project
            .defects
            .push(Defect::PointReflector(PointReflector {
                x: 10.0,
                y: 20.0,
                amplitude: 1.0,
            }));

        assert_eq!(app.defect_at_position(10.0, 20.0, 3.0), Some(0));
        assert_eq!(app.defect_at_position(11.0, 20.0, 3.0), Some(0));
        assert_eq!(app.defect_at_position(50.0, 50.0, 3.0), None);
    }

    #[test]
    fn app_starts_with_no_reconstruction() {
        let app = App::default();
        assert!(app.tfm_image.is_none());
        assert_eq!(app.tfm_generation, 0);
    }

    #[test]
    fn run_simulation_populates_tfm_image_and_bumps_generation() {
        let mut app = App::default();
        // Keep this small: the default 64-element/300x300 case is a
        // multi-hundred-ms release-mode operation and much slower in debug.
        app.project.probe.num_elements = 4;
        app.project.material.width_mm = 20.0;
        app.project.material.depth_mm = 20.0;
        app.project
            .defects
            .push(Defect::PointReflector(PointReflector {
                x: 0.0,
                y: 10.0,
                amplitude: 1.0,
            }));

        app.run_simulation();

        let image = app
            .tfm_image
            .as_ref()
            .expect("reconstruction should be set");
        assert_eq!(image.grid.width_mm, 20.0);
        assert_eq!(image.grid.depth_mm, 20.0);
        assert_eq!(app.tfm_generation, 1);

        app.run_simulation();
        assert_eq!(app.tfm_generation, 2);
    }
}
