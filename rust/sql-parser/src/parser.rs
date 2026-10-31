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
use crate::{budget::Budget, ParseFailure, Span, StackBounds};
use crate::diagnostic::Diagnostics;

pub struct Parsed {
    root: Node,
    expanded_sql: String,
    source_prefix: usize,
    parameters: Parameters,
}

enum ParameterStyle {
    Unspecified,
    Ordinal,
    Named,
}

struct Parameters {
    count: usize,
    style: ParameterStyle,
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
    pub fn parameters(&self) -> usize { self.parameters.count }
    pub fn has_ordinal_parameters(&self) -> bool { matches!(self.parameters.style, ParameterStyle::Ordinal) }
    pub fn named_parameters(&self) -> bool { matches!(self.parameters.style, ParameterStyle::Named) }
}

fn stack_failure(offset: usize) -> ParseFailure {
    ParseFailure::at(ParseErrorKind::StackLimit, "parser stack limit exceeded", Span { start:offset,end:offset })
}

pub fn parse(sql: &[u8], mode: SqlMode, parse_mode: ParseMode, bounds: StackBounds) -> Result<Parsed, ParseFailure> {
    let budget = &Budget::new(bounds);
    budget.checkpoint(0).map_err(|_| stack_failure(0))?;
    let sql = &sql[..normalized_sql_length(sql)];
    let sql = std::str::from_utf8(sql).map_err(|error| ParseFailure::at(
        ParseErrorKind::Syntax, error.to_string(),
        Span { start:error.valid_up_to(),end:error.valid_up_to()+error.error_len().unwrap_or(0) },
    ))?;
    let source = if parse_mode == ParseMode::RoutineDefinition {
        let mut source = String::with_capacity("CREATE\n".len() + sql.len());
        source.push_str("CREATE\n");
        source.push_str(sql);
        std::borrow::Cow::Owned(source)
    } else {
        std::borrow::Cow::Borrowed(sql)
    };
    let source_prefix = source.len() - sql.len();
    parse_source(source, source_prefix, mode, parse_mode, budget)
        .map_err(|error| error.remove_source_prefix(source_prefix))
}

