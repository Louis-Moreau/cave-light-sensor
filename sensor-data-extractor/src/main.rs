#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod serial;
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types

use std::thread::{self, JoinHandle};

use eframe::egui;
use serialport::SerialPortInfo;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        resizable: true,
        initial_window_size: Some(egui::vec2(1600.0, 900.0)),
        ..Default::default()
    };

    // Our application state:
    let mut thread: Option<JoinHandle<_>> = Option::None::<JoinHandle<_>>;
    let mut is_thread_finished = true;
    let mut selected_serial: Option<SerialPortInfo> = None;
    let mut baud_rate: u64 = 9600;
    let mut vec_serial: Vec<SerialPortInfo> = match serialport::available_ports() {
        Ok(v) => v,
        Err(_) => Vec::new(),
    };

    eframe::run_simple_native("Sensor data extractor", options, move |ctx, _frame| {
        ctx.set_pixels_per_point(2f32);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Sensor data extractor");

            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Serial Port")
                    .selected_text(format!(
                        "{}",
                        selected_serial
                            .clone()
                            .map_or("None".to_string(), |v| v.port_name.clone())
                    ))
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.set_min_width(100.0);
                        ui.selectable_value(&mut selected_serial, None, "None");
                        for port in &vec_serial {
                            ui.selectable_value(
                                &mut selected_serial,
                                Some(port.clone()),
                                &port.port_name,
                            );
                        }
                    });

                if ui.button("Refresh").clicked() {
                    vec_serial = match serialport::available_ports() {
                        Ok(v) => v,
                        Err(_) => Vec::new(),
                    }
                }
            });

            egui::ComboBox::from_label("Baud rate")
                .selected_text(format!("{baud_rate:?}"))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(60.0);
                    ui.selectable_value(&mut baud_rate, 9600, "9600");
                    ui.selectable_value(&mut baud_rate, 4800, "4800");
                    ui.selectable_value(&mut baud_rate, 2400, "2400");
                    ui.selectable_value(&mut baud_rate, 1200, "1200");
                });

            is_thread_finished = match &thread {
                Some(t) => t.is_finished(),
                None => true,
            };
            ui.horizontal(|ui| {
                ui.group(|ui| {
                    ui.set_enabled(selected_serial.is_some());
                    if ui.button("Disconnect").clicked() {
                        match thread {
                            Some(t) => (),
                            None => todo!(),
                        };
                    }
                });

                ui.group(|ui| {
                    ui.set_enabled(selected_serial.is_some() && is_thread_finished);
                    if ui.button("Connect").clicked() {
                        thread = Some(thread::spawn(|| {}));
                        is_thread_finished = false;
                    }
                });
            });
        });
    })
}
