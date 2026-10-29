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

use crate::bridge::{
    self,
    ffi::{ColumnReference, ExpressionInfo, ParseMode},
};
use crate::node::{Kind, Node};
use crate::SqlMode;

pub(crate) fn inspect_expression(sql: &[u8], mode: SqlMode) -> ExpressionInfo {
    let parsed = match crate::parse(sql, mode, ParseMode::Expression) {
        Ok(parsed) => parsed,
        Err(error) => return ExpressionInfo {
            root_kind: Kind::Missing,
            function_name: String::new(),
            qualified_function: false,
            columns: Vec::new(),
            error: error.message,
            error_offset: error.offset,
        },
    };
    let mut root = parsed.root();
    while root.kind == Kind::Parenthesized {
        root = &root.children[0];
    }
    let function_name = match root.kind {
        Kind::Function | Kind::Cast | Kind::StringConcat => crate::expression::call_name(root).into(),
        _ => String::new(),
    };
    let mut columns = Vec::new();
    collect_columns(root, &mut columns);
    ExpressionInfo {
        root_kind: root.kind,
        function_name,
        qualified_function: !crate::expression::call_qualifiers(root).is_empty(),
        columns,
        error: String::new(),
        error_offset: 0,
    }
}

fn collect_columns(node: &Node, columns: &mut Vec<ColumnReference>) {
    if node.kind == Kind::Column {
        columns.push(ColumnReference {
            parts: node
                .children
                .iter()
                .filter(|child| child.kind == Kind::Ident)
                .map(|child| child.text.clone())
                .collect(),
            wildcard: node.children.iter().any(|child| child.kind == Kind::Star),
        });
    }
    for child in &node.children {
        collect_columns(child, columns);
    }
}

pub(crate) fn embedded_sql_references(statement: &Node) -> Vec<bridge::ffi::SqlReference<'_>> {
    fn visit<'a>(node: &'a Node, references: &mut Vec<bridge::ffi::SqlReference<'a>>) {
        use bridge::ffi::{SqlReference, SqlReferenceKind as Role};
        if matches!(node.kind, Kind::IntoVariables | Kind::TrailingIntoVariables) {
            references.push(SqlReference {
                kind: Role::Into,
                node,
            });
            return;
        }
        if node.kind == Kind::Diagnostics {
            if crate::command::diagnostics_has_condition(node) {
                visit(crate::command::diagnostics_condition(node), references);
            }
            for item in crate::command::diagnostics_items(node) {
                let target = crate::command::diagnostics_target(item);
                if target.kind == Kind::Ident {
                    references.push(SqlReference {
                        kind: Role::Column,
                        node: target,
                    });
                } else {
                    visit(target, references);
                }
            }
            return;
        }
        for child in &node.children {
            visit(child, references);
        }
        let kind = match node.kind {
            Kind::Column => Role::Column,
            Kind::Relation => Role::Relation,
            Kind::Function => Role::Function,
            Kind::Projection => Role::Projection,
            _ => return,
        };
        references.push(SqlReference { kind, node });
    }
    let mut references = Vec::new();
    visit(statement, &mut references);
    references
}

pub(crate) fn query_has_mixed_string_array(node: &Node) -> bool {
    fn literal_kind(mut node: &Node) -> Option<crate::bridge::ffi::LiteralKind> {
        use crate::bridge::ffi::{LiteralKind, UnaryOperator};
        while node.kind == Kind::Parenthesized {
            node = crate::expression::parenthesized_expression(node);
        }
        if node.kind == Kind::Unary
            && matches!(
                crate::expression::unary_operator(node),
                UnaryOperator::Plus | UnaryOperator::Minus
            )
        {
            return literal_kind(crate::expression::unary_operand(node)).filter(|kind| {
                matches!(
                    *kind,
                    LiteralKind::Integer
                        | LiteralKind::Unsigned
                        | LiteralKind::Decimal
                        | LiteralKind::Float
                )
            });
        }
        if node.kind == Kind::CharsetLiteral {
            Some(LiteralKind::String)
        } else if crate::expression::is_literal(node) {
            Some(crate::expression::expression_literal(node).kind)
        } else {
            None
        }
    }
    if node.kind == Kind::ArrayLiteral {
        use crate::bridge::ffi::LiteralKind;
        let mut string = false;
        let mut scalar = false;
        for value in crate::expression::call_arguments(node) {
            match literal_kind(value) {
                Some(LiteralKind::String | LiteralKind::NationalString) => string = true,
                Some(LiteralKind::Null | LiteralKind::Placeholder) | None => {}
                Some(_) => scalar = true,
            }
        }
        if string && scalar {
            return true;
        }
    }
    node.children.iter().any(query_has_mixed_string_array)
}

pub fn syntax_literals(syntax: &Node) -> Vec<bridge::ffi::LiteralReference<'_>> {
    fn collect<'a>(node: &'a Node, literals: &mut Vec<bridge::ffi::LiteralReference<'a>>) {
        if crate::expression::is_literal(node) {
            literals.push(bridge::ffi::LiteralReference { node });
        } else {
            for child in &node.children {
                collect(child, literals);
            }
        }
    }
    let mut literals = Vec::new();
    collect(syntax, &mut literals);
    literals
}
