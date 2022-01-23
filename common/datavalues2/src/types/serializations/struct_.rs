// Copyright 2021 Datafuse Labs.
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

use common_exception::ErrorCode;
use common_exception::Result;
use itertools::izip;

use crate::prelude::*;

pub struct StructSerializer {
    pub inners: Vec<Box<dyn TypeSerializer>>,
    pub types: Vec<DataTypePtr>,
}

impl TypeSerializer for StructSerializer {
    fn serialize_value(&self, value: &DataValue) -> Result<String> {
        if let DataValue::Struct(vals) = value {
            let mut res = String::new();
            res.push('(');
            let mut first = true;

            for (val, inner, typ) in izip!(vals, &self.inners, &self.types) {
                if !first {
                    res.push_str(", ");
                }
                first = false;

                let s = inner.serialize_value(val)?;
                if typ.data_type_id().is_quoted() {
                    res.push_str(&format!("'{}'", s));
                } else {
                    res.push_str(&s);
                }
            }
            res.push(')');
            Ok(res)
        } else {
            Err(ErrorCode::BadBytes("Incorrect Struct value"))
        }
    }

    fn serialize_column(&self, column: &ColumnRef) -> Result<Vec<String>> {
        let column: &StructColumn = Series::check_get(column)?;
        let mut result = Vec::with_capacity(column.len());
        for i in 0..column.len() {
            let val = column.get(i);
            let s = self.serialize_value(&val)?;
            result.push(s);
        }
        Ok(result)
    }
}
