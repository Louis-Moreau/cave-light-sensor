use tokio::sync::oneshot;
use std::result::Result::Ok;
use tokio::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};
use tokio::runtime::Runtime;
use tokio::task;
use tokio_serial::SerialStream;

use crate::commands::{Commands, ping};

pub fn spawn_command_handler(
    mut port: SerialStream,
    mut commands_receiver: Receiver<Commands>,
    logs_sender: Sender<String>,
    quit : oneshot::Receiver<()>
) -> JoinHandle<()> {
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {

            task::spawn(async move {
                loop {
                    let command = match commands_receiver.recv().await {
                        Some(c) => c,
                        None => break,
                    };
                    let _ = match command {
                        Commands::Ping => _ = ping(&mut port, &logs_sender).await,
                        Commands::SetSensorId => (),
                        Commands::GetEverything => (),
                        Commands::GetEverythingAndSave(_) => (),
                        Commands::ResetSensor => (),
                        Commands::GetTime => (),
                        Commands::SyncTime => (),
                    };
                }
            });

            match quit.await {
                Ok(_) => (),
                Err(_) => (),
            }
        });
    })
}
