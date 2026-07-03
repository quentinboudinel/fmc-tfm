use crate::core::{
    AddDefect, CommandHistory, Defect, MaterialPreset, MoveDefect, PointReflector, Project,
    RemoveDefect,
};
use crate::ui::Canvas;
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
            drag_start_pos: None,
        }
    }
}

impl App {
    fn handle_keyboard(&mut self, ctx: &egui::Context) {
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
        });
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
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);

        egui::SidePanel::right("control_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("FMC-TFM");
                ui.separator();

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
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let response = self
                .canvas
                .show(ui, &self.project.defects, self.selected_defect);
            self.handle_canvas_interaction(&response);
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
}
