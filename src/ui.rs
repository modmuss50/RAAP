use eframe::egui;
use settings::Settings;
use std::cmp::min;
use std::default::Default;
use std::ops::Sub;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};
use thousands::Separable;

use crate::denoise::denoise;
use crate::plot::plot;
use crate::settings;
use crate::{adsb, data};

pub struct Channels {
    pub plot_rx: mpsc::Receiver<data::Point>,
    pub connection_state_rx: mpsc::Receiver<adsb::ConnectionState>,

    pub connect_tx: mpsc::Sender<String>,
}

pub fn run(channels: Channels) {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 900.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Plotter",
        options,
        Box::new(|_cc| Box::new(Plotter::new(channels))),
    );
}

struct HistoricalData {
    oldest_point: SystemTime,
    newest_point: SystemTime,
}

struct Plotter {
    points: Vec<data::Point>,
    connection_state: adsb::ConnectionState,
    historical_data: Option<HistoricalData>,
    open_settings: bool,
    time_offset: u32,

    settings: Settings,
    channels: Channels,
}

impl Plotter {
    fn new(channels: Channels) -> Self {
        Self {
            points: vec![],
            connection_state: adsb::ConnectionState::Disconnected,
            historical_data: None,
            open_settings: false,
            time_offset: 0,
            settings: Settings::default(),

            channels,
        }
    }

    fn recv(&mut self) {
        for plot in self.channels.plot_rx.try_iter() {
            self.points.push(plot);
        }

        if self.historical_data.is_none() {
            self.prune_old_data();
        }

        if let Ok(state) = self.channels.connection_state_rx.try_recv() {
            self.connection_state = state;
        }
    }

    fn prune_old_data(&mut self) {
        let now = SystemTime::now();

        // We know the points are always ordered, so we can save iterating over the whole vec.
        loop {
            if let Some(plot) = self.points.first() {
                let seconds_ago = match now.duration_since(plot.time) {
                    Ok(n) => n,
                    Err(..) => panic!("Point is from the future?!"),
                };

                if seconds_ago.as_secs() > u64::from(self.settings.max_data_age) {
                    self.points.remove(0);
                    continue;
                }
            }

            break;
        }
    }

    fn main_ui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let start = SystemTime::now();

        egui::CentralPanel::default().show(ctx, |ui| {
            // TODO can we only do this when we have new data?
            ui.ctx().request_repaint_after(Duration::from_millis(25));
            self.recv();
            self.handle_key_press(ctx);

            // Header
            ui.horizontal(|ui| {
                if ui.button("Settings").clicked() {
                    self.open_settings = true;
                }

                let points_len = self.points.len();

                match self.connection_state {
                    adsb::ConnectionState::Disconnected => {
                        if ui.button("Connect").clicked() {
                            self.historical_data = None;
                            self.points.clear();

                            self.channels
                                .connect_tx
                                .send(self.settings.hostname.clone())
                                .expect("Unable to connect");
                        }

                        if ui.button("Load").clicked() {
                            self.load_historical();
                        }
                    }
                    adsb::ConnectionState::Connecting => {
                        ui.spinner();
                    }
                    adsb::ConnectionState::Connected => {
                        if ui.button("Disconnect").clicked() {
                            todo!()
                        }

                        if points_len > 0 && ui.button("Save").clicked() {
                            if let Err(e) = data::write(
                                "data.raap",
                                data::Plot {
                                    points: self.points.clone(),
                                },
                            ) {
                                eprintln!("Failed to write: {e}");
                            }
                        }
                    }
                }

                ui.label(format!("Points: {:}", points_len.separate_with_commas()));
            });

            ui.separator();

            if let Some(historical_data) = &self.historical_data {
                let data_age_delta = historical_data
                    .newest_point
                    .duration_since(historical_data.oldest_point)
                    .expect("Oldest point must be older");

                let max_offset = data_age_delta.as_secs();

                if max_offset > u64::from(self.settings.max_display_age) {
                    let key_scroll_amount = 60;
                    if ctx.input().key_pressed(egui::Key::ArrowLeft) {
                        self.time_offset += key_scroll_amount;
                    }
                    if ctx.input().key_pressed(egui::Key::ArrowRight)
                        && self.time_offset > key_scroll_amount
                    {
                        self.time_offset -= key_scroll_amount;
                    }

                    ui.horizontal(|ui| {
                        ui.spacing_mut().slider_width = ui.available_width();
                        ui.add(egui::Slider::new(
                            &mut self.time_offset,
                            0..=u32::try_from(
                                max_offset - u64::from(self.settings.max_display_age),
                            )
                            .unwrap(),
                        ));
                    });

                    self.time_offset = min(
                        self.time_offset,
                        u32::try_from(max_offset - u64::from(self.settings.max_display_age))
                            .unwrap(),
                    );
                }
            }

            // Use the newest point when showing historical data
            // Or use the current time for live data
            let data_x_age = self
                .historical_data
                .as_mut()
                .map_or_else(SystemTime::now, |data| {
                    data.newest_point
                        .sub(Duration::from_secs(u64::from(self.time_offset)))
                });

            plot(ui, &self.points, &self.settings, &data_x_age);
        });

        let end = SystemTime::now();
        let update_time = end.duration_since(start).unwrap().as_millis();

        self.settings.ui(&mut self.open_settings, ctx, update_time);
    }

    fn load_historical(&mut self) {
        let result = match data::read("data.raap") {
            Ok(plot) => Some(plot),
            Err(e) => {
                eprintln!("Unable to read plot: {e}");
                None
            }
        };

        if let Some(plot) = result {
            for x in plot.points {
                self.points.push(x);
            }
        }

        if self.points.is_empty() {
            todo!();
        }

        self.points = denoise(&self.points);

        let oldest_point = self
            .points
            .first()
            .map_or_else(SystemTime::now, |point| point.time);

        let newest_point = self
            .points
            .last()
            .map_or_else(SystemTime::now, |point| point.time);

        self.historical_data = Some(HistoricalData {
            oldest_point,
            newest_point,
        });
    }

    fn handle_key_press(&mut self, ctx: &egui::Context) {
        if ctx.input().key_pressed(egui::Key::PlusEquals) && self.settings.max_display_age >= 120 {
            self.settings.max_display_age -= 60;
        }
        if ctx.input().key_pressed(egui::Key::Minus) {
            self.settings.max_display_age += 60;
        }
    }
}

impl eframe::App for Plotter {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.main_ui(ctx, frame);
    }
}
