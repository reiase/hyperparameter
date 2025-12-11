#![allow(non_local_definitions)]

use std::cell::RefCell;
use std::ffi::c_void;

use hyperparameter::*;
use pyo3::exceptions::PyValueError;
use pyo3::ffi::Py_XDECREF;
use pyo3::prelude::*;
use pyo3::types::PyBool;
use pyo3::types::PyDict;
use pyo3::types::PyFloat;
use pyo3::types::PyInt;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::FromPyPointer;

/// Thread-local handler 标记，用于标识当前 Python 上下文的 handler
/// Handler 是 storage 对象的地址（int64），由 Python 侧在切换上下文时设置
thread_local! {
    static PYTHON_HANDLER: RefCell<Option<i64>> = RefCell::new(None);
}

/// 设置当前线程的 Python handler 标记（由 Python 调用）
/// handler 是 storage 对象的地址（Python id() 的返回值）
#[pyfunction]
pub fn set_python_handler(handler: Option<i64>) {
    PYTHON_HANDLER.with(|h| {
        *h.borrow_mut() = handler;
    });
}

#[repr(C)]
enum UserDefinedType {
    PyObjectType = 1,
}

// impl Into<Value> for *mut pyo3::ffi::PyObject {
//     fn into(self) -> Value {
//         Value::managed(
//             self as *mut c_void,
//             UserDefinedType::PyObjectType as i32,
//             |obj: *mut c_void| unsafe {
//                 Py_DecRef(obj as *mut pyo3::ffi::PyObject);
//             },
//         )
//     }
// }

/// Convert a PyObject pointer into a Value with a GIL-safe destructor.
fn make_value_from_pyobject(obj: *mut pyo3::ffi::PyObject) -> Value {
    // `Py_XDECREF` requires the GIL; wrap the drop in `Python::with_gil` to
    // ensure the object is decref'd safely even if drop happens on another thread.
    unsafe fn drop_pyobject(ptr: *mut c_void) {
        Python::with_gil(|_| {
            Py_XDECREF(ptr as *mut pyo3::ffi::PyObject);
        });
    }

    Value::managed(
        obj as *mut c_void,
        UserDefinedType::PyObjectType as i32,
        drop_pyobject,
    )
}

#[pyclass]
pub struct KVStorage {
    storage: ParamScope,
    current_handler: Option<i64>,
    is_current: bool,  // 标记是否通过current()创建
}

#[pymethods]
impl KVStorage {
    #[new]
    pub fn new() -> KVStorage {
        KVStorage {
            storage: ParamScope::default(),
            current_handler: None,
            is_current: false,
        }
    }

    pub fn clone(&self) -> KVStorage {
        // clone时，创建新的storage副本，但重置current_handler为None
        // 这样新的KVStorage实例会使用自己的handler（由Python侧设置）
        KVStorage {
            storage: self.storage.clone(),
            current_handler: None,  // 重置handler，让Python侧设置新的handler
            is_current: false,  // clone后的实例不是current，不应该回退到with_current_storage
        }
    }

    pub unsafe fn storage(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let res = PyDict::new(py);
        // 先添加self.storage中的值
        if let ParamScope::Just(ref changes) = self.storage {
            for (_, entry) in changes.iter() {
                match entry.value() {
                    Value::Empty => Ok(()),
                    Value::Int(v) => res.set_item(&entry.key, v),
                    Value::Float(v) => res.set_item(&entry.key, v),
                    Value::Text(v) => res.set_item(&entry.key, v.as_str()),
                    Value::Boolean(v) => res.set_item(&entry.key, v),
                    Value::UserDefined(v, kind, _) => {
                        if *kind == UserDefinedType::PyObjectType as i32 {
                            // Borrowed pointer; increment refcount so Value's drop remains balanced.
                            let obj = PyAny::from_borrowed_ptr_or_err(py, *v as *mut pyo3::ffi::PyObject)?;
                            res.set_item(&entry.key, obj)
                        } else {
                            res.set_item(&entry.key, *v as u64)
                        }
                    }
                }
                .map_err(|e| e)?;
            }
        }
        // 然后添加with_current_storage中的值（如果self.storage中没有）
        with_current_storage(|ts| {
            for (hkey, entry) in ts.params.iter() {
                let key = &entry.key;
                // 如果res中已经有这个key，跳过（self.storage优先）
                if res.contains(key).unwrap_or(false) {
                    continue;
                }
                match entry.value() {
                    Value::Empty => {}
                    Value::Int(v) => {
                        let _ = res.set_item(key, *v);
                    }
                    Value::Float(v) => {
                        let _ = res.set_item(key, *v);
                    }
                    Value::Text(v) => {
                        let _ = res.set_item(key, v.as_str());
                    }
                    Value::Boolean(v) => {
                        let _ = res.set_item(key, *v);
                    }
                    Value::UserDefined(v, k, _) => {
                        if *k == UserDefinedType::PyObjectType as i32 {
                            if let Ok(obj) = PyAny::from_borrowed_ptr_or_err(py, *v as *mut pyo3::ffi::PyObject) {
                                let _ = res.set_item(key, obj);
                            }
                        } else {
                            let _ = res.set_item(key, *v as u64);
                        }
                    }
                }
            }
        });
        Ok(res.into())
    }

