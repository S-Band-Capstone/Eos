use eframe::egui::Vec2;
use eframe::egui::{self};
use serde::{Serialize, Deserialize};
use std::error::Error;
use tokio_serial::SerialPort;
use tokio::runtime::Runtime;
use std::sync::mpsc;
use std::thread;
use std::io::Read;
use std::time::Duration;
use num_base::Based;

const FREQUENCY_FACTOR: f64 = (2_u32.pow(16)as f64) / 26.0;

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
    value: u8,
    input_frequency: String,
    binary_frequency_string: String,
    binary_frequency: u64,
    rounded_frequency_string: String,
    rounded_frequency: f64
}

const TEXT_EDIT: Vec2 = Vec2 {
    x: 80.0,
    y: 0.0
};

struct RegisterAddress {
    IOCFG2: u16,
    IOCFG1: u16,
    IOCFG0: u16,
    SYNC1: u16,
    SYNC0: u16,
    PKTLEN: u16,
    PKTCTRL1: u16,
    PKTCTRL0: u16,
    ADDR: u16,
    CHANNR: u16,
    FSCTRL1: u16,
    FSCTRL0: u16,
    FREQ2: u16,
    FREQ1: u16,
    FREQ0: u16,
    MDMCFG4: u16,
    MDMCFG3: u16,
    MDMCFG2: u16,
    MDMCFG1: u16,
    MDMCFG0: u16,
    DEVIATN: u16,
    MCSM2: u16,
    MCSM1: u16,
    MCSM0: u16,
    FOCCFG: u16,
    BSCFG: u16,
    AGCCTRL2: u16,
    AGCCTRL1: u16,
    AGCCTRL0: u16,
    FREND1: u16,
    FREND0: u16,
    FSCAL3: u16,
    FSCAL2: u16,
    FSCAL1: u16,
    FSCAL0: u16,
    TEST2: u16,
    TEST1: u16,
    TEST0: u16,
    PA_TABLE0: u16
}

const REGISTER_ADDRESS: RegisterAddress = RegisterAddress {
    IOCFG2: 0xDF2F,
    IOCFG1: 0xDF30,
    IOCFG0: 0xDF31,
    SYNC1: 0xDF00,
    SYNC0: 0xDF01,
    PKTLEN: 0xDF02,
    PKTCTRL1: 0xDF03,
    PKTCTRL0: 0xDF04,
    ADDR: 0xDF05,
    CHANNR: 0xDF06,
    FSCTRL1: 0xDF07,
    FSCTRL0: 0xDF08,
    FREQ2: 0xDF09,
    FREQ1: 0xDF0A,
    FREQ0: 0xDF0B,
    MDMCFG4: 0xDF0C,
    MDMCFG3: 0xDF0D,
    MDMCFG2: 0xDF0E,
    MDMCFG1: 0xDF0F,
    MDMCFG0: 0xDF10,
    DEVIATN: 0xDF11,
    MCSM2: 0xDF12,
    MCSM1: 0xDF13,
    MCSM0: 0xDF14,
    FOCCFG: 0xDF15,
    BSCFG: 0xDF16,
    AGCCTRL2: 0xDF17,
    AGCCTRL1: 0xDF18,
    AGCCTRL0: 0xDF19,
    FREND1: 0xDF1A,
    FREND0: 0xDF1B,
    FSCAL3: 0xDF1C,
    FSCAL2: 0xDF1D,
    FSCAL1: 0xDF1E,
    FSCAL0: 0xDF1F,
    TEST2: 0xDF23,
    TEST1: 0xDF24,
    TEST0: 0xDF25,
    PA_TABLE0: 0xDF2E  
};

struct Registers {
    IOCFG2: u8,
    IOCFG1: u8,
    IOCFG0: u8,
    SYNC1: u8,
    SYNC0: u8,
    PKTLEN: u8,
    PKTCTRL1: u8,
    PKTCTRL0: u8,
    ADDR: u8,
    CHANNR: u8,
    FSCTRL1: u8,
    FSCTRL0: u8,
    FREQ2: u8,
    FREQ1: u8,
    FREQ0: u8,
    MDMCFG4: u8,
    MDMCFG3: u8,
    MDMCFG2: u8,
    MDMCFG1: u8,
    MDMCFG0: u8,
    DEVIATN: u8,
    MCSM2: u8,
    MCSM1: u8,
    MCSM0: u8,
    FOCCFG: u8,
    BSCFG: u8,
    AGCCTRL2: u8,
    AGCCTRL1: u8,
    AGCCTRL0: u8,
    FREND1: u8,
    FREND0: u8,
    FSCAL3: u8,
    FSCAL2: u8,
    FSCAL1: u8,
    FSCAL0: u8,
    TEST2: u8,
    TEST1: u8,
    TEST0: u8,
    PA_TABLE0: u8
}

