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

use crate::{
    bridge::ffi::{
        CacheFilter, CacheKind, ShowFilter, TransactionAccess, TransactionIsolation,
        TransactionSetting, TransactionOptions, VariableScope,
    },
    node::{Kind, Node},
};

pub(crate) fn prepared_source(node: &Node) -> &Node {
    &node.children[0]
}

pub(crate) fn execution_arguments(node: &Node) -> &[Node] {
    &node.children
}

pub(crate) fn transaction_options(node: &Node) -> TransactionOptions {
    let mut options = TransactionOptions {
        consistent_snapshot: false,
        access: TransactionAccess::Default,
    };
    for option in &node.children {
        match option.kind {
            Kind::ConsistentSnapshot => options.consistent_snapshot = true,
            Kind::ReadOnly => options.access = TransactionAccess::ReadOnly,
            Kind::ReadWrite => options.access = TransactionAccess::ReadWrite,
            _ => {}
        }
    }
    options
}

pub(crate) fn transaction_setting(statement: &Node) -> TransactionSetting {
    let characteristics = &statement.children[1].children;
    TransactionSetting {
        scope: match statement.children[0].value {
            1 => VariableScope::Global,
            2 => VariableScope::Session,
            _ => VariableScope::Default,
        },
        access: if characteristics[0].kind == Kind::Missing {
            TransactionAccess::Default
        } else if characteristics[0].value == 2 {
            TransactionAccess::ReadOnly
        } else {
            TransactionAccess::ReadWrite
        },
        isolation: if characteristics[1].kind == Kind::Missing {
            TransactionIsolation::Default
        } else {
            match characteristics[1].value {
                0 => TransactionIsolation::ReadUncommitted,
                1 => TransactionIsolation::ReadCommitted,
                2 => TransactionIsolation::RepeatableRead,
                3 => TransactionIsolation::Serializable,
                _ => unreachable!(),
            }
        },
    }
}

pub(crate) fn empty_query_has_comment(statement: &Node) -> bool {
    statement.value != 0
}
pub(crate) fn freeze_is_major(statement: &Node) -> bool {
    statement.children[0].value == 1
}

pub(crate) fn cache_flush_kind(statement: &Node) -> CacheKind {
    match statement.children[0].text.as_str() {
        "plan" => CacheKind::Plan,
        "pl" => CacheKind::Procedure,
        "ps" => CacheKind::PreparedStatement,
        "lib" => CacheKind::Library,
        _ => unreachable!(),
    }
}
pub(crate) fn cache_flush_has_namespace(statement: &Node) -> bool {
    statement.children[1].kind != Kind::Missing
}
pub(crate) fn cache_flush_namespace(statement: &Node) -> &str {
    &statement.children[1].children[0].text
}
pub(crate) fn cache_flush_filter(statement: &Node) -> CacheFilter {
    match statement.children[2].kind {
        Kind::Missing => CacheFilter::All,
        Kind::SqlId => CacheFilter::SqlId,
        Kind::SchemaId => CacheFilter::SchemaId,
        _ => unreachable!(),
    }
}
pub(crate) fn cache_flush_filter_value(statement: &Node) -> &Node {
    &statement.children[2].children[0]
}
pub(crate) fn cache_flush_has_databases(statement: &Node) -> bool {
    statement.children[3].kind != Kind::Missing
}
pub(crate) fn cache_flush_databases(statement: &Node) -> &str {
    &statement.children[3].children[0].text
}
pub(crate) fn kv_cache_has_name(statement: &Node) -> bool {
    statement.children[0].kind != Kind::Missing
}
pub(crate) fn kv_cache_name(statement: &Node) -> &str {
    &statement.children[0].children[0].text
}
pub(crate) fn configuration_assignments(statement: &Node) -> &[Node] {
    &statement.children[0].children
}
pub(crate) fn configuration_name(assignment: &Node) -> &str {
    &assignment.children[0].text
}
pub(crate) fn configuration_value(assignment: &Node) -> &Node {
    &assignment.children[1]
}
pub(crate) fn configuration_comment(assignment: &Node) -> &str {
    &assignment.children[2].text
}
pub(crate) fn tracepoint_options(statement: &Node) -> &[Node] {
    &statement.children[0].children
}
pub(crate) fn tracepoint_value(option: &Node) -> &Node {
    &option.children[0]
}

