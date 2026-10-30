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

use crate::{grammar, node::{Kind, Node}, prefix::{normalized_sql_length, classify_statement}, ParseMode, SqlMode};
use crate::bridge::ffi::ParseErrorKind;

pub struct Parsed {
    root: Node,
    expanded_sql: String,
    source_prefix: usize,
    parameters: usize,
    parameter_names: Vec<String>,
    named_parameters: bool,
}

impl Parsed {
    pub fn root(&self) -> &Node { &self.root }
    pub fn expanded_sql(&self) -> &str { &self.expanded_sql }
    pub fn original_span(&self, node: &Node) -> crate::bridge::ffi::Span {
        crate::bridge::ffi::Span {
            start: node.start.saturating_sub(self.source_prefix),
            end: node.end.saturating_sub(self.source_prefix),
        }
    }
    pub fn parameters(&self) -> usize { self.parameters }
    pub fn parameter_names(&self) -> &[String] { &self.parameter_names }
    pub fn named_parameters(&self) -> bool { self.named_parameters }
}

#[derive(Debug)]
pub struct ParseFailure {
    pub kind: ParseErrorKind,
    pub message: String,
    pub offset: usize,
    pub expanded_sql: String,
}

impl std::fmt::Display for ParseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ParseFailure {}

pub fn parse(sql: &[u8], mode: SqlMode, parse_mode: ParseMode) -> Result<Parsed, ParseFailure> {
    let sql = &sql[..normalized_sql_length(sql)];
    let sql = std::str::from_utf8(sql).map_err(|error| ParseFailure {
        kind: ParseErrorKind::Syntax,
        message: error.to_string(),
        offset: error.valid_up_to(),
        expanded_sql: String::new(),
    })?;
    let source = if parse_mode == ParseMode::RoutineDefinition {
        let mut source = String::with_capacity("CREATE\n".len() + sql.len());
        source.push_str("CREATE\n");
        source.push_str(sql);
        std::borrow::Cow::Owned(source)
    } else {
        std::borrow::Cow::Borrowed(sql)
    };
    let source_prefix = source.len() - sql.len();
    let sql = source.as_ref();
    let entry = |text: &str| {
        let pl = matches!(parse_mode, ParseMode::Routine | ParseMode::Trigger | ParseMode::RoutineDefinition
            | ParseMode::PlSql | ParseMode::PlExpression)
            || (parse_mode == ParseMode::Sql && classify_statement(text.as_bytes(), mode).pl != crate::bridge::ffi::PlPrefix::None);
        match parse_mode {
        ParseMode::Routine => grammar::sql::routine_body(text, mode, pl),
        ParseMode::Trigger => grammar::sql::trigger(text, mode, pl),
        ParseMode::Statistics => grammar::sql::statistics_options(text, mode, pl),
        ParseMode::Expression | ParseMode::PlExpression => grammar::sql::expression(text, mode, pl),
        ParseMode::Sql | ParseMode::RoutineDefinition | ParseMode::PlSql => grammar::sql::query(text, mode, pl),
        _ => unreachable!("unknown parser entry point"),
        }.map(|root| (root, pl))
    };
    let mut expanded_sql = String::new();
    let parsed = grammar::executable_sql(sql, mode, true).and_then(|mut text| {
        let mut parsed = entry(&text);
        if parsed.is_err() && matches!(text, std::borrow::Cow::Owned(_)) {
            text = grammar::executable_sql(sql, mode, false)?;
            parsed = entry(&text);
        }
        if let std::borrow::Cow::Owned(text) = text {
            expanded_sql = text;
        }
        parsed
    });
    let parsed = parsed.map_err(|error| {
        let offset = if error.location.offset == sql.len() {
            grammar::sql::last_token_start(sql, mode, false).unwrap()
        } else {
            error.location.offset
        };
        (error.to_string(), offset)
    }).and_then(|(query, pl)| {
        if parse_mode == ParseMode::RoutineDefinition
            && !matches!(query.kind, Kind::CreateProcedure | Kind::CreateFunction) {
            Err(("expected procedure or function definition".into(), query.start))
        } else {
            Ok((query, pl))
        }
    });
    if expanded_sql.is_empty() {
        if let std::borrow::Cow::Owned(source) = source {
            expanded_sql = source;
        }
    }
    let (mut root, pl) = parsed.map_err(|(message, offset)| ParseFailure {
        kind: ParseErrorKind::Syntax,
        message, offset, expanded_sql: std::mem::take(&mut expanded_sql),
    })?;
    if parse_mode == ParseMode::PlSql {
        if let Some(offset) = invalid_bit_width(&root) {
            return Err(ParseFailure {
                kind: ParseErrorKind::BitWidth,
                message: "BIT display width exceeds 64".into(),
                offset,
                expanded_sql,
            });
        }
    }
    let mut parameters = 0;
    let mut parameter_names = Vec::new();
    number_parameters(&mut root, &mut parameters, &mut parameter_names).map_err(|offset| ParseFailure {
        kind: ParseErrorKind::Syntax,
        message: "ordinal and named parameters cannot be combined".into(),
        offset,
        expanded_sql: std::mem::take(&mut expanded_sql),
    })?;
    if root.kind == Kind::CreateView && parameters != 0 && parse_mode != ParseMode::PlSql {
        return Err(ParseFailure {
            kind: ParseErrorKind::Syntax,
            message: "CREATE VIEW does not accept parameter markers".into(),
            offset: root.start,
            expanded_sql,
        });
    }
    if parameters != 0 && parse_mode == ParseMode::Sql && !pl
        && !statement_accepts_parameters(&root)
    {
        return Err(ParseFailure {
            kind: ParseErrorKind::Syntax,
            message: "statement does not accept parameter markers".into(),
            offset: root.start,
            expanded_sql,
        });
    }
    let named_parameters = parameter_names.iter().any(|name| !name.as_bytes()[1].is_ascii_digit());
    Ok(Parsed { root, expanded_sql, source_prefix, parameters, parameter_names, named_parameters })
}