const REGISTER_VALUE: Registers = Registers {
    IOCFG2: 0x00,
    IOCFG1: 0x00,
    IOCFG0: 0x00,
    SYNC1: 0xD3,
    SYNC0: 0x91,
    PKTLEN: 0xFF,
    PKTCTRL1: 0x04,
    PKTCTRL0: 0x45,
    ADDR: 0x00,
    CHANNR: 0x00,
    FSCTRL1: 0x0F,
    FSCTRL0: 0x00,
    FREQ2: 0x5E,
    FREQ1: 0xC4,
    FREQ0: 0xEC,
    MDMCFG4: 0x8C,
    MDMCFG3: 0x22,
    MDMCFG2: 0x02,
    MDMCFG1: 0x22,
    MDMCFG0: 0xF8,
    DEVIATN: 0x47,
    MCSM2: 0x07,
    MCSM1: 0x30,
    MCSM0: 0x04,
    FOCCFG: 0x76,
    BSCFG: 0x6C,
    AGCCTRL2: 0x03,
    AGCCTRL1: 0x40,
    AGCCTRL0: 0x91,
    FREND1: 0x56,
    FREND0: 0x10,
    FSCAL3: 0xA9,
    FSCAL2: 0x0A,
    FSCAL1: 0x20,
    FSCAL0: 0x0D,
    TEST2: 0x88,
    TEST1: 0x11,
    TEST0: 0x0B,
    PA_TABLE0: 0x00
};

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
            input_frequency: "2464.0".to_string(),
            binary_frequency_string: format!("{}{}{}", format!("{:08b}", REGISTER_VALUE.FREQ2).to_string(), format!("{:08b}", REGISTER_VALUE.FREQ1).to_string(), format!("{:08b}", REGISTER_VALUE.FREQ0).to_string()),
            binary_frequency: 0b010111101100010011101100,
            rounded_frequency_string: "2464.000000".to_string(),
            rounded_frequency: 2464.000000,
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
            egui::Grid::new("freq").show(ui, |ui| {
                ui.label("Base Frequency");
                ui.vertical(|ui| {
                    if ui.add(egui::TextEdit::singleline(&mut self.input_frequency).min_size(TEXT_EDIT)).changed() {
                        // println!("---------------------------------------------------------------------------");
                        // println!("original decimal: {:?}", self.input_frequency);
                        // match self.dec_2_bin(self.input_frequency * (2f64.powf(16.0)/26.0)) {
                        //     Ok(value) => println!("Binary value: {}", value),
                        //     Err(e) => println!("Failed to parse value: {}", e),
                        // }
                        let mut intermediate_input_frequency = self.input_frequency.parse::<f64>().unwrap() * FREQUENCY_FACTOR; 
                        // println!("intermediate input frequency: {:?}", intermediate_input_frequency);
    
    
                        intermediate_input_frequency = f64::floor(intermediate_input_frequency);
                        let intermediate_input_frequency_u64: u64 = intermediate_input_frequency as u64;
                        // println!("intermediate input frequency after floor: {:?}", intermediate_input_frequency.clone());
    
    
                        self.binary_frequency_string = format!("{:b}", intermediate_input_frequency_u64);
                        // println!("binary frequency after doing it the easy way: {:?}", self.binary_frequency_string);
                        // println!("binary representation of {} is {:?}!", intermediate_input_frequency, self.binary_frequency_string);

                        // println!("binary {:?}", self.binary_frequency_string);
    
                        let intermediate_binary_frequency = u64::from_str_radix(&self.binary_frequency_string, 2).expect("Invalid binary string").to_string();
                        self.rounded_frequency_string = (intermediate_binary_frequency.parse::<f64>().unwrap() / FREQUENCY_FACTOR).to_string();
                        // println!("new rounded frequency {:?}", self.rounded_frequency_string);
                        // println!("rounded representation of {} is {:?}!", self.input_frequency, self.rounded_frequency_string);
                    }

                    ui.add(
                        egui::TextEdit::singleline(&mut self.binary_frequency_string.clone()).desired_width(80.0)
                    );

                });
                ui.label("MHz");
            });
            ui.horizontal(|ui| {

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

        egui::CentralPanel::default().show(ctx, |ui| {
            
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
