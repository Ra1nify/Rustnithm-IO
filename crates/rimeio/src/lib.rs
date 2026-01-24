use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use windows::Win32::Foundation::{S_OK, S_FALSE};
use windows::Win32::System::Memory::*;

use shared::ChunIoIpcMemoryInfo as RustnithmSharedMemory;

static SHMEM_PTR: AtomicPtr<RustnithmSharedMemory> = AtomicPtr::new(ptr::null_mut());

const SHMEM_NAME: windows::core::PCWSTR = windows::core::w!("RustnithmSharedMemory");


#[no_mangle]
pub unsafe extern "system" fn aime_io_get_api_version() -> u32 {
    0x0100
}

#[no_mangle]
pub unsafe extern "system" fn aime_io_init() -> i32 {
    let handle = OpenFileMappingW(
        FILE_MAP_ALL_ACCESS.0,
        false,
        SHMEM_NAME,
    );

    match handle {
        Ok(h) => {
            let view = MapViewOfFile(h, FILE_MAP_ALL_ACCESS, 0, 0, 0);
            if !view.Value.is_null() {
                SHMEM_PTR.store(view.Value as *mut RustnithmSharedMemory, Ordering::SeqCst);
                return S_OK.0;
            }
            S_FALSE.0
        }
        Err(_) => S_FALSE.0,
    }
}


#[no_mangle]
pub unsafe extern "system" fn aime_io_nfc_poll(_unit_no: u8) -> i32 {
    let ptr = SHMEM_PTR.load(Ordering::SeqCst);
    if ptr.is_null() {
        return S_FALSE.0;
    }

    if (*ptr).remote_card_read != 0 {
        return S_OK.0;
    }

    S_FALSE.0
}

#[no_mangle]
pub unsafe extern "system" fn aime_io_nfc_get_aime_id(
    _unit_no: u8,
    luid: *mut u8,
    luid_size: usize,
) -> i32 {
    let ptr = SHMEM_PTR.load(Ordering::SeqCst);
    if ptr.is_null() || luid.is_null() {
        return S_FALSE.0;
    }

    if (*ptr).remote_card_read != 0 && luid_size == 10 {
        ptr::copy_nonoverlapping((*ptr).remote_card_id.as_ptr(), luid, 10);
        
        (*ptr).remote_card_read = 0;
        
        return S_OK.0;
    }

    S_FALSE.0
}

#[no_mangle]
pub unsafe extern "system" fn aime_io_nfc_get_felica_id(
    _unit_no: u8,
    _id: *mut u8,
    _id_size: usize,
) -> i32 {
    S_FALSE.0
}

#[no_mangle]
pub unsafe extern "system" fn aime_io_led_set_color(_unit_no: u8, _r: u8, _g: u8, _b: u8) -> i32 {
    S_OK.0
}