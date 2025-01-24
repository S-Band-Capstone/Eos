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

const FREQUENCY_FACTOR: f64 = (2_u32.pow(16)as f64) / 26.0;
const BASE_FREQUENCY_MIN: f64 = 2400.0;
const BASE_FREQUENCY_MAX: f64 = 2843.5;

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

// this struct is the application struct, declares variables that the application itself can see
struct SerialApp {
    runtime: Runtime,
    port: Option<Box<dyn SerialPort>>,
    rx: mpsc::Receiver<u8>,
    received_data: Vec<u8>,
    register: u8,
    value: u8,
    register_value: RegisterValue,
    register_address:RegisterAddress,
    user_input_frequency: String,
    user_input_channel_number: u8,
    user_input_mod_scheme: String,
    is_whitened: bool,
    manchester_enabled: bool,
    user_input_tx_power: i8,
    invalid_frequency_popup: bool,
}

const TEXT_EDIT: Vec2 = Vec2 {
    x: 68.0,
    y: 0.0
};

// Name of all register addresses and their length (16 bits)
struct RegisterAddress {
    iocfg2: u16,
    iocfg1: u16,
    iocfg0: u16,
    sync1: u16,
    sync0: u16,
    pktlen: u16,
    pktctrl1: u16,
    pktctrl0: u16,
    addr: u16,
    channr: u16,
    fsctrl1: u16,
    fsctrl0: u16,
    freq2: u16,
    freq1: u16,
    freq0: u16,
    mdmcfg4: u16,
    mdmcfg3: u16,
    mdmcfg2: u16,
    mdmcfg1: u16,
    mdmcfg0: u16,
    deviatn: u16,
    mcsm2: u16,
    mcsm1: u16,
    mcsm0: u16,
    foccfg: u16,
    bscfg: u16,
    agcctrl2: u16,
    agcctrl1: u16,
    agcctrl0: u16,
    frend1: u16,
    frend0: u16,
    fscal3: u16,
    fscal2: u16,
    fscal1: u16,
    fscal0: u16,
    test2: u16,
    test1: u16,
    test0: u16,
    pa_table0: u16
}

// Register value struct declaration (this is the struct of the actual register values)
struct RegisterValue {
    iocfg2: u8,
    iocfg1: u8,
    iocfg0: u8,
    sync1: u8,
    sync0: u8,
    pktlen: u8,
    pktctrl1: u8,
    pktctrl0: u8,
    addr: u8,
    channr: u8,
    fsctrl1: u8,
    fsctrl0: u8,
    freq2: u8,
    freq1: u8,
    freq0: u8,
    mdmcfg4: u8,
    mdmcfg3: u8,
    mdmcfg2: u8,
    mdmcfg1: u8,
    mdmcfg0: u8,
    deviatn: u8,
    mcsm2: u8,
    mcsm1: u8,
    mcsm0: u8,
    foccfg: u8,
    bscfg: u8,
    agcctrl2: u8,
    agcctrl1: u8,
    agcctrl0: u8,
    frend1: u8,
    frend0: u8,
    fscal3: u8,
    fscal2: u8,
    fscal1: u8,
    fscal0: u8,
    test2: u8,
    test1: u8,
    test0: u8,
    pa_table0: u8
}

// Register value struct is instantiated with baseline values
// the values chosen are the ones that SmartRF Studio 7 shows when you reset the registers on the chip

