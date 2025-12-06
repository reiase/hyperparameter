#![allow(non_local_definitions)]

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
}

#[pymethods]
impl KVStorage {
    #[new]
    pub fn new() -> KVStorage {
        KVStorage {
            storage: ParamScope::default(),
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
                        // Take owned pointer; convert to PyAny safely
                        let obj = PyAny::from_owned_ptr_or_err(py, v as *mut pyo3::ffi::PyObject)?;
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

    pub unsafe fn _update(&mut self, kws: &PyDict, prefix: Option<String>) {
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
                self._update(dict, Some(full_key));
            } else {
                // Best-effort; ignore errors to avoid panic
                let _ = self.put(full_key, v);
            }
        }
    }

    pub unsafe fn update(&mut self, kws: &PyDict) {
        self._update(kws, None);
    }

    pub unsafe fn clear(&mut self) {
        for k in self.storage.keys().iter() {
            self.storage.put(k, Value::Empty);
        }
    }

    pub unsafe fn get(&mut self, py: Python<'_>, key: String) -> PyResult<Option<PyObject>> {
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

    pub unsafe fn put(&mut self, key: String, val: &PyAny) -> PyResult<()> {
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
        self.storage.enter();
    }

    pub fn exit(&mut self) {
        self.storage.exit();
    }

    #[staticmethod]
    pub fn current() -> KVStorage {
        KVStorage {
            storage: ParamScope::Nothing,
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
    Ok(())
}
