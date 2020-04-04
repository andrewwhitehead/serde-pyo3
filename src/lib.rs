mod de;
mod error;
mod ser;

pub use de::{from_py, Deserializer};
pub use error::{Error, Result, ResultExt};
pub use ser::{to_py, Serializer};

use pyo3::{FromPyObject, PyAny, PyResult, Python};

/// Use as an argument in a py function
pub struct FromPyDeserialize<T>(T);

impl<'de, T> FromPyObject<'de> for FromPyDeserialize<T>
where
    T: for<'a> serde::de::Deserialize<'a>,
{
    fn extract(input: &'de PyAny) -> PyResult<Self> {
        let py = unsafe { Python::assume_gil_acquired() };
        from_py(py, input).map(Self).to_py_result()
    }
}

impl<T> FromPyDeserialize<T> {
    pub fn unwrap(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for FromPyDeserialize<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

#[test]
fn test_fn_arg() {
    use pyo3::prelude::*;
    use pyo3::py_run;
    use pyo3::wrap_pyfunction;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, PartialEq, Debug, Serialize)]
    struct Test {
        int: u32,
        seq: Vec<String>,
    }

    #[pyfunction]
    fn test(py: Python, arg: FromPyDeserialize<Test>) -> PyResult<PyObject> {
        let arg: Test = arg.unwrap();
        assert_eq!(
            arg,
            Test {
                int: 1,
                seq: vec!["one".to_owned(), "two".to_owned()],
            },
        );
        to_py(py, &arg).to_py_result()
    }

    let gil = Python::acquire_gil();
    let py = gil.python();

    let test_fn = wrap_pyfunction!(test)(py);
    py_run!(
        py,
        test_fn,
        r#"
        input = { "int": 1, "seq": ["one", "two"] }
        output = test_fn(input)
        assert output == input
    "#
    )
}
