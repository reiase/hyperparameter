use std::ffi::c_void;

use pyo3::prelude::*;
use pyo3::types::PyBool;
use pyo3::types::PyDict;
use pyo3::types::PyFloat;
use pyo3::types::PyInt;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::FromPyPointer;

use pyo3::exceptions::PyValueError;

use crate::entry::Value;
use crate::tls_storage::Storage;

use crate::tls_storage::MGR;

#[pyclass]
pub struct KVStorage {
    storage: Storage,
    isview: bool,
}

impl KVStorage {
    pub fn _storage(&mut self) -> *mut Storage {
        match self.isview {
            true => MGR.with(|mgr| mgr.borrow_mut().current.last().unwrap().clone()),
            false => &mut self.storage,
        }
    }
}

#[pymethods]
impl KVStorage {
    #[new]
    pub fn new() -> KVStorage {
        KVStorage {
            storage: Storage::new(),
            isview: false,
        }
    }

    pub unsafe fn storage(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let res = PyDict::new(py);
        let s = self._storage();
        for k in (*s).keys().iter() {
            match (*s).get(k).unwrap() {
                Value::Empty => Ok(()),
                Value::Int(v) => res.set_item(k, v),
                Value::Float(v) => res.set_item(k, v),
                Value::Text(v) => res.set_item(k, v.as_str()),
                Value::Boolen(v) => res.set_item(k, v),
                Value::UserDefined(v) => res.set_item(k, v as u64),
                Value::PyObject(v) => {
                    res.set_item(k, PyAny::from_owned_ptr(py, v as *mut pyo3::ffi::PyObject))
                }
            }
            .unwrap();
        }
        Ok(res.into())
    }

    pub unsafe fn keys(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        let s = self._storage();
        let res = PyList::new(py, (*s).keys());
        Ok(res.into())
    }

    pub unsafe fn _update(&mut self, kws: &PyDict, prefix: Option<String>) {
        kws.iter()
            .map(|(k, v)| {
                let key = match &prefix {
                    Some(p) => format!("{}.{}", p, k.extract::<String>().unwrap()),
                    None => k.extract::<String>().unwrap(),
                };
                if v.is_instance_of::<PyDict>().unwrap() {
                    self._update(&v.downcast::<PyDict>().unwrap(), Some(key));
                } else {
                    self.put(key, v).unwrap();
                }
            })
            .count();
    }

    pub unsafe fn update(&mut self, kws: &PyDict) {
        self._update(kws, None);
    }

    pub unsafe fn clear(&mut self) {
        let s = self._storage();
        for k in (*s).keys().iter() {
            self.storage.tree.put(k, Value::Empty);
        }
    }

    pub unsafe fn get(&mut self, py: Python<'_>, key: String) -> PyResult<Option<PyObject>> {
        let s = self._storage();
        match (*s).get(key) {
            Some(val) => match val {
                Value::Empty => Err(PyValueError::new_err("not found")),
                Value::Int(v) => Ok(Some(v.into_py(py))),
                Value::Float(v) => Ok(Some(v.into_py(py))),
                Value::Text(v) => Ok(Some(v.into_py(py))),
                Value::Boolen(v) => Ok(Some(v.into_py(py))),
                Value::UserDefined(v) => Ok(Some((v as u64).into_py(py))),
                Value::PyObject(v) => Ok(Some(
                    PyAny::from_owned_ptr(py, v as *mut pyo3::ffi::PyObject).into(),
                )),
            },
            None => Err(PyValueError::new_err("not found")),
        }
    }

    pub unsafe fn put(&mut self, key: String, val: &PyAny) -> PyResult<()> {
        if val.is_none() {
            return Ok(());
        }
        let s = self._storage();
        if val.is_instance_of::<PyBool>().unwrap() {
            (*s).put(key, val.extract::<bool>().unwrap());
        } else if val.is_instance_of::<PyFloat>().unwrap() {
            (*s).put(key, val.extract::<f64>().unwrap());
        } else if val.is_instance_of::<PyString>().unwrap() {
            (*s).put(key, val.extract::<&str>().unwrap());
        } else if val.is_instance_of::<PyInt>().unwrap() {
            (*s).put(key, val.extract::<i64>().unwrap());
        } else {
            (*s).put(key, Value::PyObject(val.into_ptr() as *mut c_void));
            // return Err(PyValueError::new_err(format!(
            //     "bad parameter value {}, {} is not supported",
            //     val.to_string(),
            //     val.get_type().to_string()
            // )));
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
            storage: Storage::new(),
            isview: true,
        }
    }
}

#[pymodule]
fn rbackend(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<KVStorage>()?;
    Ok(())
}