fn statement_accepts_parameters(node: &Node) -> bool {
    match node.kind {
        Kind::Select | Kind::Insert | Kind::Update | Kind::Delete | Kind::Explain
        | Kind::CreateOutline | Kind::AlterOutline | Kind::Prepare | Kind::VariableSet
        | Kind::Kill | Kind::Diagnostics => true,
        Kind::CreateTable => crate::ddl::table_definition(node).select_mode != crate::TableSelectMode::None,
        _ => false,
    }
}

fn invalid_bit_width(node: &Node) -> Option<usize> {
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        if node.kind == Kind::DataType && node.text == "bit"
            && node.children.first().is_some_and(|width| width.value > 64)
        {
            return Some(node.start);
        }
        pending.extend(node.children.iter().rev());
    }
    None
}

fn number_parameters(
    node: &mut Node,
    count: &mut usize,
    names: &mut Vec<String>,
) -> Result<(), usize> {
    let mut parameters = Vec::new();
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        if node.kind == Kind::Parameter {
            parameters.push(node);
        } else {
            pending.extend(node.children.iter_mut().rev());
        }
    }
    parameters.sort_by_key(|parameter| parameter.start);
    for node in parameters {
        if node.text.is_empty() {
            if !names.is_empty() {
                return Err(node.start);
            }
            node.value = *count as u64;
            *count += 1;
        } else {
            if *count != names.len() {
                return Err(node.start);
            }
            let index = names
                .iter()
                .position(|name| name.eq_ignore_ascii_case(&node.text));
            node.value = index.unwrap_or_else(|| {
                names.push(node.text.clone());
                *count += 1;
                names.len() - 1
            }) as u64;
        }
    }
    Ok(())
}