    pub unsafe fn keys(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        // 先从self.storage读取
        let mut keys: Vec<String> = if let ParamScope::Just(ref changes) = self.storage {
            changes.values().map(|e| e.key.clone()).collect()
        } else {
            Vec::new()
        };
        // 如果self.storage是ParamScope::Nothing，从with_current_storage读取（支持enter/exit机制）
        if matches!(self.storage, ParamScope::Nothing) {
            with_current_storage(|ts| {
                for entry in ts.params.values() {
                    if !keys.contains(&entry.key) {
                        keys.push(entry.key.clone());
                    }
                }
            });
        }
        let res = PyList::new(py, keys);
        Ok(res.into())
    }

    pub unsafe fn _update(&mut self, py: Python<'_>, kws: &PyDict, prefix: Option<String>) {
        for (k, v) in kws.iter() {
            let key: String = match k.extract() {
                Ok(s) => s,
                Err(_) => continue, // skip non-string keys
            };
            let full_key = match &prefix {
                Some(p) => format!("{}.{}", p, key),
                None => key,
            };
            if let Ok(dict) = v.downcast::<PyDict>() {
                self._update(py, dict, Some(full_key));
            } else {
                // Best-effort; ignore errors to avoid panic
                let _ = self.put(py, full_key, v);
            }
        }
    }

    pub unsafe fn update(&mut self, py: Python<'_>, kws: &PyDict) {
        // 不再检查handler，因为Python侧已经通过ContextVar管理了正确的storage对象
        // 在异步环境下，check_and_sync_handler会导致不同任务的KVStorage对象被错误同步
        self._update(py, kws, None);
    }

    pub unsafe fn clear(&mut self) {
        for k in self.storage.keys().iter() {
            self.storage.put(k, Value::Empty);
        }
    }

    pub unsafe fn get(&mut self, py: Python<'_>, key: String) -> PyResult<Option<PyObject>> {
        // 先检查self.storage中是否有值
        let hkey = key.xxh();
        let value = if let ParamScope::Just(ref changes) = self.storage {
            if let Some(e) = changes.get(&hkey) {
                match e.value() {
                    Value::Empty => Value::Empty,
                    v => v.clone(),
                }
            } else {
                Value::Empty
            }
        } else {
            Value::Empty
        };
        
        // 如果self.storage中没有值，回退到with_current_storage（用于支持enter/exit机制）
        // 使用ParamScope::get_with_hash()，它会自动处理回退逻辑：
        // 1. 先检查self.storage中是否有值
        // 2. 如果没有，回退到with_current_storage
        // 这确保了enter()后，新的KVStorage实例可以读取到with_current_storage中的参数
        let value = if matches!(value, Value::Empty) {
            self.storage.get_with_hash(hkey)
        } else {
            value
        };
        
        match value {
            Value::Empty => Err(PyValueError::new_err("not found")),
            Value::Int(v) => Ok(Some(v.into_py(py))),
            Value::Float(v) => Ok(Some(v.into_py(py))),
            Value::Text(v) => Ok(Some(v.into_py(py))),
            Value::Boolean(v) => Ok(Some(v.into_py(py))),
            Value::UserDefined(v, k, _) => {
                if k == UserDefinedType::PyObjectType as i32 {
                    // borrowed ptr; convert with safety check
                    let obj = PyAny::from_borrowed_ptr_or_err(py, v as *mut pyo3::ffi::PyObject)?;
                    Ok(Some(obj.into()))
                } else {
                    Ok(Some((v as u64).into_py(py)))
                }
            }
        }
    }

