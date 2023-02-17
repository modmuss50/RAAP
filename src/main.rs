#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::mpsc;
use std::thread;

mod adsb;
mod data;
mod denoise;
mod plot;
mod settings;
mod ui;

fn main() {
    let (plot_tx, plot_rx) = mpsc::channel::<data::Point>();
    let (connect_tx, connect_rx) = mpsc::channel::<String>();
    let (connection_state_tx, connection_state_rx) = mpsc::channel::<adsb::ConnectionState>();

    thread::Builder::new()
        .name("ADSB".to_string())
        .spawn(move || {
            adsb::run(adsb::Channels {
                connect_rx,
                plot_tx,
                connection_state_tx,
            });
        })
        .expect("Failed to start ADSB thread");

    ui::run(ui::Channels {
        plot_rx,
        connection_state_rx,
        connect_tx,
    });
}
