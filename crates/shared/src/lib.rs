use windows::Win32::System::WindowsProgramming::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::core::{PCWSTR, w};


#[repr(C)]
pub struct ChunIoIpcMemoryInfo {
    pub air_io_status: [u8; 6],      // 0-5
    pub slider_io_status: [u8; 32],   // 6-37
    pub led_rgb_data: [u8; 96],       // 38-133
    pub test_btn: u8,                // 134
    pub service_btn: u8,             // 135
    pub coin_insertion: u8,          // 136
    pub card_read: u8,               // 137
    pub remote_card_read: u8,        // 138
    pub remote_card_type: u8,        // 139
    pub remote_card_id: [u8; 10],    // 140-149
}

const CHUNI_IO_DEFAULT_CELLS: [u8; 32] = [
    b'L', b'L', b'L', b'L', b'K', b'K', b'K', b'K', b'J', b'J', b'J', b'J', b'H', b'H', b'H', b'H',
    b'G', b'G', b'G', b'G', b'F', b'F', b'F', b'F', b'D', b'D', b'D', b'D', b'S', b'S', b'S', b'S',
];

const CHUNI_IO_DEFAULT_IR: [u8; 6] = [b'4', b'5', b'6', b'7', b'8', b'9'];

#[derive(Clone)]
pub struct ChunIoConfig {
    pub vk_test: u8,
    pub vk_service: u8,
    pub vk_coin: u8,
    pub vk_ir_emu: u8,
    pub vk_ir: [u8; 6],
    pub vk_cell: [u8; 32],
    pub cab_led_output_pipe: bool,
    pub cab_led_output_serial: bool,
    pub controller_led_output_pipe: bool,
    pub controller_led_output_serial: bool,
    pub controller_led_output_openithm: bool,
    pub led_serial_port: [u16; 12],
    pub led_serial_baud: i32,
}

impl Default for ChunIoConfig {
    fn default() -> Self {
        Self {
            vk_test: VK_F1.0 as u8,
            vk_service: VK_F2.0 as u8,
            vk_coin: VK_F3.0 as u8,
            vk_ir_emu: VK_SPACE.0 as u8,
            vk_ir: CHUNI_IO_DEFAULT_IR,
            vk_cell: CHUNI_IO_DEFAULT_CELLS,
            cab_led_output_pipe: true,
            cab_led_output_serial: false,
            controller_led_output_pipe: true,
            controller_led_output_serial: false,
            controller_led_output_openithm: false,
            led_serial_port: [0; 12],
            led_serial_baud: 921600,
        }
    }
}

pub unsafe fn chuni_io_config_load(cfg: &mut ChunIoConfig, filename: PCWSTR) {
    let io3_section = w!("io3");
    let ir_section = w!("ir");
    let slider_section = w!("slider");
    let led_section = w!("led");

    cfg.vk_test = GetPrivateProfileIntW(io3_section, w!("test"), VK_F1.0 as i32, filename) as u8;
    cfg.vk_service =
        GetPrivateProfileIntW(io3_section, w!("service"), VK_F2.0 as i32, filename) as u8;
    cfg.vk_coin = GetPrivateProfileIntW(io3_section, w!("coin"), VK_F3.0 as i32, filename) as u8;
    cfg.vk_ir_emu = GetPrivateProfileIntW(io3_section, w!("ir"), VK_SPACE.0 as i32, filename) as u8;

    for i in 0..6 {
        let key_name = format!("ir{}\0", i + 1);
        let key_wide: Vec<u16> = key_name.encode_utf16().collect();
        cfg.vk_ir[i] = GetPrivateProfileIntW(
            ir_section,
            PCWSTR::from_raw(key_wide.as_ptr()),
            CHUNI_IO_DEFAULT_IR[i] as i32,
            filename,
        ) as u8;
    }

    for i in 0..32 {
        let key_name = format!("cell{}\0", i + 1);
        let key_wide: Vec<u16> = key_name.encode_utf16().collect();
        cfg.vk_cell[i] = GetPrivateProfileIntW(
            slider_section,
            PCWSTR::from_raw(key_wide.as_ptr()),
            CHUNI_IO_DEFAULT_CELLS[i] as i32,
            filename,
        ) as u8;
    }

    cfg.cab_led_output_pipe =
        GetPrivateProfileIntW(led_section, w!("cabLedOutputPipe"), 1, filename) != 0;
    cfg.cab_led_output_serial =
        GetPrivateProfileIntW(led_section, w!("cabLedOutputSerial"), 0, filename) != 0;
    cfg.controller_led_output_pipe =
        GetPrivateProfileIntW(led_section, w!("controllerLedOutputPipe"), 1, filename) != 0;
    cfg.controller_led_output_serial =
        GetPrivateProfileIntW(led_section, w!("controllerLedOutputSerial"), 0, filename) != 0;
    cfg.controller_led_output_openithm =
        GetPrivateProfileIntW(led_section, w!("controllerLedOutputOpeNITHM"), 0, filename) != 0;
    cfg.led_serial_baud = GetPrivateProfileIntW(led_section, w!("serialBaud"), 921600, filename);

    let mut port_input = [0u16; 6];
    let default_port = w!("COM5");
    GetPrivateProfileStringW(
        led_section,
        w!("serialPort"),
        default_port,
        Some(&mut port_input),
        filename,
    );

    let prefix: Vec<u16> = "\\\\.\\".encode_utf16().collect();
    for (i, &c) in prefix.iter().enumerate() {
        if i < 12 {
            cfg.led_serial_port[i] = c;
        }
    }

    let mut offset = prefix.len();
    for &c in &port_input {
        if c == 0 || offset >= 11 {
            break;
        }
        cfg.led_serial_port[offset] = c;
        offset += 1;
    }
    cfg.led_serial_port[11] = 0;
}