    pub unsafe fn get_entry(&mut self, py: Python<'_>, hkey: u64) -> PyResult<Option<PyObject>> {
        // 先检查self.storage中是否有值
        let value = if let ParamScope::Just(ref changes) = self.storage {
            if let Some(e) = changes.get(&hkey) {
                match e.value() {
                    Value::Empty => Value::Empty,
                    v => v.clone(),
                }
            } else {
                Value::Empty
            }
        } else {
            Value::Empty
        };

        // 如果self.storage中没有值，回退到with_current_storage（用于支持enter/exit机制）
        // 使用ParamScope::get_with_hash()，它会自动处理回退逻辑
        let value = if matches!(value, Value::Empty) {
            self.storage.get_with_hash(hkey)
        } else {
            value
        };

        match value {
            Value::Empty => Err(PyValueError::new_err("not found")),
            Value::Int(v) => Ok(Some(v.into_py(py))),
            Value::Float(v) => Ok(Some(v.into_py(py))),
            Value::Text(v) => Ok(Some(v.into_py(py))),
            Value::Boolean(v) => Ok(Some(v.into_py(py))),
            Value::UserDefined(v, k, _) => {
                if k == UserDefinedType::PyObjectType as i32 {
                    // borrowed ptr; convert with safety check
                    let obj = PyAny::from_borrowed_ptr_or_err(py, v as *mut pyo3::ffi::PyObject)?;
                    Ok(Some(obj.into()))
                } else {
                    Ok(Some((v as u64).into_py(py)))
                }
            }
        }
    }

    pub unsafe fn put(&mut self, py: Python<'_>, key: String, val: &PyAny) -> PyResult<()> {
        // 确保storage是ParamScope::Just状态，这样才能正确存储参数
        if matches!(self.storage, ParamScope::Nothing) {
            self.storage = ParamScope::default();
        }
        
        // 先更新self.storage
        let value = if val.is_none() {
            Value::Empty
        } else if val.is_instance_of::<PyBool>() {
            Value::Boolean(val.extract::<bool>()?)
        } else if val.is_instance_of::<PyFloat>() {
            Value::Float(val.extract::<f64>()?)
        } else if val.is_instance_of::<PyString>() {
            Value::Text(val.extract::<&str>()?.to_string())
        } else if val.is_instance_of::<PyInt>() {
            Value::Int(val.extract::<i64>()?)
        } else {
            make_value_from_pyobject(val.into_ptr())
        };
        
        self.storage.put(key.clone(), value.clone());
        
        // 只有当通过current()创建时，才更新with_current_storage（用于支持current()机制）
        // 否则会导致参数泄漏到全局存储中
        if self.is_current {
            with_current_storage(|ts| {
                ts.put(key, value);
            });
        }
        
        Ok(())
    }

    pub fn enter(&mut self) {
        // 调用ParamScope::enter()以支持with_current_storage机制
        // 这对于直接使用KVStorage的测试（不通过TLSKVStorage）是必要的
        self.storage.enter();
    }

    pub fn exit(&mut self) {
        // 调用ParamScope::exit()以支持with_current_storage机制
        // 这对于直接使用KVStorage的测试（不通过TLSKVStorage）是必要的
        self.storage.exit();
    }

    #[staticmethod]
    pub fn current() -> KVStorage {
        // 使用ParamScope::capture()来获取当前with_current_storage中的参数
        KVStorage {
            storage: ParamScope::capture(),
            current_handler: None,
            is_current: true,  // 标记为通过current()创建
        }
    }

    #[staticmethod]
    pub fn frozen() {
        frozen();
    }
}

#[pyfunction]
pub fn xxh64(s: &str) -> u64 {
    s.xxh()
}

#[pymodule]
fn librbackend(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<KVStorage>()?;
    m.add_function(wrap_pyfunction!(xxh64, m)?)?;
    m.add_function(wrap_pyfunction!(set_python_handler, m)?)?;
    Ok(())
}
