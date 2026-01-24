// src/config.rs
use windows::core::w;
use windows::core::PCWSTR;

// 必须与 aimeio.c 中的 struct aime_io_ipc_memory_info 完全一致
// 使用 #[repr(C)] 确保 Rust 不会为了优化而重排字段顺序
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RustnithmSharedMemory {
    pub air_io_status: [u8; 6],      // 空气传感器 (6字节)
    pub slider_io_status: [u8; 32],   // 滑条状态 (32字节)
    pub led_rgb_data: [u8; 96],       // 灯光数据 (32 * 3 字节)
    pub test_btn: u8,                // Test 键状态
    pub service_btn: u8,             // Service 键状态
    pub coin_insertion: u8,          // 投币状态
    pub card_read: u8,               // 物理读卡器信号
    pub remote_card_read: u8,        // 【关键】手机触发标志位 (1为有卡，0为无卡)
    pub remote_card_type: u8,        // 卡片类型 (Aime/FeliCa)
    pub remote_card_id: [u8; 10],    // 【关键】10字节十六进制卡号 (LUID)
}

// 定义共享内存的名称，必须与服务端 server.rs 里的名称完全一致
pub const SHMEM_NAME: PCWSTR = w!("RustnithmSharedMemory");