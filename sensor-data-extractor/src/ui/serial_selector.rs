use eframe::egui;
use egui::Widget;
use tokio_serial::SerialPortInfo;
pub struct SerialSelector {
    pub selected_serial: Option<SerialPortInfo>,
    vec_serial: Vec<SerialPortInfo>,
}

impl Default for SerialSelector {
    fn default() -> Self {
        Self {
            selected_serial: None,
            vec_serial: Vec::new(),
        }
    }
}

impl Widget for SerialSelector {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Serial Port")
                .selected_text(&self.selected_serial.map_or("None".to_string(), |v| v.port_name))
                .show_ui(ui, |ui| {
                    //ui.style_mut().wrap = Some(false);
                    //ui.set_min_width(100.0);
                    ui.selectable_value(&mut self.selected_serial, None, "None");
                    for port in self.vec_serial {
                        ui.selectable_value(&mut self.selected_serial, Some(port.clone()), &port.port_name);
                    }
                });

            if ui.button("Refresh").clicked() {
                self.update_available_ports();
            }
        })
        .response
    }
}

impl SerialSelector {
    fn update_available_ports(&mut self) {
        self.vec_serial = match tokio_serial::available_ports() {
            Ok(v) => v,
            Err(_) => Vec::new(),
        }
    }
}