// Implementation of the SerialApp struct further up, declares startup things, such as port selection & initial variable values
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
            register_value: RegisterValue{
                iocfg2: 0x00,
                iocfg1: 0x00,
                iocfg0: 0x00,
                sync1: 0xD3,
                sync0: 0x91,
                pktlen: 0xFF,
                pktctrl1: 0x04,
                pktctrl0: 0x45,
                addr: 0x00,
                channr: 0x00,
                fsctrl1: 0x0F,
                fsctrl0: 0x00,
                freq2: 0x5E,
                freq1: 0xC4,
                freq0: 0xEC,
                mdmcfg4: 0x8C,
                mdmcfg3: 0x22,
                mdmcfg2: 0x02,
                mdmcfg1: 0x22,
                mdmcfg0: 0xF8,
                deviatn: 0x47,
                mcsm2: 0x07,
                mcsm1: 0x30,
                mcsm0: 0x04,
                foccfg: 0x76,
                bscfg: 0x6C,
                agcctrl2: 0x03,
                agcctrl1: 0x40,
                agcctrl0: 0x91,
                frend1: 0x56,
                frend0: 0x10,
                fscal3: 0xA9,
                fscal2: 0x0A,
                fscal1: 0x20,
                fscal0: 0x0D,
                test2: 0x88,
                test1: 0x11,
                test0: 0x0B,
                pa_table0: 0x00
            },
            register_address: RegisterAddress{
                iocfg2: 0xDF2F,
                iocfg1: 0xDF30,
                iocfg0: 0xDF31,
                sync1: 0xDF00,
                sync0: 0xDF01,
                pktlen: 0xDF02,
                pktctrl1: 0xDF03,
                pktctrl0: 0xDF04,
                addr: 0xDF05,
                channr: 0xDF06,
                fsctrl1: 0xDF07,
                fsctrl0: 0xDF08,
                freq2: 0xDF09,
                freq1: 0xDF0A,
                freq0: 0xDF0B,
                mdmcfg4: 0xDF0C,
                mdmcfg3: 0xDF0D,
                mdmcfg2: 0xDF0E,
                mdmcfg1: 0xDF0F,
                mdmcfg0: 0xDF10,
                deviatn: 0xDF11,
                mcsm2: 0xDF12,
                mcsm1: 0xDF13,
                mcsm0: 0xDF14,
                foccfg: 0xDF15,
                bscfg: 0xDF16,
                agcctrl2: 0xDF17,
                agcctrl1: 0xDF18,
                agcctrl0: 0xDF19,
                frend1: 0xDF1A,
                frend0: 0xDF1B,
                fscal3: 0xDF1C,
                fscal2: 0xDF1D,
                fscal1: 0xDF1E,
                fscal0: 0xDF1F,
                test2: 0xDF23,
                test1: 0xDF24,
                test0: 0xDF25,
                pa_table0: 0xDF2E 
            },
            user_input_frequency: "2464.0".to_string(),
            user_input_channel_number: 0,
            user_input_mod_scheme: "2-FSK".to_string(),
            is_whitened: true,
            manchester_enabled: false,
            user_input_tx_power: -55,
            invalid_frequency_popup: false,
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

    fn update_base_frequency_from_parameter(&mut self) {
        let intermediate_input_frequency_u64 = f64::floor(self.user_input_frequency.parse::<f64>().unwrap() * FREQUENCY_FACTOR) as u64; 
        self.register_value.freq0 = (intermediate_input_frequency_u64 & 0xFF) as u8;
        self.register_value.freq1 = ((intermediate_input_frequency_u64 >> 8) & 0xFF) as u8;
        self.register_value.freq2 = ((intermediate_input_frequency_u64 >> 16) & 0xFF) as u8;
        u64::from_str_radix(format!("{}{}{}", format!("{:08b}", self.register_value.freq2).to_string(), format!("{:08b}", self.register_value.freq1).to_string(), format!("{:08b}", self.register_value.freq0)).as_str(), 2).expect("Invalid binary string").to_string();
    }

    fn update_channel_number_from_parameter(&mut self) {
        self.register_value.channr = self.user_input_channel_number
    }

    fn update_tx_power_from_parameter(&mut self) {
        self.register_value.pa_table0 = 0x00;
        let value = self.user_input_tx_power;
        match value {
            1 => self.register_value.pa_table0 = 0xFF,
            0 => self.register_value.pa_table0 = 0xFE,
            -2 => self.register_value.pa_table0 = 0xBF,
            -4 => self.register_value.pa_table0 = 0xAA,
            -6 => self.register_value.pa_table0 = 0x7F,
            -8 => self.register_value.pa_table0 = 0x99,
            -10 => self.register_value.pa_table0 = 0xCB,
            -12 => self.register_value.pa_table0 = 0x95,
            -14 => self.register_value.pa_table0 = 0x59,
            -16 => self.register_value.pa_table0 = 0x87,
            -18 => self.register_value.pa_table0 = 0xC8,
            -20 => self.register_value.pa_table0 = 0xC1,
            -22 => self.register_value.pa_table0 = 0x83,
            -24 => self.register_value.pa_table0 = 0x53,
            -26 => self.register_value.pa_table0 = 0x54,
            -28 => self.register_value.pa_table0 = 0x41,
            -30 => self.register_value.pa_table0 = 0x44,
            -55 => self.register_value.pa_table0 = 0x00,
            _ => self.register_value.pa_table0 = self.register_value.pa_table0,
        }
    }

    fn update_modulation_scheme_from_parameter(&mut self) {
        self.register_value.mdmcfg2 &= 0x8F;
        let value = self.user_input_mod_scheme.as_str();
        match value {
            "2-FSK" => self.register_value.mdmcfg2 |= 0x00,
            "GFSK" => self.register_value.mdmcfg2 |= 0x10,
            "MSK" => self.register_value.mdmcfg2 |= 0x70,
            _ => self.register_value.mdmcfg2 = self.register_value.mdmcfg2,
        }
    }

    fn update_data_whitening_from_parameters(&mut self) {
        if self.is_whitened {self.register_value.pktctrl0 |= 0x40} else {self.register_value.pktctrl0 &= 0xBF};
    }
    
    fn update_manchester_from_parameters(&mut self) {
        if self.manchester_enabled {self.register_value.mdmcfg2 |= 0x08} else {self.register_value.mdmcfg2 &= 0xF7};
    }

    fn get_concatenated_freq(&self) -> String {
        let intermediate_decimal_frequency = ((self.register_value.freq2 as u32) << 16) | ((self.register_value.freq1 as u32) << 8) | self.register_value.freq0 as u32;
        let result = intermediate_decimal_frequency as f64 / FREQUENCY_FACTOR;
        format!("{}", result)
    }
}

