use eframe::egui::{self};
use serde::{Serialize, Deserialize};
use std::error::Error;
use tokio_serial::SerialPort;
use tokio::runtime::Runtime;
use std::sync::mpsc;
use std::thread;
use std::io::Read;
use std::time::Duration;
mod structs;

const FREQUENCY_FACTOR: f64 = (2_u32.pow(16)as f64) / 26.0;
const BASE_FREQUENCY_MIN: f64 = 2400.0;
const BASE_FREQUENCY_MAX: f64 = 2483.5;
const DEVIATION_MIN: f64 = 1.6;
const DEVATION_MAX: f64 = 381.0;
const DEVIATION_FACTOR: f64 = 26000.0 / 2u32.pow(17)as f64;
const DATA_RATE_MIN: f64 = 0.025;
const DATA_RATE_MAX: f64 = 1622.0;
const DATA_RATE_FACTOR: f64 = 26000.0 / 2u32.pow(28)as f64;

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
    register_value: structs::RegisterValue,
    register_address:structs::RegisterAddress,
    user_input_frequency: String,
    user_input_channel_number: u8,
    user_input_mod_scheme: String,
    is_whitened: bool,
    manchester_enabled: bool,
    user_input_tx_power: i8,
    user_input_phase_transition_time: u8,
    user_input_deviation: String,
    user_input_dr: String,
    invalid_frequency_popup: bool,
    invalid_deviation_popup: bool,
    invalid_dr_popup: bool,
    is_hex: bool,
}

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
            register_value: structs::RegisterValue{
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
            register_address: structs::RegisterAddress{
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
            user_input_phase_transition_time: 0,
            user_input_deviation: "47.7".to_string(),
            user_input_dr: "115.051".to_string(),
            invalid_frequency_popup: false,
            invalid_deviation_popup: false,
            invalid_dr_popup: false,
            is_hex: true,
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
        u64::from_str_radix(format!("{}{}{}", 
            format!("{:08b}", self.register_value.freq2).to_string(), 
            format!("{:08b}", self.register_value.freq1).to_string(), 
            format!("{:08b}", self.register_value.freq0))
            .as_str(), 2).expect("Invalid binary string")
            .to_string();
    }
    
    fn update_channel_number_from_parameter(&mut self) {
        self.register_value.channr = self.user_input_channel_number
    }
    
    fn update_tx_power_from_parameter(&mut self) {
        self.register_value.pa_table0 = 0x00;
        match self.user_input_tx_power {
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
    
    fn update_data_whitening_from_parameter(&mut self) {
        if self.is_whitened {self.register_value.pktctrl0 |= 0x40} else {self.register_value.pktctrl0 &= 0xBF};
    }
    
    fn update_manchester_from_parameter(&mut self) {
        if self.manchester_enabled {self.register_value.mdmcfg2 |= 0x08} else {self.register_value.mdmcfg2 &= 0xF7};
    }
    
    fn update_phase_transition_time_from_parameter(&mut self) {
        match self.user_input_phase_transition_time {
            0 => self.register_value.deviatn |= 0x00,
            1 => self.register_value.deviatn |= 0x01,
            2 => self.register_value.deviatn |= 0x02,
            3 => self.register_value.deviatn |= 0x03,
            4 => self.register_value.deviatn |= 0x04,
            5 => self.register_value.deviatn |= 0x05,
            6 => self.register_value.deviatn |= 0x06,
            7 => self.register_value.deviatn |= 0x07,
            _ => self.register_value.deviatn = self.register_value.deviatn,
        }
    }
    
    fn update_deviation_from_parameter(&mut self) {
        let intermediate_deviation_u64 = f64::floor(self.user_input_deviation.parse::<f64>().unwrap() / DEVIATION_FACTOR) as u64;
        let deviation_e = (intermediate_deviation_u64 / 8).checked_ilog2().unwrap() as u8;
        let deviation_m = ((intermediate_deviation_u64 / 2u64.pow(deviation_e as u32)) % 8) as u8;
        self.register_value.deviatn = deviation_e << 4 | deviation_m;
    }
    
    fn update_dr_from_parameter(&mut self) {
        self.register_value.mdmcfg4 &= 0xF0;
        let intermediate_dr_u64 = f64::floor(self.user_input_dr.parse::<f64>().unwrap() / DATA_RATE_FACTOR) as u64;
        let dr_e = (intermediate_dr_u64 / 256).checked_ilog2().unwrap() as u8;
        let dr_m = ((intermediate_dr_u64 / 2u64.pow(dr_e as u32)) % 256) as u8;
        self.register_value.mdmcfg4 |= dr_e;
        self.register_value.mdmcfg3 = dr_m;
    }

    // fn update_base_frequency_from_parameter(&mut self) {
    //     let intermediate_input_frequency_u64 = f64::floor(self.user_input_frequency.parse::<f64>().unwrap() * FREQUENCY_FACTOR) as u64; 
    //     self.register_value.freq0 = (intermediate_input_frequency_u64 & 0xFF) as u8;
    //     self.register_value.freq1 = ((intermediate_input_frequency_u64 >> 8) & 0xFF) as u8;
    //     self.register_value.freq2 = ((intermediate_input_frequency_u64 >> 16) & 0xFF) as u8;
    //     u64::from_str_radix(format!("{}{}{}", format!("{:08b}", self.register_value.freq2).to_string(), format!("{:08b}", self.register_value.freq1).to_string(), format!("{:08b}", self.register_value.freq0)).as_str(), 2).expect("Invalid binary string").to_string();
    // }

    fn print_concatenated_freq(&self) -> String {
        let intermediate_decimal_frequency = ((self.register_value.freq2 as u32) << 16) | ((self.register_value.freq1 as u32) << 8) | self.register_value.freq0 as u32;
        format!("{}", (intermediate_decimal_frequency as f64 / FREQUENCY_FACTOR))
    }
    
    fn print_deviation(&self) -> String {
        let register_deviatn_m = self.register_value.deviatn & 0x07;
        let register_deviatn_e = (self.register_value.deviatn & 0x70) >> 4;
        let intermediate_decimal_deviation = DEVIATION_FACTOR * ((8 + register_deviatn_m) as u64 * 2u64.pow(register_deviatn_e as u32)) as f64;
        intermediate_decimal_deviation.to_string()
    }
    
    fn print_dr(&self) -> String {
        let register_dr_e = self.register_value.mdmcfg4 & 0x0F;
        let register_dr_m = self.register_value.mdmcfg3;
        let intermediate_decimal_dr = DATA_RATE_FACTOR * ((256 + register_dr_m as u64) * 2u64.pow(register_dr_e as u32)) as f64;
        intermediate_decimal_dr.to_string()
    }

    fn frequency_input_is_out_of_bounds(&mut self) {
        if let Ok(value) = self.user_input_frequency.trim().parse::<f64>() {
            // Check if the value is out of bounds
            if value < BASE_FREQUENCY_MIN || value > BASE_FREQUENCY_MAX {
                self.invalid_frequency_popup = true; // Trigger the popup
            } else {
                self.update_base_frequency_from_parameter();
                self.invalid_frequency_popup = false;
            }
        } else {
            // Show popup for invalid input
            self.invalid_frequency_popup = true;
        } 
    }

    fn show_invalid_frequency_popup(&mut self, ctx: &egui::Context) {
        egui::Window::new("Invalid Frequency Input")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("The base frequency must be between {:?} and {:?}!", BASE_FREQUENCY_MIN, BASE_FREQUENCY_MAX));
                if ui.button("OK").clicked() {
                    self.invalid_frequency_popup = false; // Close the popup
                }
        });
    }

    fn deviation_input_is_out_of_bounds(&mut self) {
        if let Ok(value) = self.user_input_deviation.trim().parse::<f64>() {
            // Check if the value is out of bounds
            if value < DEVIATION_MIN || value > DEVATION_MAX {
                self.invalid_deviation_popup = true; // Trigger the popup
            } else {
                self.update_deviation_from_parameter();
                self.invalid_deviation_popup = false;
            }
        } else {
            // Show popup for invalid input
            self.invalid_deviation_popup = true;
        }
    }

    fn show_invalid_deviation_popup(&mut self, ctx: &egui::Context) {
        egui::Window::new("Invalid Deviation Input")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("The deviation must be between {:?} and {:?}!", DEVIATION_MIN, DEVATION_MAX));
                if ui.button("OK").clicked() {
                    self.invalid_deviation_popup = false; // Close the popup
                }
        });
    }

    fn dr_input_is_out_of_bounds(&mut self) {
        if let Ok(value) = self.user_input_dr.trim().parse::<f64>() {
            // Check if the value is out of bounds
            if value < DATA_RATE_MIN || value > DATA_RATE_MAX {
                self.invalid_dr_popup = true; // Trigger the popup
            } else {
                self.update_dr_from_parameter();
                self.invalid_dr_popup = false;
            }
        } else {
            // Show popup for invalid input
            self.invalid_dr_popup = true;
        }
    }

    fn show_invalid_dr_popup(&mut self, ctx: &egui::Context) {
        egui::Window::new("Invalid Data Rate Input")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui: &mut egui::Ui| {
                ui.label(format!("The data rate must be between {:?} and {:?}!", DATA_RATE_MIN, DATA_RATE_MAX));
                if ui.button("OK").clicked() {
                    self.invalid_dr_popup = false; // Close the popup
                }
        });
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
            egui::Grid::new("left_panels")
                .min_col_width(150.0)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Base Frequency");
                    ui.vertical(|ui| {
                        let frequency_text_box = ui.add(egui::TextEdit::singleline(&mut self.user_input_frequency).desired_width(68.0));
                        if frequency_text_box.lost_focus() {
                            self.frequency_input_is_out_of_bounds();
                        }
                        if self.invalid_frequency_popup {
                            self.show_invalid_frequency_popup(ctx);
                        }
                        ui.add(
                            egui::TextEdit::singleline(&mut self.print_concatenated_freq()).clip_text(true).desired_width(68.0)
                        );
                    });
                    ui.label("MHz");
                ui.end_row();

                ui.label("Channel Number");
                ui.horizontal(|ui| {
                    let channel_number_box = ui.add(egui::DragValue::new(&mut self.user_input_channel_number)
                        .speed(1.0)
                        .clamp_existing_to_range(true)
                        .range(0..=255));
                    if channel_number_box.changed() {
                        self.update_channel_number_from_parameter();
                    }
                });
                ui.label(self.register_value.channr.to_string());
                ui.end_row();

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
                ui.end_row();

                ui.label("Data Whitening");
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.is_whitened, "Data Whitening").clicked() {
                        self.update_data_whitening_from_parameter();
                    }
                });
                ui.label(self.register_value.pktctrl0.to_string());
                ui.end_row();

                ui.label("Manchester Enable");
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.manchester_enabled, "Manchester Enable").clicked() {
                        self.update_manchester_from_parameter();
                    }
                });
                ui.label(self.register_value.mdmcfg2.to_string());
                ui.end_row();
                
                ui.label("TX Power");
                ui.horizontal(|ui| {
                    egui::ComboBox::from_label(" ")
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
                    
                });
                ui.label(self.register_value.pa_table0.to_string());
                ui.end_row();

                if self.register_value.mdmcfg2 & 0x70 == 0x70 {
                    ui.label("Phase Transition Time");
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_label("Phase Transition Time")
                            .selected_text(&self.user_input_phase_transition_time.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.user_input_phase_transition_time, 0, "0");
                                ui.selectable_value(&mut self.user_input_phase_transition_time, 1, "1");
                                ui.selectable_value(&mut self.user_input_phase_transition_time, 2, "2");
                                ui.selectable_value(&mut self.user_input_phase_transition_time, 3, "3");
                                ui.selectable_value(&mut self.user_input_phase_transition_time, 4, "4");
                                ui.selectable_value(&mut self.user_input_phase_transition_time, 5, "5");
                                ui.selectable_value(&mut self.user_input_phase_transition_time, 6, "6");
                                ui.selectable_value(&mut self.user_input_phase_transition_time, 7, "7");
                        });
                        self.update_phase_transition_time_from_parameter();
                    });
                } else {
                    ui.label("Deviation");
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let deviation_text_box = ui.add(egui::TextEdit::singleline(&mut self.user_input_deviation).desired_width(68.0));
                            if deviation_text_box.lost_focus() {
                                self.deviation_input_is_out_of_bounds();
                            }
                            if self.invalid_deviation_popup {
                                self.show_invalid_deviation_popup(ctx);
                            }
                        });
                        ui.add(
                            egui::TextEdit::singleline(&mut self.print_deviation()).clip_text(true).desired_width(68.0)
                        ); 
                    });
                    ui.label(self.register_value.deviatn.to_string());
                    // ui.label(format!("register_deviatn_m = {:08b} ---> :{:?}", self.register_value.deviatn, self.register_value.deviatn));
                }
                ui.end_row();

                ui.label("Data Rate");
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let dr_text_box = ui.add(egui::TextEdit::singleline(&mut self.user_input_dr).desired_width(68.0));
                        if dr_text_box.lost_focus() {
                            self.dr_input_is_out_of_bounds();
                        }
                        if self.invalid_dr_popup {
                            self.show_invalid_dr_popup(ctx);
                        }
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut self.print_dr()).clip_text(true).desired_width(68.0)
                    ); 
                });
                ui.label((self.register_value.mdmcfg4 & 0x0F).to_string());
                ui.label(self.register_value.mdmcfg3.to_string());
                ui.label("kBaud");
                // ui.label(format!("register_mdmcfg4 = {:08b}", self.register_value.mdmcfg4));
                // ui.label(format!("register_dr_m = {:08b}", self.register_value.mdmcfg3));
                ui.end_row();
                
                ui.horizontal(|ui| {
    
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
            egui::Grid::new("right_panel")
                .striped(true)
                .show(ui, |ui| {
                    let mut toggle = toggle_ui(ui, &mut self.is_hex);
                });
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

fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}
