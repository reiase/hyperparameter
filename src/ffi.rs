use std::ffi::{CStr, CString};

use super::storage::GetOrElse;
use super::storage::Storage;

#[no_mangle]
pub unsafe extern "C" fn hyper_create_storage() -> *mut Storage {
    let s = Box::new(Storage::new());
    Box::leak(s)
}

#[no_mangle]
pub unsafe extern "C" fn hyper_destory_storage(this: *mut Storage) {
    drop(Box::from_raw(this));
}

#[no_mangle]
pub unsafe extern "C" fn storage_enter(this: *mut Storage) {
    (*this).enter()
}

#[no_mangle]
pub unsafe extern "C" fn storage_exit(this: *mut Storage) {
    (*this).exit();
}

#[no_mangle]
pub unsafe extern "C" fn storage_hget_or_i64(this: *mut Storage, hkey: u64, def: i64) -> i64 {
    (*this).get_or_else(hkey, def)
}

#[no_mangle]
pub unsafe extern "C" fn storage_hget_or_f64(this: *mut Storage, hkey: u64, def: f64) -> f64 {
    (*this).get_or_else(hkey, def)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn storage_hget_or_str(
    this: *mut Storage,
    hkey: u64,
    def: *mut i8,
) -> *mut i8 {
    let raw = CStr::from_ptr(def).to_str().unwrap().to_string();
    let s = (*this).get_or_else(hkey, raw);
    CString::new(s).unwrap().into_raw()
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn storage_hget_or_str(
    this: *mut Storage,
    hkey: u64,
    def: *mut i8,
) -> *mut u8 {
    let raw = CStr::from_ptr(def as *const u8)
        .to_str()
        .unwrap()
        .to_string();
    let s = (*this).get_or_else(hkey, raw);
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn storage_hget_or_bool(this: *mut Storage, hkey: u64, def: bool) -> bool {
    (*this).get_or_else(hkey, def)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn storage_put_i64(this: *mut Storage, key: *const i8, val: i64) {
    let key = CStr::from_ptr(key);
    (*this).put(key.to_string_lossy(), val)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn storage_put_i64(this: *mut Storage, key: *const i8, val: i64) {
    let key = CStr::from_ptr(key as *const u8);
    (*this).put(key.to_string_lossy(), val)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn storage_put_f64(this: *mut Storage, key: *const i8, val: f64) {
    let key = CStr::from_ptr(key);
    (*this).put(key.to_string_lossy(), val)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn storage_put_f64(this: *mut Storage, key: *const i8, val: f64) {
    let key = CStr::from_ptr(key as *const u8);
    (*this).put(key.to_string_lossy(), val)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn storage_put_bool(this: *mut Storage, key: *const i8, val: bool) {
    let key = CStr::from_ptr(key);
    (*this).put(key.to_string_lossy(), val)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn storage_put_bool(this: *mut Storage, key: *const i8, val: bool) {
    let key = CStr::from_ptr(key as *const u8);
    (*this).put(key.to_string_lossy(), val)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn storage_put_str(this: *mut Storage, key: *const i8, val: *const i8) {
    let key = CStr::from_ptr(key);
    let val = CStr::from_ptr(val);
    (*this).put(key.to_string_lossy(), val.to_string_lossy().to_string())
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn storage_put_str(this: *mut Storage, key: *const i8, val: *const i8) {
    let key = CStr::from_ptr(key as *const u8);
    let val = CStr::from_ptr(val as *const u8);
    (*this).put(key.to_string_lossy(), val.to_string_lossy().to_string())
}
