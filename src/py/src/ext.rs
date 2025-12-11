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

/// Thread-local handler to identify the current Python context.
/// The handler is the storage object's address (int64), set by Python when switching contexts.
thread_local! {
    static PYTHON_HANDLER: RefCell<Option<i64>> = RefCell::new(None);
}

/// Set the current thread's Python handler (called by Python).
/// The handler is the storage object's address (return value of Python id()).
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
    is_current: bool,
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
        KVStorage {
            storage: self.storage.clone(),
            current_handler: None,
            is_current: false,
        }
    }

    pub unsafe fn storage(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let res = PyDict::new(py);
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
        with_current_storage(|ts| {
            for (_, entry) in ts.params.iter() {
                let key = &entry.key;
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
        let mut keys: Vec<String> = if let ParamScope::Just(ref changes) = self.storage {
            changes.values().map(|e| e.key.clone()).collect()
        } else {
            Vec::new()
        };
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
        self._update(py, kws, None);
    }

    pub unsafe fn clear(&mut self) {
        for k in self.storage.keys().iter() {
            self.storage.put(k, Value::Empty);
        }
    }

    pub unsafe fn get(&mut self, py: Python<'_>, key: String) -> PyResult<Option<PyObject>> {
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
        if matches!(self.storage, ParamScope::Nothing) {
            self.storage = ParamScope::default();
        }
        
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
        
        if self.is_current {
            with_current_storage(|ts| {
                ts.put(key, value);
            });
        }
        
        Ok(())
    }

    pub fn enter(&mut self) {
        self.storage.enter();
    }

    pub fn exit(&mut self) {
        self.storage.exit();
    }

    #[staticmethod]
    pub fn current() -> KVStorage {
        KVStorage {
            storage: ParamScope::capture(),
            current_handler: None,
            is_current: true,
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
