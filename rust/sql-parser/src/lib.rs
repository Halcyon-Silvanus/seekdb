// Copyright (c) 2026 OceanBase.
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

mod access;
mod batch;
mod parameterize;
mod prefix;
mod bridge;
mod command;
mod ddl;
mod embedded_sql;
mod expression;
mod fts;
mod grammar;
mod inspect;
mod lex;
mod node;
mod query;
mod routine;
mod scan;
mod statistics;

pub use bridge::ffi::*;
pub use bridge::{parse_sql};
pub use batch::{compress_insert, compress_values, restore_parameter_sql};
pub use prefix::{classify_statement, normalized_sql_length, is_single_statement, split_sql, fast_parameterization_allowed, transaction_command, insert_outline_hint};
pub use parameterize::{parameterize, classify_parameter_tokens, rewrite_parameter_sql, fold_parameter_in_lists, unchecked_parameter_types};
pub use scan::{begin_scan, finish_scan, Scanner};
pub use fts::{parse_search_query};
pub use embedded_sql::{embedded_sql_symbols, rewrite_embedded_sql};
pub use lex::{decode_binary_literal, identifier_requires_quotes, normalize_numeric_sign, integer_literal_is_nonnegative};
