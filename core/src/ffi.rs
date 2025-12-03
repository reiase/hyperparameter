use std::ffi::{CStr, CString};

use super::api::ParamScope;
use super::api::ParamScopeOps;

/// Creates a new ParamScope object and returns a pointer to it.
/// 
/// # Safety
/// The returned pointer must be freed using `param_scope_destroy`.
#[no_mangle]
pub unsafe extern "C" fn param_scope_create() -> *mut ParamScope {
    let ps = Box::<ParamScope>::default();
    Box::leak(ps)
}

/// Destroys the ParamScope object at the given address.
/// 
/// # Safety
/// - `this` must be a valid pointer returned by `param_scope_create`
/// - `this` must not be used after this function is called
/// - This function is safe to call with a null pointer (no-op)
#[no_mangle]
pub unsafe extern "C" fn param_scope_destroy(this: *mut ParamScope) {
    if this.is_null() {
        return;
    }
    drop(Box::from_raw(this));
}

/// Enters the given ParamScope object.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - The ParamScope must have been created by `param_scope_create`
/// - If `this` is null, this function does nothing
#[no_mangle]
pub unsafe extern "C" fn param_scope_enter(this: *mut ParamScope) {
    if this.is_null() {
        return;
    }
    (*this).enter()
}

/// Exits the given ParamScope object.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - The ParamScope must have been created by `param_scope_create`
/// - If `this` is null, this function does nothing
#[no_mangle]
pub unsafe extern "C" fn param_scope_exit(this: *mut ParamScope) {
    if this.is_null() {
        return;
    }
    (*this).exit();
}

/// Gets an integer value from the given ParamScope object by hashed key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - If `this` is null, returns `def`
/// 
/// # Returns
/// The parameter value if found, otherwise `def`
#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_i64(this: *mut ParamScope, hkey: u64, def: i64) -> i64 {
    if this.is_null() {
        return def;
    }
    (*this).get_or_else(hkey, def)
}

/// Gets a float value from the given ParamScope object by hashed key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - If `this` is null, returns `def`
/// 
/// # Returns
/// The parameter value if found, otherwise `def`
#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_or_f64(
    this: *mut ParamScope,
    hkey: u64,
    def: f64,
) -> f64 {
    if this.is_null() {
        return def;
    }
    (*this).get_or_else(hkey, def)
}

/// Frees a string pointer returned by `param_scope_hget_or_str`.
/// 
/// # Safety
/// - `ptr` must be a pointer returned by `param_scope_hget_or_str`, or null
/// - `ptr` must not be used after this function is called
/// - This function is safe to call with a null pointer (no-op)
/// - Do not call this function more than once for the same pointer
#[no_mangle]
pub unsafe extern "C" fn param_scope_free_str(ptr: *mut i8) {
    if ptr.is_null() {
        return;
    }
    // Reconstruct CString to properly free the memory
    let _ = CString::from_raw(ptr);
}

/// Gets a string value from the given ParamScope object by hashed key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `def` must be a valid, null-terminated C string pointer, or null
/// - If `this` is null, returns a copy of `def` (or empty string if `def` is null)
/// - The returned pointer must be freed by the caller using `param_scope_free_str()`
/// 
/// # Returns
/// A newly allocated C string containing the parameter value, or a copy of `def` if not found.
/// Returns null if memory allocation fails or if the string contains null bytes.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_or_str(
    this: *mut ParamScope,
    hkey: u64,
    def: *mut i8,
) -> *mut i8 {
    // Handle null default string
    let default_str = if def.is_null() {
        String::new()
    } else {
        match CStr::from_ptr(def).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(), // Invalid UTF-8, use empty string
        }
    };

    // Handle null ParamScope
    let result = if this.is_null() {
        default_str
    } else {
        (*this).get_or_else(hkey, default_str)
    };

    // Convert to CString, handle allocation failure
    match CString::new(result) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => std::ptr::null_mut(), // Allocation failed or contains null bytes
    }
}

/// Gets a string value from the given ParamScope object by hashed key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `def` must be a valid, null-terminated C string pointer, or null
/// - If `this` is null, returns a copy of `def` (or empty string if `def` is null)
/// - The returned pointer must be freed by the caller using `param_scope_free_str()`
/// 
/// # Returns
/// A newly allocated C string containing the parameter value, or a copy of `def` if not found.
/// Returns null if memory allocation fails or if the string contains null bytes.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_or_str(
    this: *mut ParamScope,
    hkey: u64,
    def: *mut i8,
) -> *mut i8 {
    // Handle null default string
    let default_str = if def.is_null() {
        String::new()
    } else {
        match CStr::from_ptr(def as *const i8).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => String::new(), // Invalid UTF-8, use empty string
        }
    };

    // Handle null ParamScope
    let result = if this.is_null() {
        default_str
    } else {
        (*this).get_or_else(hkey, default_str)
    };

    // Convert to CString, handle allocation failure
    match CString::new(result) {
        Ok(cstr) => cstr.into_raw(),
        Err(_) => std::ptr::null_mut(), // Allocation failed or contains null bytes
    }
}

