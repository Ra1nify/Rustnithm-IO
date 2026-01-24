use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use windows::Win32::Foundation::{S_OK, S_FALSE}; // 删除了 HANDLE
use windows::Win32::System::Memory::*;

mod config;
use config::{RustnithmSharedMemory, SHMEM_NAME};

// 使用原子指针安全存储共享内存地址
static SHMEM_PTR: AtomicPtr<RustnithmSharedMemory> = AtomicPtr::new(ptr::null_mut());

// --- DLL 生命周期接口 ---

#[no_mangle]
pub unsafe extern "system" fn aime_io_get_api_version() -> u32 {
    0x0100
}

#[no_mangle]
pub unsafe extern "system" fn aime_io_init() -> i32 {
    // 打开已经由服务端创建好的共享内存映射
    let handle = OpenFileMappingW(
        FILE_MAP_ALL_ACCESS.0,
        false,
        SHMEM_NAME,
    );

    match handle {
        Ok(h) => {
            let view = MapViewOfFile(h, FILE_MAP_ALL_ACCESS, 0, 0, 0);
            // 修正点 1: 在新版 windows-rs 中，使用 Value.is_null() 检查
            if !view.Value.is_null() {
                // 修正点 2: 使用 .Value 获取指针地址
                SHMEM_PTR.store(view.Value as *mut RustnithmSharedMemory, Ordering::SeqCst);
                return S_OK.0;
            }
            S_FALSE.0
        }
        Err(_) => S_FALSE.0,
    }
}

// --- NFC 核心逻辑接口 ---

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
        
        // 信号复位
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