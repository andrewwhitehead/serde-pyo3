use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::{PyObject, Python, ToPyObject};
use serde::{ser, Serialize};

use super::error::{Error, Result};

pub struct Serializer<'a> {
    pub py: Python<'a>,
}
pub struct PyDictSerializer<'a> {
    root: &'a Serializer<'a>,
    dict: &'a PyDict,
    key: Option<PyObject>,
}
pub struct PyDictVariantSerializer<'a> {
    root: &'a Serializer<'a>,
    variant: PyObject,
    dict: &'a PyDict,
}
pub struct PyListSerializer<'a> {
    root: &'a Serializer<'a>,
    list: &'a PyList,
}
pub struct PyTupleSerializer<'a> {
    root: &'a Serializer<'a>,
    stack: Vec<PyObject>,
}
pub struct PyTupleVariantSerializer<'a> {
    root: &'a Serializer<'a>,
    variant: PyObject,
    stack: Vec<PyObject>,
}

pub fn to_py<'a, T>(py: Python<'a>, value: &T) -> Result<PyObject>
where
    T: Serialize,
{
    let serializer = Serializer { py };
    Ok(value.serialize(&serializer)?)
}

impl<'a> ser::Serializer for &'a Serializer<'a> {
    type Ok = PyObject;

    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeMap = PyDictSerializer<'a>;
    type SerializeSeq = PyListSerializer<'a>;
    type SerializeStruct = PyDictSerializer<'a>;
    type SerializeStructVariant = PyDictVariantSerializer<'a>;
    type SerializeTuple = PyTupleSerializer<'a>;
    type SerializeTupleStruct = PyTupleSerializer<'a>;
    type SerializeTupleVariant = PyTupleVariantSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<PyObject> {
        Ok(v.to_object(self.py))
    }

    fn serialize_i8(self, v: i8) -> Result<PyObject> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<PyObject> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<PyObject> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<PyObject> {
        Ok(v.to_object(self.py))
    }

    fn serialize_u8(self, v: u8) -> Result<PyObject> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<PyObject> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<PyObject> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<PyObject> {
        Ok(v.to_object(self.py))
    }

    fn serialize_f32(self, v: f32) -> Result<PyObject> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<PyObject> {
        Ok(v.to_object(self.py))
    }

    fn serialize_char(self, v: char) -> Result<PyObject> {
        Ok(v.to_string().to_object(self.py))
    }

    fn serialize_str(self, v: &str) -> Result<PyObject> {
        Ok(v.to_object(self.py))
    }

    // not currently called - requires specialization or serde-bytes
    fn serialize_bytes(self, v: &[u8]) -> Result<PyObject> {
        Ok(v.to_object(self.py))
    }

    fn serialize_none(self) -> Result<PyObject> {
        Ok(self.py.None())
    }

    fn serialize_some<T>(self, value: &T) -> Result<PyObject>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<PyObject> {
        // Ok(PyTuple::empty(self.py).to_object(self.py))
        Ok(self.py.None())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<PyObject> {
        // FIXME how should this be represented?
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<PyObject> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<PyObject>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<PyObject>
    where
        T: ?Sized + Serialize,
    {
        let dict = PyDict::new(self.py);
        let key = variant.serialize(&*self)?;
        let value = value.serialize(&*self)?;
        dict.set_item(key, value)?;
        Ok(dict.to_object(self.py))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(PyListSerializer {
            root: self,
            list: PyList::empty(self.py),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(PyTupleSerializer {
            root: self,
            stack: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(PyTupleSerializer {
            root: self,
            stack: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        let variant = variant.serialize(&*self)?;
        Ok(PyTupleVariantSerializer {
            root: self,
            variant,
            stack: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(PyDictSerializer {
            root: self,
            dict: PyDict::new(self.py),
            key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        let variant = variant.serialize(&*self)?;
        Ok(PyDictVariantSerializer {
            root: self,
            dict: PyDict::new(self.py),
            variant,
        })
    }
}

impl<'a> ser::SerializeSeq for PyListSerializer<'a> {
    type Ok = PyObject;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(self.root)?;
        Ok(self.list.append(value)?)
    }

    fn end(self) -> Result<PyObject> {
        Ok(self.list.to_object(self.root.py))
    }
}

impl<'a> ser::SerializeTuple for PyTupleSerializer<'a> {
    type Ok = PyObject;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.stack.push(value.serialize(self.root)?);
        Ok(())
    }

    fn end(self) -> Result<PyObject> {
        Ok(PyTuple::new(self.root.py, self.stack).to_object(self.root.py))
    }
}

impl<'a> ser::SerializeTupleStruct for PyTupleSerializer<'a> {
    type Ok = PyObject;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.stack.push(value.serialize(self.root)?);
        Ok(())
    }

    fn end(self) -> Result<PyObject> {
        Ok(PyTuple::new(self.root.py, self.stack).to_object(self.root.py))
    }
}

impl<'a> ser::SerializeTupleVariant for PyTupleVariantSerializer<'a> {
    type Ok = PyObject;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.stack.push(value.serialize(self.root)?);
        Ok(())
    }

    fn end(self) -> Result<PyObject> {
        let dict = PyDict::new(self.root.py);
        let tuple = PyTuple::new(self.root.py, self.stack).to_object(self.root.py);
        dict.set_item(self.variant, tuple)?;
        Ok(dict.to_object(self.root.py))
    }
}

impl<'a> ser::SerializeMap for PyDictSerializer<'a> {
    type Ok = PyObject;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.key.replace(key.serialize(self.root)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(self.root)?;
        self.dict.set_item(self.key.take(), value)?;
        Ok(())
    }

    fn end(self) -> Result<PyObject> {
        Ok(self.dict.to_object(self.root.py))
    }
}

impl<'a> ser::SerializeStruct for PyDictSerializer<'a> {
    type Ok = PyObject;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = key.serialize(self.root)?;
        let value = value.serialize(self.root)?;
        self.dict.set_item(key, value)?;
        Ok(())
    }

    fn end(self) -> Result<PyObject> {
        Ok(self.dict.to_object(self.root.py))
    }
}

impl<'a> ser::SerializeStructVariant for PyDictVariantSerializer<'a> {
    type Ok = PyObject;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = key.serialize(self.root)?;
        let value = value.serialize(self.root)?;
        self.dict.set_item(key, value)?;
        Ok(())
    }

    fn end(self) -> Result<PyObject> {
        let result = PyDict::new(self.root.py);
        let dict = self.dict.to_object(self.root.py);
        result.set_item(self.variant, dict)?;
        Ok(result.to_object(self.root.py))
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
    use pyo3::py_run;

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"],
        };
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = to_py(py, &test).unwrap();
        py_run!(
            py,
            obj,
            "print(obj); assert obj == {'int': 1, 'seq': ['a', 'b']}"
        );
    }

    #[test]
    fn test_bytes() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = to_py(py, &"hello".as_bytes()).unwrap();
        py_run!(
            py,
            obj,
            "print(obj); assert obj == [104, 101, 108, 108, 111]"
        );
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let gil = Python::acquire_gil();
        let py = gil.python();

        let u = to_py(py, &E::Unit).unwrap();
        py_run!(py, u, "assert u == 'Unit'");

        let n = to_py(py, &E::Newtype(1)).unwrap();
        py_run!(py, n, "assert n == {'Newtype': 1}");

        let t = to_py(py, &E::Tuple(1, 2)).unwrap();
        py_run!(py, t, "assert t == {'Tuple': (1, 2)}");

        let s = to_py(py, &E::Struct { a: 1 }).unwrap();
        py_run!(py, s, "assert s == {'Struct': {'a': 1}}");
    }
}
