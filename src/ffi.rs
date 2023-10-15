use std::ffi::{CStr, CString};

use super::api::ParamScope;
use super::api::ParamScopeOps;

#[no_mangle]
pub unsafe extern "C" fn param_scope_create() -> *mut ParamScope {
    let ps = Box::<ParamScope>::default();
    Box::leak(ps)
}

#[no_mangle]
pub unsafe extern "C" fn param_scope_destroy(this: *mut ParamScope) {
    drop(Box::from_raw(this));
}

#[no_mangle]
pub unsafe extern "C" fn param_scope_enter(this: *mut ParamScope) {
    (*this).enter()
}

#[no_mangle]
pub unsafe extern "C" fn param_scope_exit(this: *mut ParamScope) {
    (*this).exit();
}

#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_i64(this: *mut ParamScope, hkey: u64, def: i64) -> i64 {
    (*this).get_or_else(hkey, def)
}

#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_or_f64(
    this: *mut ParamScope,
    hkey: u64,
    def: f64,
) -> f64 {
    (*this).get_or_else(hkey, def)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_or_str(
    this: *mut ParamScope,
    hkey: u64,
    def: *mut i8,
) -> *mut i8 {
    let raw = CStr::from_ptr(def).to_str().unwrap().to_string();
    let s = (*this).get_or_else(hkey, raw);
    CString::new(s).unwrap().into_raw()
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_or_str(
    this: *mut ParamScope,
    hkey: u64,
    def: *mut i8,
) -> *mut i8 {
    let raw = CStr::from_ptr(def as *const i8)
        .to_str()
        .unwrap()
        .to_string();
    let s = (*this).get_or_else(hkey, raw);
    CString::new(s).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_or_bool(
    this: *mut ParamScope,
    hkey: u64,
    def: bool,
) -> bool {
    (*this).get_or_else(hkey, def)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_i64(this: *mut ParamScope, key: *const i8, val: i64) {
    let key = CStr::from_ptr(key);
    (*this).put(key.to_string_lossy().to_string(), val)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_i64(this: *mut ParamScope, key: *const i8, val: i64) {
    let key = CStr::from_ptr(key as *const i8);
    (*this).put(key.to_string_lossy().to_string(), val)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_f64(this: *mut ParamScope, key: *const i8, val: f64) {
    let key = CStr::from_ptr(key);
    (*this).put(key.to_string_lossy().to_string(), val)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_f64(this: *mut ParamScope, key: *const i8, val: f64) {
    let key = CStr::from_ptr(key as *const i8);
    (*this).put(key.to_string_lossy().to_string(), val)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_bool(this: *mut ParamScope, key: *const i8, val: bool) {
    let key = CStr::from_ptr(key);
    (*this).put(key.to_string_lossy().to_string(), val)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_bool(this: *mut ParamScope, key: *const i8, val: bool) {
    let key = CStr::from_ptr(key as *const i8);
    (*this).put(key.to_string_lossy().to_string(), val)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_str(
    this: *mut ParamScope,
    key: *const i8,
    val: *const i8,
) {
    let key = CStr::from_ptr(key);
    let val = CStr::from_ptr(val);
    (*this).put(
        key.to_string_lossy().to_string(),
        val.to_string_lossy().to_string(),
    )
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_str(
    this: *mut ParamScope,
    key: *const i8,
    val: *const i8,
) {
    let key = CStr::from_ptr(key as *const i8);
    let val = CStr::from_ptr(val as *const i8);
    (*this).put(
        key.to_string_lossy().to_string(),
        val.to_string_lossy().to_string(),
    )
}
