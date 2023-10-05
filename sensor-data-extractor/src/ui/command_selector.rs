use std::path::PathBuf;

use eframe::egui;
use egui::Widget;

use crate::commands::Command;

pub struct CommandSelector {
    pub selected_command: Option<Command>,
}

const COMMAND_LIST : [Command;7] = [
Command::Ping,
Command::SetSensorId(0),
Command::GetEverything,
Command::GetEverythingAndSave(PathBuf::new()),
Command::ResetSensor,
Command::GetTime,
Command::SyncTime];

impl Default for CommandSelector {
    fn default() -> Self {
        Self {
            selected_command: None,
        }
    }
}

impl Widget for CommandSelector {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            egui::ComboBox::from_label("Command")
                .selected_text(&self.selected_command.map_or("None".to_string(), |v| v.to_string()))
                .show_ui(ui, |ui| {
                    //ui.style_mut().wrap = Some(false);
                    //ui.set_min_width(100.0);
                    ui.selectable_value(&mut self.selected_command, None, "None");
                    for command in COMMAND_LIST {
                        ui.selectable_value(&mut self.selected_command, Some(command), command.to_string());
                    }
                    
                });

            match self.selected_command {
                Some(Command::SetSensorId(_)) => todo!(),
                None => todo!(),
            }
        })
        .response
    }
}
