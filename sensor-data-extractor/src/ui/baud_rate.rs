use eframe::egui;
use egui::Widget;

const BAUD_RATE_LIST : [u32;4] = [1200,2400,4800,9600];
pub struct BaudRateSelector {
    pub baud_rate: u32,
}

impl Default for BaudRateSelector {
    fn default() -> Self {
        Self {
            baud_rate: 9600,
        }
    }
}

impl Widget for BaudRateSelector {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        egui::ComboBox::from_label("Baud rate")
                    .selected_text(self.baud_rate.to_string())
                    .show_ui(ui, |ui| {
                        //ui.style_mut().wrap = Some(false);
                        //ui.set_min_width(80.0);
                        for rate in BAUD_RATE_LIST {
                            ui.selectable_value(&mut self.baud_rate, rate, rate.to_string());
                        }

                    })
        .response
    }
}
