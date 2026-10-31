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

use crate::{budget::Budget, grammar, ParseDiagnostic, ParseErrorKind, Span, SqlMode};

#[derive(Debug)]
pub struct ParseFailure {
    pub kind: ParseErrorKind,
    pub message: String,
    pub diagnostic: ParseDiagnostic,
    pub expanded_sql: String,
}

impl ParseFailure {
    pub(crate) fn at(kind: ParseErrorKind, message: impl Into<String>, span: Span) -> Self {
        let message = message.into();
        Self {
            kind,
            message: message.clone(),
            diagnostic: ParseDiagnostic { span, expected: Vec::new(), detail: message },
            expanded_sql: String::new(),
        }
    }

    pub(crate) fn syntax(
        sql: &str,
        mode: SqlMode,
        error: peg::error::ParseError<peg::str::LineCol>,
    ) -> Self {
        let span = grammar::sql::diagnostic_span(
            sql, mode, false, &Budget::lexical_scan(), error.location.offset,
        ).expect("diagnostic tokenization accepts every character");
        Self {
            kind: ParseErrorKind::Syntax,
            message: error.to_string(),
            diagnostic: ParseDiagnostic {
                span,
                expected: error.expected.tokens().map(str::to_owned).collect(),
                detail: String::new(),
            },
            expanded_sql: String::new(),
        }
    }

    pub(crate) fn remove_source_prefix(mut self, prefix: usize) -> Self {
        self.diagnostic.span.start = self.diagnostic.span.start.saturating_sub(prefix);
        self.diagnostic.span.end = self.diagnostic.span.end.saturating_sub(prefix);
        if prefix != 0 && !self.expanded_sql.is_empty() {
            self.expanded_sql.drain(..prefix);
        }
        self
    }
}

impl std::fmt::Display for ParseFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ParseFailure {}
