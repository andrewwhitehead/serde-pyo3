use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};
use serde::Deserialize;

use pyo3::buffer::PyBuffer;
use pyo3::types::{PyDict, PyIterator, PyList, PySequence, PyString, PyTuple};
use pyo3::{AsPyPointer, FromPyObject, PyAny, PyTryFrom, PyTypeInfo, Python};

use super::error::{Error, Result};

pub struct Deserializer<'de> {
    py: Python<'de>,
    input: &'de PyAny,
}

impl<'de> Deserializer<'de> {
    pub fn from_py(py: Python<'de>, input: &'de PyAny) -> Self {
        Deserializer { py, input }
    }
}

pub fn from_py<'de, T>(py: Python<'de>, input: &'de PyAny) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::from_py(py, input);
    Ok(T::deserialize(&mut deserializer)?)
}

impl<'de> Deserializer<'de> {
    #[inline]
    fn downcast<T>(&mut self) -> Result<T>
    where
        T: for<'a> FromPyObject<'a>,
    {
        if let Ok(result) = T::extract(self.input) {
            Ok(result)
        } else {
            Err(Error::ExpectedInteger)
        }
    }

    #[inline]
    fn is_none(&self) -> bool {
        self.input.as_ptr() == unsafe { pyo3::ffi::Py_None() }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.is_none() {
            visitor.visit_unit()
        } else if let Ok(val) = self.downcast::<String>() {
            visitor.visit_string(val)
        } else if let Ok(val) = self.downcast::<bool>() {
            visitor.visit_bool(val)
        } else if let Ok(val) = self.downcast::<u64>() {
            visitor.visit_u64(val)
        } else if let Ok(val) = self.downcast::<f64>() {
            visitor.visit_f64(val)
        } else if <PyList as PyTypeInfo>::is_instance(self.input)
            || <PyTuple as PyTypeInfo>::is_instance(self.input)
        {
            self.deserialize_seq(visitor)
        } else if <PyDict as PyTypeInfo>::is_instance(self.input) {
            self.deserialize_map(visitor)
        } else {
            Err(Error::Syntax)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.downcast()?)
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.downcast()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.downcast()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.downcast()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.downcast()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.downcast()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.downcast()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.downcast()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.downcast()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.downcast()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.downcast()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let strval = self.downcast::<String>()?;
        if strval.len() == 1 {
            visitor.visit_char(strval.chars().next().unwrap())
        } else {
            Err(Error::ExpectedString)
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let strval = <PyString as PyTryFrom>::try_from(self.input)?;
        let strval = unsafe { std::str::from_utf8_unchecked(strval.as_bytes()?) };
        visitor.visit_borrowed_str(strval)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.downcast::<String>()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let buf = PyBuffer::get(self.py, self.input)?;
        if let Some(_) = buf.as_slice::<u8>(self.py) {
            let buf: &[u8] =
                unsafe { std::slice::from_raw_parts(buf.buf_ptr() as *const u8, buf.item_count()) };
            visitor.visit_borrowed_bytes(buf)
        } else {
            Err(Error::ExpectedNull)
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.downcast::<Vec<u8>>()?;
        visitor.visit_byte_buf(bytes)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.is_none() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.is_none() {
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNull)
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let seq = <PySequence as PyTryFrom>::try_from(self.input)?;
        match PyIterator::from_object(self.py, seq) {
            Ok(iter) => {
                let size = seq.len().map(|x| x as usize).ok();
                let value = visitor.visit_seq(SeqIter::new(self.py, iter, size))?;
                Ok(value)
            }
            Err(_) => Err(Error::ExpectedArray),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let dict = <PyDict as PyTryFrom>::try_from(self.input)?;
        visitor.visit_map(DictIter::new(self.py, dict))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if <PyString as PyTypeInfo>::is_instance(self.input) {
            let key: String = self.downcast()?;
            visitor.visit_enum(key.into_deserializer())
        } else {
            let dict = <PyDict as PyTryFrom>::try_from(self.input)?;
            if let Some(key) = dict.keys().iter().next() {
                if let Some(val) = dict.get_item(key) {
                    let value = visitor.visit_enum(Enum::new(self.py, key, val))?;
                    Ok(value)
                } else {
                    Err(Error::ExpectedMapComma)
                }
            } else {
                Err(Error::ExpectedMapComma)
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // result is ignored, can we skip downcast?
        self.deserialize_any(visitor)
    }
}

struct SeqIter<'de> {
    py: Python<'de>,
    input: PyIterator<'de>,
    size: Option<usize>,
}

impl<'de> SeqIter<'de> {
    fn new(py: Python<'de>, input: PyIterator<'de>, size: Option<usize>) -> Self {
        Self { py, input, size }
    }
}

impl<'de, 'a: 'de> SeqAccess<'de> for SeqIter<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(item) = self.input.next() {
            match item {
                Ok(val) => seed
                    .deserialize(&mut Deserializer::from_py(self.py.clone(), val))
                    .map(Some),
                Err(_) => Err(Error::ExpectedMapComma),
            }
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.size
    }
}

struct DictIter<'de> {
    py: Python<'de>,
    input: &'de PyDict,
    keys: &'de PyList,
    index: isize,
    size: isize,
}

impl<'de> DictIter<'de> {
    fn new(py: Python<'de>, input: &'de PyDict) -> Self {
        let keys = input.keys();
        Self {
            py,
            input,
            keys,
            index: 0,
            size: keys.len() as isize,
        }
    }
}

impl<'de, 'a: 'de> MapAccess<'de> for DictIter<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.index < self.size {
            let key = self.keys.get_item(self.index);
            seed.deserialize(&mut Deserializer::from_py(self.py.clone(), key))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let idx = self.index;
        self.index += 1;
        if let Some(item) = self.input.get_item(self.keys.get_item(idx)) {
            seed.deserialize(&mut Deserializer::from_py(self.py.clone(), item))
        } else {
            Err(Error::ExpectedMapComma)
        }
    }
}

struct Enum<'de> {
    py: Python<'de>,
    key: &'de PyAny,
    val: &'de PyAny,
}

