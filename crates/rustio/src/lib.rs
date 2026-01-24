use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::ptr::{addr_of, addr_of_mut};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Power::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use shared::{ChunIoConfig, ChunIoIpcMemoryInfo, chuni_io_config_load};

const CHUNI_IO_OPBTN_TEST: u8 = 0x01;
const CHUNI_IO_OPBTN_SERVICE: u8 = 0x02;

struct ChunIoState {
    coin: bool,
    coins: u16,
    hand_pos: u8,
    slider_stop_flag: bool,
    cfg: ChunIoConfig,
    file_mapping_handle: isize,
    file_mapping: usize,
}

unsafe impl Send for ChunIoState {}
unsafe impl Sync for ChunIoState {}

static mut CHUNI_IO_STATE: Option<Arc<Mutex<ChunIoState>>> = None;
static mut SLIDER_THREAD_HANDLE: Option<thread::JoinHandle<()>> = None;

unsafe fn chuni_io_init_shared_memory(state: &mut ChunIoState) {
    if state.file_mapping != 0 {
        return;
    }

    let mapping_name = w!("Local\\RustnithmSharedMemory");
    
    let handle_result = windows::Win32::System::Memory::CreateFileMappingW(
        INVALID_HANDLE_VALUE,
        None,
        PAGE_READWRITE,
        0,
        std::mem::size_of::<ChunIoIpcMemoryInfo>() as u32,
        mapping_name,
    );

    match handle_result {
        Ok(h) => {
            state.file_mapping_handle = h.0 as isize;
            
            let view = MapViewOfFile(
                h,
                FILE_MAP_ALL_ACCESS,
                0,
                0,
                std::mem::size_of::<ChunIoIpcMemoryInfo>(),
            );

            let memory = view.Value as *mut ChunIoIpcMemoryInfo;
            if !memory.is_null() {
                std::ptr::write_bytes(memory, 0, 1);
                state.file_mapping = memory as usize;
                
                let _ = SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED);
            }
        }
        Err(_) => {}
    }
}

#[no_mangle]
pub extern "C" fn chuni_io_get_api_version() -> u16 {
    0x0102
}

#[no_mangle]
pub extern "C" fn chuni_io_jvs_init() -> HRESULT {
    unsafe {
        let mut state = ChunIoState {
            coin: false,
            coins: 0,
            hand_pos: 0,
            slider_stop_flag: false,
            cfg: ChunIoConfig::default(),
            file_mapping_handle: 0,
            file_mapping: 0,
        };

        chuni_io_config_load(&mut state.cfg, w!(".\\segatools.ini"));
        chuni_io_init_shared_memory(&mut state);

        CHUNI_IO_STATE = Some(Arc::new(Mutex::new(state)));
        
        S_OK
    }
}

#[no_mangle]
pub extern "C" fn chuni_io_jvs_read_coin_counter(out: *mut u16) {
    unsafe {
        if out.is_null() {
            return;
        }

        let state_ptr = addr_of!(CHUNI_IO_STATE);
        if let Some(state_arc) = (*state_ptr).as_ref() {
            let mut state = state_arc.lock().unwrap();
            
            if state.file_mapping != 0 {
                let mapping = state.file_mapping as *mut ChunIoIpcMemoryInfo;
                if (*mapping).coin_insertion != 0 {
                    state.coins += 1;
                    (*mapping).coin_insertion = 0;
                }
            } else {
                let key_state = GetAsyncKeyState(state.cfg.vk_coin as i32);
                if key_state != 0 {
                    if !state.coin {
                        state.coin = true;
                        state.coins += 1;
                    }
                } else {
                    state.coin = false;
                }
            }

            *out = state.coins;
        }
    }
}