fn parse_source(source: std::borrow::Cow<'_, str>, source_prefix: usize,
                mode: SqlMode, parse_mode: ParseMode, budget: &Budget) -> Result<Parsed, ParseFailure> {
    let sql = source.as_ref();
    let diagnostics = &Diagnostics::default();
    let entry = |text: &str| {
        diagnostics.clear();
        let pl = matches!(parse_mode, ParseMode::Routine | ParseMode::Trigger | ParseMode::RoutineDefinition
            | ParseMode::PlSql | ParseMode::PlExpression)
            || (parse_mode == ParseMode::Sql && classify_statement(text.as_bytes(), mode).pl != crate::bridge::ffi::PlPrefix::None);
        match parse_mode {
        ParseMode::Routine => grammar::sql::routine_body(text, mode, pl, budget, diagnostics),
        ParseMode::Trigger => grammar::sql::trigger(text, mode, pl, budget, diagnostics),
        ParseMode::Statistics => grammar::sql::statistics_options(text, mode, pl, budget, diagnostics),
        ParseMode::Expression | ParseMode::PlExpression => grammar::sql::expression(text, mode, pl, budget, diagnostics),
        ParseMode::Sql | ParseMode::RoutineDefinition | ParseMode::PlSql => grammar::sql::query(text, mode, pl, budget, diagnostics),
        _ => unreachable!("unknown parser entry point"),
        }.map(|root| (root, pl))
    };
    let mut expanded_sql = String::new();
    let parsed = grammar::executable_sql(sql, mode, true).and_then(|mut text| {
        let mut parsed = entry(&text);
        if budget.failure().is_none() && parsed.is_err() && matches!(text, std::borrow::Cow::Owned(_)) {
            text = grammar::executable_sql(sql, mode, false)?;
            parsed = entry(&text);
        }
        if let std::borrow::Cow::Owned(text) = text {
            expanded_sql = text;
        }
        parsed
    });
    if let Some(offset) = budget.failure() { return Err(stack_failure(offset)); }
    let parsed = parsed.map_err(|error| ParseFailure::syntax(
        if expanded_sql.is_empty() { sql } else { &expanded_sql }, mode, error, diagnostics,
    )).and_then(|(query, pl)| {
        if parse_mode == ParseMode::RoutineDefinition
            && !matches!(query.kind, Kind::CreateProcedure | Kind::CreateFunction) {
            Err(ParseFailure::at(ParseErrorKind::Syntax, "expected procedure or function definition",
                Span { start:query.start,end:query.end }))
        } else {
            Ok((query, pl))
        }
    });
    if expanded_sql.is_empty() {
        if let std::borrow::Cow::Owned(source) = source {
            expanded_sql = source;
        }
    }
    let (mut root, pl) = parsed.map_err(|mut error| {
        error.expanded_sql = std::mem::take(&mut expanded_sql);
        error
    })?;
    validate_types(&root).map_err(|mut error| {
        error.expanded_sql = std::mem::take(&mut expanded_sql);
        error
    })?;
    let parameters = number_parameters(&mut root).map_err(|span| {
        let mut error = ParseFailure::at(ParseErrorKind::Syntax,
            "ordinal and named parameters cannot be combined", span);
        error.expanded_sql = std::mem::take(&mut expanded_sql);
        error
    })?;
    if root.kind == Kind::CreateView && parameters.count != 0 && parse_mode != ParseMode::PlSql {
        let mut error = ParseFailure::at(ParseErrorKind::Syntax,
            "CREATE VIEW does not accept parameter markers", Span { start:root.start,end:root.end });
        error.expanded_sql = expanded_sql;
        return Err(error);
    }
    if parameters.count != 0 && parse_mode == ParseMode::Sql && !pl
        && !statement_accepts_parameters(&root)
    {
        let mut error = ParseFailure::at(ParseErrorKind::Syntax,
            "statement does not accept parameter markers", Span { start:root.start,end:root.end });
        error.expanded_sql = expanded_sql;
        return Err(error);
    }
    Ok(Parsed { root, expanded_sql, source_prefix, parameters })
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

fn validate_types(node: &Node) -> Result<(), ParseFailure> {
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        let cast_target = match node.kind {
            Kind::Cast => Some(crate::expression::cast_target(node)),
            Kind::JsonQuery => Some(crate::expression::json_returning(node)),
            _ => None,
        };
        if let Some(target) = cast_target.filter(|target| target.kind == Kind::DataType) {
            let mut error = ParseFailure::at(ParseErrorKind::Syntax,
                format!("invalid CAST target type: {}", target.text),
                Span { start:target.start,end:target.end });
            error.diagnostic.expected.push("a supported CAST target type".into());
            return Err(error);
        }
        pending.extend(node.children.iter().rev());
    }
    Ok(())
}

fn number_parameters(node: &mut Node) -> Result<Parameters, Span> {
    let mut bindings = Parameters { count: 0, style: ParameterStyle::Unspecified };
    let mut names: Vec<(String, usize)> = Vec::new();
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
            if matches!(bindings.style, ParameterStyle::Named) {
                return Err(Span { start:node.start,end:node.end });
            }
            bindings.style = ParameterStyle::Ordinal;
            node.value = bindings.count as u64;
            bindings.count += 1;
        } else {
            if !node.text.as_bytes()[1].is_ascii_digit() {
                if matches!(bindings.style, ParameterStyle::Ordinal) {
                    return Err(Span { start:node.start,end:node.end });
                }
                bindings.style = ParameterStyle::Named;
            }
            let index = names
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&node.text))
                .map(|(_, index)| *index);
            node.value = index.unwrap_or_else(|| {
                let index = bindings.count;
                names.push((node.text.clone(), index));
                bindings.count += 1;
                index
            }) as u64;
        }
    }
    Ok(bindings)
}

