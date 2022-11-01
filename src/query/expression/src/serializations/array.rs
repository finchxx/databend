// Copyright 2022 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use common_arrow::arrow::buffer::Buffer;
use common_exception::Result;
use common_io::prelude::FormatSettings;
use serde_json::Value;

use crate::types::DataType;
use crate::Column;
use crate::TypeSerializer;
use crate::TypeSerializerImpl;

#[derive(Clone)]
pub struct ArraySerializer {
    pub(crate) offsets: Buffer<u64>,
    pub(crate) inner: Box<TypeSerializerImpl>,
}

impl ArraySerializer {
    pub fn try_create(col: Column, inner_ty: &DataType) -> Result<Self, String> {
        let column = col
            .into_array()
            .map_err(|_| "unable to get array column".to_string())?;

        let inner = inner_ty.create_serializer(column.values)?;
        Ok(Self {
            offsets: column.offsets,
            inner: Box::new(inner),
        })
    }
}

impl TypeSerializer for ArraySerializer {
    fn write_field(&self, row_index: usize, buf: &mut Vec<u8>, format: &FormatSettings) {
        let start = self.offsets[row_index] as usize;
        let end = self.offsets[row_index + 1] as usize;
        buf.push(b'[');
        let inner = &self.inner;
        for i in start..end {
            if i != start {
                buf.extend_from_slice(b", ");
            }
            inner.write_field_quoted(i, buf, format, b'\'');
        }
        buf.push(b']');
    }

    fn serialize_json_values(&self, format: &FormatSettings) -> Result<Vec<Value>, String> {
        let size = self.offsets.len() - 1;
        let mut result = Vec::with_capacity(size);
        let inner = self.inner.serialize_json_values(format)?;
        let mut iter = inner.into_iter();
        for i in 0..size {
            let len = (self.offsets[i + 1] - self.offsets[i]) as usize;
            let value = iter.by_ref().take(len).collect();
            result.push(Value::Array(value))
        }
        Ok(result)
    }
}
