// use eframe::egui;
// use serde::{Serialize, Deserialize};
// use std::error::Error;
// use tokio_serial::SerialPort;
// use tokio::runtime::Runtime;
// use std::sync::mpsc;
// use std::thread;
// use std::io::Read;
// use std::time::Duration;

// #[repr(C)]
// #[derive(Serialize, Deserialize)]
// struct Message {
//     content: String,
// }

// struct SerialApp {
//     runtime: Runtime,
//     port: Option<Box<dyn SerialPort>>,
//     rx: mpsc::Receiver<u8>,
//     received_data: Vec<u8>,
// }

// impl SerialApp {
//     fn new(cc: &eframe::CreationContext) -> Self {
//         re_ui::apply_style_and_install_loaders(&cc.egui_ctx);

//         // Initialize Tokio runtime
//         let runtime = Runtime::new().expect("Failed to create Tokio runtime");

//         // Create a channel for receiving serial data
//         let (tx, rx) = mpsc::channel();

//         // Setup serial port
//         let mut port = runtime.block_on(async {
//             tokio_serial::new("COM8", 9600)
//                 .open()
//                 .ok()
//         });

//         // Start reader thread if port is available
//         if let Some(ref mut port) = port {
//             let mut reader_port = port.try_clone().expect("Failed to clone port");
//             thread::spawn(move || {
//                 let mut serial_buf: [u8; 1] = [0; 1];
//                 loop {
//                     // Read a single byte
//                     if let Ok(bytes_read) = reader_port.read(&mut serial_buf) {
//                         if bytes_read > 0 {
//                             let byte = serial_buf[0];
//                             println!("Received byte: {}", byte);
//                             if tx.send(byte).is_err() {
//                                 break;
//                             }
//                         }
//                     }
//                     // Small delay to prevent busy-waiting
//                     thread::sleep(Duration::from_millis(10));
//                 }
//             });
//         }

//         Self {
//             runtime,
//             port,
//             rx,
//             received_data: Vec::new(),
//         }
//     }

//     fn send_message(&mut self, message: &Message) -> Result<(), Box<dyn Error>> {
//         if let Some(port) = &mut self.port {
//             let serialized = postcard::to_allocvec(&message)?;
//             println!("Sending bytes: {:?}", serialized);
//             port.write_all(&serialized)?;
//         }
//         Ok(())
//     }
// }

// impl eframe::App for SerialApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         // Check for received data
//         while let Ok(byte) = self.rx.try_recv() {
//             self.received_data.push(byte);
//         }

//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.vertical(|ui| {
//                 if ui.button("Send Hello World").clicked() {
//                     let message = Message {
//                         content: "Hello World".to_string(),
//                     };

//                     if let Err(e) = self.send_message(&message) {
//                         eprintln!("Failed to send message: {}", e);
//                     }
//                 }

//                 // Display received data
//                 if !self.received_data.is_empty() {
//                     ui.label(format!(
//                         "Received data: {:?}",
//                         String::from_utf8_lossy(&self.received_data)
//                     ));
//                 }
//             });
//         });
//     }
// }

// fn main() -> Result<(), eframe::Error> {
//     let native_options = eframe::NativeOptions {
//         viewport: egui::ViewportBuilder::default(),
//         ..Default::default()
//     };

//     eframe::run_native(
//         "Serial Communication",
//         native_options,
//         Box::new(|cc| Ok(Box::new(SerialApp::new(cc))))
//     )
// }


use eframe::egui;
use serde::{Serialize, Deserialize};
use std::error::Error;
use tokio_serial::SerialPort;
use tokio::runtime::Runtime;
use std::sync::mpsc;
use std::thread;
use std::io::Read;
use std::time::Duration;

// Commands
#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
enum CommandID {
    Ping,
    WriteRegister,
    ReadRegister,
    PerformAction
}

// Frames
#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
struct WriteRegisterFrame  {
    address: u8,
    value: u8
}

// Base Structure
#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
struct Packet {
    command_id: CommandID,
    payload: Vec<u8>,
}

struct SerialApp {
    runtime: Runtime,
    port: Option<Box<dyn SerialPort>>,
    rx: mpsc::Receiver<u8>,
    received_data: Vec<u8>,

    register: u8,
    value: u8
}

impl SerialApp {
    fn new(cc: &eframe::CreationContext) -> Self {
        re_ui::apply_style_and_install_loaders(&cc.egui_ctx);

        // Initialize Tokio runtime
        let runtime = Runtime::new().expect("Failed to create Tokio runtime");

        // Create a channel for receiving serial data
        let (tx, rx) = mpsc::channel();

        // Setup serial port
        let mut port = runtime.block_on(async {
            tokio_serial::new("COM8", 9600)
                .open()
                .ok()
        });

        // Start reader thread if port is available
        if let Some(ref mut port) = port {
            let mut reader_port = port.try_clone().expect("Failed to clone port");
            thread::spawn(move || {
                let mut serial_buf: [u8; 1] = [0; 1];
                loop {
                    // Read a single byte
                    if let Ok(bytes_read) = reader_port.read(&mut serial_buf) {
                        if bytes_read > 0 {
                            let byte = serial_buf[0];
                            println!("Received byte: {}", byte);
                            if tx.send(byte).is_err() {
                                break;
                            }
                        }
                    }
                    // Small delay to prevent busy-waiting
                    thread::sleep(Duration::from_millis(10));
                }
            });
        }

        Self {
            runtime,
            port,
            rx,
            received_data: Vec::new(),
            value: 0,
            register: 0,
        }
    }

    fn send_message(&mut self, message: &Packet) -> Result<(), Box<dyn Error>> {
        if let Some(port) = &mut self.port {
            let mut serialized = postcard::to_allocvec(&message)?;
            // SOF
            serialized.insert(0, 69);
            serialized.remove(2);

            // CRC
            serialized.push(255);

            println!("Sending bytes: {:?}", serialized);
            port.write_all(&serialized)?;
        }
        Ok(())
    }
}

impl eframe::App for SerialApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for received data
        while let Ok(byte) = self.rx.try_recv() {
            self.received_data.push(byte);
        }

        egui::SidePanel::left("left_panel").show(ctx, |ui| {

        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Register: ");
                    ui.add(
                        egui::DragValue::new(&mut self.register)
                            .speed(1.0)
                            .clamp_existing_to_range(true)
                            .range(0..=255)
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Value: ");
                    ui.add(
                        egui::DragValue::new(&mut self.value)
                            .speed(1.0)
                            .clamp_existing_to_range(true)
                            .range(0..=255)
                    );
                });

                if ui.button("Write Register").clicked() {
                    let write_register_frame = WriteRegisterFrame {
                        address: self.register,
                        value: self.value,
                    };

                    let packet = Packet {
                        command_id: CommandID::WriteRegister,
                        payload: postcard::to_allocvec(&write_register_frame).expect("Failed to serialize packet"),
                    };

                    self.send_message(&packet).expect("Failed to send message");
                }

                // Display received data
                if !self.received_data.is_empty() {
                    ui.label(format!(
                        "Received data: {:?}",
                        String::from_utf8_lossy(&self.received_data)
                    ));
                }
            });
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {

        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    println!("HEllo");

    eframe::run_native(
"Eos",
        native_options,
        Box::new(|cc| Ok(Box::new(SerialApp::new(cc))))
    )
}
