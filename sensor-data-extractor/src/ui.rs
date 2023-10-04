use std::thread::JoinHandle;

mod serial_selector;
mod baud_rate;

use eframe::egui;
use egui::Widget;
use tokio_serial::{SerialPortInfo, SerialStream};

use self::{serial_selector::SerialSelector, baud_rate::BaudRateSelector};

struct MyApp {
    command_thread: Option<JoinHandle<()>>,
    serial_port: Option<SerialStream>,
    serial_selector : SerialSelector,
    baud_selector : BaudRateSelector
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            command_thread: None,
            serial_port: None,
            serial_selector : SerialSelector::default(),
            baud_selector : BaudRateSelector::default()
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(2f32);
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Sensor data extractor");

                ui.add(self.serial_selector);
                ui.add(self.baud_selector);


                ui.horizontal(|ui| {
                    ui.group(|ui| {
                        ui.set_enabled(selected_serial.is_some() && command_thread.is_none());
                        let button = ui.button("Connect");
                        if let Some(s) = &selected_serial {
                            if button.clicked() {
                                //println!("test")
                                serial_port = Some(SerialStream::open(&tokio_serial::new(&s.port_name, baud_rate)).unwrap());
                            }
                        }
                    });
                });
            });
        });
    }
}
