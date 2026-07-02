use eframe::egui;

pub struct App {
    label: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            label: "FMC-TFM".to_string(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.label);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_default_creates_valid_state() {
        let app = App::default();
        assert_eq!(app.label, "FMC-TFM");
    }
}