#[no_mangle]
pub extern "C" fn chuni_io_jvs_poll(opbtn: *mut u8, beams: *mut u8) {
    unsafe {
        let state_ptr = addr_of!(CHUNI_IO_STATE);
        if let Some(state_arc) = (*state_ptr).as_ref() {
            let mut state = state_arc.lock().unwrap();

            let test_pressed = if state.file_mapping != 0 {
                let mapping = state.file_mapping as *mut ChunIoIpcMemoryInfo;
                (*mapping).test_btn != 0
            } else {
                false
            } || GetAsyncKeyState(state.cfg.vk_test as i32) != 0;

            if test_pressed {
                *opbtn |= CHUNI_IO_OPBTN_TEST;
            }

            let service_pressed = if state.file_mapping != 0 {
                let mapping = state.file_mapping as *mut ChunIoIpcMemoryInfo;
                (*mapping).service_btn != 0
            } else {
                false
            } || GetAsyncKeyState(state.cfg.vk_service as i32) != 0;

            if service_pressed {
                *opbtn |= CHUNI_IO_OPBTN_SERVICE;
            }

            if state.cfg.vk_ir_emu != 0 {
                if GetAsyncKeyState(state.cfg.vk_ir_emu as i32) != 0 {
                    if state.hand_pos < 6 {
                        state.hand_pos += 1;
                    }
                } else {
                    if state.hand_pos > 0 {
                        state.hand_pos -= 1;
                    }
                }

                for i in 0..6 {
                    if state.hand_pos > i {
                        *beams |= 1 << i;
                    }
                }
            } else {
                for i in 0..6 {
                    if GetAsyncKeyState(state.cfg.vk_ir[i] as i32) & 0x8000u16 as i16 != 0 {
                        *beams |= 1 << i;
                    } else {
                        *beams &= !(1 << i);
                    }
                }
            }

            if state.file_mapping != 0 {
                let mapping = state.file_mapping as *mut ChunIoIpcMemoryInfo;
                for i in 0..6 {
                    if (*mapping).air_io_status[i] != 0 {
                        *beams |= 1 << i;
                    }
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn chuni_io_slider_init() -> HRESULT {
    S_OK
}

type SliderCallback = extern "C" fn(*const u8);

#[no_mangle]
pub extern "C" fn chuni_io_slider_start(callback: SliderCallback) {
    unsafe {
        let thread_ptr = addr_of!(SLIDER_THREAD_HANDLE);
        if (*thread_ptr).is_some() {
            return;
        }

        let state_ptr = addr_of!(CHUNI_IO_STATE);
        if let Some(state_arc) = (*state_ptr).as_ref() {
            let state_clone = Arc::clone(state_arc);
            
            let handle = thread::spawn(move || {
                chuni_io_slider_thread_proc(callback, state_clone);
            });

            let thread_mut_ptr = addr_of_mut!(SLIDER_THREAD_HANDLE);
            *thread_mut_ptr = Some(handle);
        }
    }
}

#[no_mangle]
pub extern "C" fn chuni_io_slider_stop() {
    unsafe {
        let thread_mut_ptr = addr_of_mut!(SLIDER_THREAD_HANDLE);
        if let Some(handle) = (*thread_mut_ptr).take() {
            let state_ptr = addr_of!(CHUNI_IO_STATE);
            if let Some(state_arc) = (*state_ptr).as_ref() {
                {
                    let mut state = state_arc.lock().unwrap();
                    state.slider_stop_flag = true;
                }
                
                let _ = handle.join();
                
                {
                    let mut state = state_arc.lock().unwrap();
                    state.slider_stop_flag = false;
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn chuni_io_slider_set_leds(rgb: *const u8) {
    unsafe {
        let state_ptr = addr_of!(CHUNI_IO_STATE);
        if let Some(state_arc) = (*state_ptr).as_ref() {
            let state = state_arc.lock().unwrap();
            
            if state.file_mapping != 0 {
                let mapping = state.file_mapping as *mut ChunIoIpcMemoryInfo;
                std::ptr::copy_nonoverlapping(
                    rgb,
                    (*mapping).led_rgb_data.as_mut_ptr(),
                    96,
                );
            }
        }
    }
}

fn chuni_io_slider_thread_proc(callback: SliderCallback, state_arc: Arc<Mutex<ChunIoState>>) {
    let mut pressure = [0u8; 32];

    loop {
        {
            let state = state_arc.lock().unwrap();
            
            if state.slider_stop_flag {
                break;
            }

            unsafe {
                if state.file_mapping != 0 {
                    let mapping = state.file_mapping as *mut ChunIoIpcMemoryInfo;
                    std::ptr::copy_nonoverlapping(
                        (*mapping).slider_io_status.as_ptr(),
                        pressure.as_mut_ptr(),
                        32,
                    );
                    
                    for i in 0..32 {
                        if pressure[i] != 0 {
                            pressure[i] = 255;
                        }
                    }
                }
            }
        }

        callback(pressure.as_ptr());
        thread::sleep(Duration::from_millis(1));
    }
}

#[no_mangle]
pub extern "C" fn chuni_io_led_init() -> HRESULT {
    S_OK
}

#[no_mangle]
pub extern "C" fn chuni_io_led_set_colors(_board: u8, _rgb: *mut u8) {
}