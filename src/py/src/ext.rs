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

/// 获取当前线程的 Python handler 标记
fn get_python_handler() -> Option<i64> {
    PYTHON_HANDLER.with(|h| h.borrow().clone())
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
}

#[pymethods]
impl KVStorage {
    #[new]
    pub fn new() -> KVStorage {
        KVStorage {
            storage: ParamScope::default(),
            current_handler: None,
        }
    }

    pub fn clone(&self) -> KVStorage {
        KVStorage {
            storage: self.storage.clone(),
            current_handler: self.current_handler.clone(),
        }
    }

    /// 检查并更新 handler（如果不一致）
    /// 这个方法会检查 thread-local 中的 Python handler，如果与当前 handler 不一致，
    /// 则更新当前 handler。实际的存储同步由 Python 侧通过 ContextVar 管理。
    fn check_and_sync_handler(&mut self) {
        // 从 thread-local 获取 Python handler（不需要 GIL）
        let python_handler = get_python_handler();
        
        // 如果 handler 不一致，更新当前 handler（整数比较，非常快）
        if self.current_handler != python_handler {
            if python_handler.is_none() {
                // handler 为 None，清空存储
                self.storage = ParamScope::default();
            }
            self.current_handler = python_handler;
        }
    }

    pub unsafe fn storage(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let res = PyDict::new(py);
        for k in self.storage.keys().iter() {
            match self.storage.get(k) {
                Value::Empty => Ok(()),
                Value::Int(v) => res.set_item(k, v),
                Value::Float(v) => res.set_item(k, v),
                Value::Text(v) => res.set_item(k, v.as_str()),
                Value::Boolean(v) => res.set_item(k, v),
                Value::UserDefined(v, kind, _) => {
                    if kind == UserDefinedType::PyObjectType as i32 {
                        // Borrowed pointer; increment refcount so Value's drop remains balanced.
                        let obj = PyAny::from_borrowed_ptr_or_err(py, v as *mut pyo3::ffi::PyObject)?;
                        res.set_item(k, obj)
                    } else {
                        res.set_item(k, v)
                    }
                }
            }
            .map_err(|e| e)?;
        }
        Ok(res.into())
    }

    pub unsafe fn keys(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let res = PyList::new(py, self.storage.keys());
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
        // 检查并更新 handler（如果需要）
        self.check_and_sync_handler();
        self._update(py, kws, None);
    }

    pub unsafe fn clear(&mut self) {
        for k in self.storage.keys().iter() {
            self.storage.put(k, Value::Empty);
        }
    }

    pub unsafe fn get(&mut self, py: Python<'_>, key: String) -> PyResult<Option<PyObject>> {
        // 检查并更新 handler（如果需要）
        self.check_and_sync_handler();
        
        match self.storage.get(key) {
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
        match self.storage.get_with_hash(hkey) {
            Value::Empty => Err(PyValueError::new_err("not found")),
            Value::Int(v) => Ok(Some(v.into_py(py))),
            Value::Float(v) => Ok(Some(v.into_py(py))),
            Value::Text(v) => Ok(Some(v.into_py(py))),
            Value::Boolean(v) => Ok(Some(v.into_py(py))),
            Value::UserDefined(v, k, _) => {
                if k == UserDefinedType::PyObjectType as i32 {
                    let obj = PyAny::from_borrowed_ptr_or_err(py, v as *mut pyo3::ffi::PyObject)?;
                    Ok(Some(obj.into()))
                } else {
                    Ok(Some((v as u64).into_py(py)))
                }
            }
        }
    }

    pub unsafe fn put(&mut self, py: Python<'_>, key: String, val: &PyAny) -> PyResult<()> {
        // 检查并更新 handler（如果需要）
        self.check_and_sync_handler();
        
        // 执行 put 操作
        if val.is_none() {
            self.storage.put(key, Value::Empty);
        } else if val.is_instance_of::<PyBool>() {
            self.storage.put(key, val.extract::<bool>()?);
        } else if val.is_instance_of::<PyFloat>() {
            self.storage.put(key, val.extract::<f64>()?);
        } else if val.is_instance_of::<PyString>() {
            self.storage.put(key, val.extract::<&str>()?.to_string());
        } else if val.is_instance_of::<PyInt>() {
            self.storage.put(key, val.extract::<i64>()?);
        } else {
            // Py_XINCREF(val.into_ptr());
            self.storage.put(key, make_value_from_pyobject(val.into_ptr()));
        }
        
        Ok(())
    }

    pub fn enter(&mut self) {
        // enter 时不需要检查 handler，因为已经在 Python 侧设置了
        self.storage.enter();
    }

    pub fn exit(&mut self) {
        self.storage.exit();
    }

    #[staticmethod]
    pub fn current() -> KVStorage {
        KVStorage {
            storage: ParamScope::Nothing,
            current_handler: None,
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
