use std::io::{Read,Write};

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GetNumberOfEvent,
    ExtractData,
    ClearMemory,
    CompareTime,
    All
}




fn main() {
    println!("Hello, world!");
    let cli = Cli::parse();
    let mut serial = serialport::new("/dev/ttyUSB0", 9600).open_native().expect("Failed to open port");
    serial.read();
}
