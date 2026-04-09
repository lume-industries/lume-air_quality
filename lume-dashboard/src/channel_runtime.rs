#![allow(dead_code)]

use serde::de::DeserializeOwned;

pub const CHANNEL_BUF_BYTES: usize = 64 * 1024;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "vzglyd_host")]
unsafe extern "C" {
    fn channel_poll(buf_ptr: *mut u8, buf_len: i32) -> i32;
    #[link_name = "log_info"]
    fn host_log_info(ptr: *const u8, len: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
pub fn poll_bytes(buf: &mut [u8]) -> Option<usize> {
    let status = unsafe { channel_poll(buf.as_mut_ptr(), buf.len() as i32) };
    if status <= 0 {
        None
    } else {
        Some(status as usize)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn poll_bytes(_buf: &mut [u8]) -> Option<usize> {
    None
}

pub fn poll_json<T: DeserializeOwned>(buf: &mut [u8]) -> Option<T> {
    let len = poll_bytes(buf)?;
    serde_json::from_slice::<T>(&buf[..len]).ok()
}

pub fn info_log(message: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        let _ = host_log_info(message.as_ptr(), message.len() as i32);
    }

    #[cfg(not(target_arch = "wasm32"))]
    let _ = message;
}