pub(crate) fn show_full(statement: &Node) -> bool {
    match statement.kind {
        Kind::ShowTables => statement.children[2].value != 0,
        Kind::ShowColumns | Kind::ShowProcesslist => statement.children[0].value != 0,
        _ => false,
    }
}
pub(crate) fn show_extended(statement: &Node) -> bool {
    statement.kind == Kind::ShowIndexes && statement.children[3].value != 0
}
pub(crate) fn show_global(statement: &Node) -> bool {
    matches!(statement.kind, Kind::ShowVariables | Kind::ShowStatus) && statement.children[0].value == 1
}
pub(crate) fn show_if_not_exists(statement: &Node) -> bool {
    statement.kind == Kind::ShowCreateDatabase && statement.children[0].kind != Kind::Missing
}
fn show_database_clause(statement: &Node) -> Option<&Node> {
    let database = match statement.kind {
        Kind::ShowTables | Kind::ShowTableStatus => &statement.children[0],
        Kind::ShowColumns => &statement.children[2],
        Kind::ShowIndexes => &statement.children[1],
        Kind::ShowCreateDatabase => return Some(&statement.children[1]),
        _ => return None,
    };
    if database.kind == Kind::Missing {
        None
    } else {
        Some(&database.children[0])
    }
}
pub(crate) fn show_has_database(statement: &Node) -> bool {
    show_database_clause(statement).is_some()
}
pub(crate) fn show_database(statement: &Node) -> &Node {
    show_database_clause(statement).unwrap()
}
pub(crate) fn show_object(statement: &Node) -> &Node {
    &statement.children[usize::from(statement.kind == Kind::ShowColumns)]
}
fn show_condition(statement: &Node) -> Option<&Node> {
    let condition = match statement.kind {
        Kind::ShowTables | Kind::ShowTableStatus | Kind::ShowVariables | Kind::ShowStatus => {
            &statement.children[1]
        }
        Kind::ShowColumns => &statement.children[3],
        Kind::ShowIndexes => &statement.children[2],
        Kind::ShowParameters | Kind::ShowCharset | Kind::ShowCollation | Kind::ShowDatabases => {
            &statement.children[0]
        }
        _ => return None,
    };
    if condition.kind == Kind::Missing {
        None
    } else {
        Some(condition)
    }
}
pub(crate) fn show_filter(statement: &Node) -> ShowFilter {
    match show_condition(statement).map(|condition| condition.kind) {
        None => ShowFilter::None,
        Some(Kind::Where) => ShowFilter::Where,
        Some(Kind::LikeClause) => ShowFilter::Like,
        _ => unreachable!(),
    }
}
pub(crate) fn show_filter_expression(statement: &Node) -> &Node {
    &show_condition(statement).unwrap().children[0]
}
pub(crate) fn show_like_pattern(statement: &Node) -> &Node {
    &show_condition(statement).unwrap().children[0]
}
pub(crate) fn show_like_escape(statement: &Node) -> &Node {
    &show_condition(statement).unwrap().children[1]
}
pub(crate) fn show_has_account(statement: &Node) -> bool {
    statement.children[0].kind != Kind::Missing
}
pub(crate) fn show_account_name(statement: &Node) -> &str {
    &statement.children[0].children[0].text
}
pub(crate) fn show_account_has_host(statement: &Node) -> bool {
    statement.children[0].children[1].kind != Kind::Missing
}
pub(crate) fn show_account_host(statement: &Node) -> &str {
    &statement.children[0].children[1].text
}
pub(crate) fn show_has_limit(statement: &Node) -> bool {
    statement.children[0].kind != Kind::Missing
}
pub(crate) fn show_limit_offset(statement: &Node) -> u64 {
    statement.children[0].children[0].value
}
pub(crate) fn show_limit_count(statement: &Node) -> u64 {
    statement.children[0].children[1].value
}

pub(crate) fn setting_assignments(statement: &Node) -> &[Node] {
    &statement.children
}

pub(crate) fn setting_scope(assignment: &Node) -> VariableScope {
    match assignment.value {
        1 => VariableScope::Global,
        2 => VariableScope::Session,
        _ => VariableScope::Default,
    }
}

pub(crate) fn setting_charset(statement: &Node) -> &Node {
    &statement.children[0]
}

pub(crate) fn setting_has_collation(statement: &Node) -> bool {
    statement.kind == Kind::SetNames && statement.children[1].kind != Kind::Missing
}

pub(crate) fn setting_collation(statement: &Node) -> &Node {
    &statement.children[1]
}

pub(crate) fn kill_query_only(statement: &Node) -> bool {
    statement.children[0].value != 0
}

pub(crate) fn kill_target(statement: &Node) -> &Node {
    &statement.children[1]
}

pub(crate) fn diagnostics_stacked(statement: &Node) -> bool {
    statement.children[1].value == 0
}

pub(crate) fn diagnostics_has_condition(statement: &Node) -> bool {
    statement.children[2].kind != Kind::Missing
}

pub(crate) fn diagnostics_condition(statement: &Node) -> &Node {
    &statement.children[2]
}

pub(crate) fn diagnostics_items(statement: &Node) -> &[Node] {
    &statement.children[3].children
}

pub(crate) fn diagnostics_target(item: &Node) -> &Node {
    &item.children[0]
}

pub(crate) fn diagnostics_item_name(item: &Node) -> &str {
    &item.children[1].text
}