/// Gets a boolean value from the given ParamScope object by hashed key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - If `this` is null, returns `def`
/// 
/// # Returns
/// The parameter value if found, otherwise `def`
#[no_mangle]
pub unsafe extern "C" fn param_scope_hget_or_bool(
    this: *mut ParamScope,
    hkey: u64,
    def: bool,
) -> bool {
    if this.is_null() {
        return def;
    }
    (*this).get_or_else(hkey, def)
}

/// Sets an integer value in the given ParamScope object by string key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `key` must be a valid, null-terminated C string pointer
/// - If `this` or `key` is null, this function does nothing
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_i64(this: *mut ParamScope, key: *const i8, val: i64) {
    if this.is_null() || key.is_null() {
        return;
    }
    if let Ok(key_str) = CStr::from_ptr(key).to_str() {
        (*this).put(key_str.to_string(), val);
    }
}

/// Sets an integer value in the given ParamScope object.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `key` must be a valid, null-terminated C string pointer
/// - If `this` or `key` is null, this function does nothing
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_i64(this: *mut ParamScope, key: *const i8, val: i64) {
    if this.is_null() || key.is_null() {
        return;
    }
    if let Ok(key_str) = CStr::from_ptr(key as *const i8).to_str() {
        (*this).put(key_str.to_string(), val);
    }
}

/// Sets a float value in the given ParamScope object by string key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `key` must be a valid, null-terminated C string pointer
/// - If `this` or `key` is null, this function does nothing
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_f64(this: *mut ParamScope, key: *const i8, val: f64) {
    if this.is_null() || key.is_null() {
        return;
    }
    if let Ok(key_str) = CStr::from_ptr(key).to_str() {
        (*this).put(key_str.to_string(), val);
    }
}

/// Sets a float value in the given ParamScope object by string key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `key` must be a valid, null-terminated C string pointer
/// - If `this` or `key` is null, this function does nothing
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_f64(this: *mut ParamScope, key: *const i8, val: f64) {
    if this.is_null() || key.is_null() {
        return;
    }
    if let Ok(key_str) = CStr::from_ptr(key as *const i8).to_str() {
        (*this).put(key_str.to_string(), val);
    }
}

/// Sets a boolean value in the given ParamScope object by string key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `key` must be a valid, null-terminated C string pointer
/// - If `this` or `key` is null, this function does nothing
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_bool(this: *mut ParamScope, key: *const i8, val: bool) {
    if this.is_null() || key.is_null() {
        return;
    }
    if let Ok(key_str) = CStr::from_ptr(key).to_str() {
        (*this).put(key_str.to_string(), val);
    }
}

/// Sets a boolean value in the given ParamScope object by string key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `key` must be a valid, null-terminated C string pointer
/// - If `this` or `key` is null, this function does nothing
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_bool(this: *mut ParamScope, key: *const i8, val: bool) {
    if this.is_null() || key.is_null() {
        return;
    }
    if let Ok(key_str) = CStr::from_ptr(key as *const i8).to_str() {
        (*this).put(key_str.to_string(), val);
    }
}

/// Sets a string value in the given ParamScope object by string key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `key` must be a valid, null-terminated C string pointer
/// - `val` must be a valid, null-terminated C string pointer
/// - If `this`, `key`, or `val` is null, this function does nothing
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_str(
    this: *mut ParamScope,
    key: *const i8,
    val: *const i8,
) {
    if this.is_null() || key.is_null() || val.is_null() {
        return;
    }
    if let (Ok(key_str), Ok(val_str)) = (CStr::from_ptr(key).to_str(), CStr::from_ptr(val).to_str()) {
        (*this).put(key_str.to_string(), val_str.to_string());
    }
}

/// Sets a string value in the given ParamScope object by string key.
/// 
/// # Safety
/// - `this` must be a valid, non-null pointer to a ParamScope
/// - `key` must be a valid, null-terminated C string pointer
/// - `val` must be a valid, null-terminated C string pointer
/// - If `this`, `key`, or `val` is null, this function does nothing
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn param_scope_put_str(
    this: *mut ParamScope,
    key: *const i8,
    val: *const i8,
) {
    if this.is_null() || key.is_null() || val.is_null() {
        return;
    }
    if let (Ok(key_str), Ok(val_str)) = (CStr::from_ptr(key as *const i8).to_str(), CStr::from_ptr(val as *const i8).to_str()) {
        (*this).put(key_str.to_string(), val_str.to_string());
    }
}
