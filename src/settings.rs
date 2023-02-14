use eframe::egui;
use serde::{Deserialize, Serialize};
use std::cmp::max;

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub hostname: String,
    pub show_axis: bool,
    pub max_data_age: u32,
    pub max_display_age: u32,
    pub min_display_height: u32,
    pub max_display_height: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            hostname: "192.168.2.48:30002".to_owned(),
            show_axis: true,
            max_data_age: 60 * 60,
            max_display_age: 10 * 60,
            min_display_height: 0,
            max_display_height: 70_000,
        }
    }
}

impl Settings {
    pub fn ui(&mut self, open: &mut bool, ctx: &egui::Context, update_time: u128) {
        egui::Window::new("Settings")
            .open(open)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let hostname_label = ui.label("Hostname + port: ");
                    ui.text_edit_singleline(&mut self.hostname)
                        .labelled_by(hostname_label.id);
                });
                let min_fmt = |x, _| format!("{:.0} mins", x / 60.0);
                let ft_fmt = |x, _| format!("{:.0} ft", x);

                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut self.max_display_age, 60..=60 * 60)
                            .custom_formatter(min_fmt)
                            .text("Max display age"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut self.max_data_age, 60..=60 * 60)
                            .custom_formatter(min_fmt)
                            .text("Max data age"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(
                            &mut self.min_display_height,
                            0..=self.max_display_height,
                        )
                        .clamp_to_range(true)
                        .custom_formatter(ft_fmt)
                        .text("Y axis min"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(
                            &mut self.max_display_height,
                            self.min_display_height..=100_000,
                        )
                        .clamp_to_range(true)
                        .custom_formatter(ft_fmt)
                        .text("Y axis max"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.show_axis, "Show Y axis");
                });

                // Data age must be >= to display age
                self.max_data_age = max(self.max_data_age, self.max_display_age);

                ui.label(format!("Update time: {:}ms", update_time));
            });
    }
}