// implementation of the UI for SerialApp
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
                    let frequency_text_box = ui.add(egui::TextEdit::singleline(&mut self.user_input_frequency).min_size(TEXT_EDIT));
                    if frequency_text_box.changed() {
                        self.update_base_frequency_from_parameter();
                    }
                    if frequency_text_box.lost_focus() {
                        if let Ok(value) = self.user_input_frequency.trim().parse::<f64>() {
                            // Check if the value is out of bounds
                            if value < 2400.0 || value > 2483.5 {
                                self.invalid_frequency_popup = true; // Trigger the popup
                            }
                        } else {
                            // Show popup for invalid input
                            self.invalid_frequency_popup = true;
                        }
                    }
                    if self.invalid_frequency_popup {
                        egui::Window::new("Invalid Input")
                            .collapsible(false)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.label("The base frequency must be between 2400 and 2483.5!");
                                if ui.button("OK").clicked() {
                                    self.invalid_frequency_popup = false; // Close the popup
                                }
                            });
                    }
                    ui.add(
                        egui::TextEdit::singleline(&mut self.get_concatenated_freq()).clip_text(true).desired_width(68.0)
                    );
                });
                ui.label("MHz");
            });
            ui.horizontal(|ui| {

            });

            egui::Grid::new("channel").show(ui, |ui| {
                ui.label("Channel Number");
                ui.horizontal(|ui| {
                    let channel_number_box = ui.add(egui::DragValue::new(&mut self.user_input_channel_number)
                        .speed(1.0)
                        .clamp_existing_to_range(true)
                        .range(0..=255));
                    if channel_number_box.changed() {
                        self.update_channel_number_from_parameter();
                    }
                    ui.label(self.register_value.channr.to_string());
                });
            });

            egui::Grid::new("modulation").show(ui, |ui| {
                ui.label("Modulation Scheme");
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label("")
                        .selected_text(&self.user_input_mod_scheme)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.user_input_mod_scheme, "2-FSK".to_string(), "2-FSK");
                            ui.selectable_value(&mut self.user_input_mod_scheme, "GFSK".to_string(), "GFSK");
                            ui.selectable_value(&mut self.user_input_mod_scheme, "MSK".to_string(), "MSK")
                    });
                    self.update_modulation_scheme_from_parameter();
                });
            });
            egui::Grid::new("whitening").show(ui, |ui| {
                ui.label("Data Whitening");
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.is_whitened, "Data Whitening").clicked() {
                        self.update_data_whitening_from_parameters();
                    }
                });
                ui.label(self.register_value.pktctrl0.to_string());
            });
            egui::Grid::new("enable").show(ui, |ui| {
                ui.label("Manchester Enable");
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.manchester_enabled, "Manchester Enable").clicked() {
                        self.update_manchester_from_parameters();
                    }
                });
                ui.label(self.register_value.mdmcfg2.to_string());
            });

            egui::Grid::new("power").show(ui, |ui| {
                ui.label("TX Power");
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label("TX Power")
                        .selected_text(&self.user_input_tx_power.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.user_input_tx_power, 1, "1");
                            ui.selectable_value(&mut self.user_input_tx_power, 0, "0");
                            ui.selectable_value(&mut self.user_input_tx_power, -2, "-2");
                            ui.selectable_value(&mut self.user_input_tx_power, -4, "-4");
                            ui.selectable_value(&mut self.user_input_tx_power, -6, "-6");
                            ui.selectable_value(&mut self.user_input_tx_power, -8, "-8");
                            ui.selectable_value(&mut self.user_input_tx_power, -10, "-10");
                            ui.selectable_value(&mut self.user_input_tx_power, -12, "-12");
                            ui.selectable_value(&mut self.user_input_tx_power, -14, "-14");
                            ui.selectable_value(&mut self.user_input_tx_power, -16, "-16");
                            ui.selectable_value(&mut self.user_input_tx_power, -18, "-18");
                            ui.selectable_value(&mut self.user_input_tx_power, -20, "-20");
                            ui.selectable_value(&mut self.user_input_tx_power, -22, "-22");
                            ui.selectable_value(&mut self.user_input_tx_power, -24, "-24");
                            ui.selectable_value(&mut self.user_input_tx_power, -26, "-26");
                            ui.selectable_value(&mut self.user_input_tx_power, -28, "-28");
                            ui.selectable_value(&mut self.user_input_tx_power, -30, "-30");
                            ui.selectable_value(&mut self.user_input_tx_power, -55, "-55");
                    });
                    self.update_tx_power_from_parameter();
                    ui.label(self.register_value.pa_table0.to_string());
                });
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

// literally just start the app
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