impl<'de> Enum<'de> {
    fn new(py: Python<'de>, key: &'de PyAny, val: &'de PyAny) -> Self {
        Self { py, key, val }
    }
}

impl<'de, 'a: 'de> EnumAccess<'de> for Enum<'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut Deserializer::from_py(self.py, self.key))?;
        Ok((val, self))
    }
}

impl<'de, 'a: 'de> VariantAccess<'de> for Enum<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        unimplemented!()
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut Deserializer::from_py(self.py, self.val))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut Deserializer::from_py(self.py, self.val), visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(&mut Deserializer::from_py(self.py, self.val), visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ser::to_py;
    use pyo3::AsPyRef;
    use serde_json::{self, json, Value as JsonValue};
    use std::collections::HashMap;
    use std::iter::FromIterator;

    fn py_eval<'de, T: Deserialize<'de>>(py: Python<'de>, val: &str) -> T {
        let locals = PyDict::new(py);
        py.run(format!("ret = {}", val).as_str(), None, Some(locals))
            .unwrap();
        let result = locals.get_item("ret").unwrap();
        from_py(py, result).unwrap()
    }

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<String>,
        }

        let gil = Python::acquire_gil();
        let py = gil.python();
        let result: Test = py_eval(py, r#"{"seq":["a","b"], "int":1}"#);
        assert_eq!(
            result,
            Test {
                int: 1,
                seq: vec!["a".to_owned(), "b".to_owned()]
            }
        );
    }

    #[test]
    fn test_map() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let result: HashMap<&str, u32> = py_eval(py, r#"{"one": 1, "two": 2}"#);
        assert_eq!(
            result,
            HashMap::from_iter(vec![("one", 1), ("two", 2)].into_iter())
        );
    }

    #[test]
    fn test_bytes_buf() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let result: Vec<u8> = py_eval(py, r#"b"abc""#);
        assert_eq!(result, vec![97u8, 98u8, 99u8]);
    }

    #[test]
    fn test_bytes_borrowed() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let result: &[u8] = py_eval(py, r#"b"abc""#);
        assert_eq!(result, &[97u8, 98u8, 99u8]);
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let gil = Python::acquire_gil();
        let py = gil.python();

        let result: E = py_eval(py, r#""Unit""#);
        assert_eq!(result, E::Unit);

        let result: E = py_eval(py, r#"{"Newtype":1}"#);
        assert_eq!(result, E::Newtype(1));

        let result: E = py_eval(py, r#"{"Tuple":[1,2]}"#);
        assert_eq!(result, E::Tuple(1, 2));

        let result: E = py_eval(py, r#"{"Struct":{"a":1}}"#);
        assert_eq!(result, E::Struct { a: 1 });
    }

    #[test]
    fn test_json() {
        let jsonval: JsonValue = json!({
            "a": [true, null, false, 1, 2.0, {"nested": []}],
            "b": "ok"
        });

        let gil = Python::acquire_gil();
        let py = gil.python();

        let into = to_py(py, &jsonval).unwrap();
        let result: JsonValue = from_py(py, into.as_ref(py)).unwrap();
        assert_eq!(result, jsonval);
    }
}
