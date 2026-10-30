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

use crate::bridge::ffi;
pub(crate) use ffi::NodeKind as Kind;

pub struct Node {
    pub(crate) kind: Kind,
    pub(crate) children: Vec<Node>,
    pub(crate) text: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) value: u64,
    pub(crate) depth: usize,
    pub(crate) operator: Option<Operator>,
}

impl Drop for Node {
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.children);
        while let Some(mut node) = pending.pop() {
            pending.append(&mut node.children);
        }
    }
}

pub(crate) enum Operator {
    Unary(ffi::UnaryOperator),
    Binary(ffi::BinaryOperator),
}

pub(crate) const MAX_DEPTH: usize = 1024;

pub(crate) struct Name {
    pub text: String,
    pub quoted: bool,
    pub start: usize,
    pub end: usize,
}

#[derive(Default)]
pub(crate) struct Select {
    pub with: Option<Node>,
    pub parenthesized: bool,
    pub set: Option<Node>,
    pub options: Vec<Node>,
    pub projection: Vec<Node>,
    pub into: Option<Node>,
    pub into_end: Option<Node>,
    pub approximate: Option<Node>,
    pub vector_parameters: Option<Node>,
    pub from: Vec<Node>,
    pub filter: Option<Node>,
    pub group_by: Option<Node>,
    pub having: Option<Node>,
    pub order_by: Vec<Node>,
    pub limit: Option<Node>,
    pub lock: Option<Node>,
    pub hints: Vec<Node>,
    pub windows: Vec<Node>,
}

impl Select {
    pub fn into_cte(
        mut self: Box<Self>,
        name: Name,
        columns: Vec<Name>,
        recursive: bool,
    ) -> Result<Node, &'static str> {
        if recursive {
            if let Some(set) = &mut self.set {
                if set.children.len() > 2 {
                    let right = set.children.pop().unwrap();
                    let mut left = Box::<Self>::default();
                    left.set = Some(Node::new(set.kind, std::mem::take(&mut set.children)));
                    let left = Node::select(left)?;
                    set.children = vec![left, right];
                }
            }
        }
        Ok(Node::new(
            Kind::CommonTable,
            vec![
                Node::name(name),
                Node::list(
                    Kind::ColumnList,
                    columns.into_iter().map(Node::name).collect(),
                ),
                Node::select(self)?,
            ],
        ))
    }

    pub fn combine(mut self: Box<Self>, kind: Kind, right: Box<Self>) -> Result<Box<Self>, &'static str> {
        let right = Node::select(right)?;
        let flatten = matches!(kind, Kind::Union | Kind::UnionAll)
            && !self.parenthesized
            && self.order_by.is_empty()
            && self.limit.is_none()
            && self.set.as_ref().is_some_and(|set| set.kind == kind);
        let children = if flatten {
            let mut children = self.set.take().unwrap().into_children();
            children.push(right);
            children
        } else {
            vec![Node::select(self)?, right]
        };
        let mut combined = Box::<Self>::default();
        combined.set = Some(Node::new(kind, children));
        Ok(combined)
    }

    pub fn finish(
        mut self: Box<Self>,
        order: Vec<Node>,
        limit: Option<Node>,
        lock: Option<Node>,
        into: Option<Node>,
        approximate: Option<Node>,
        vector_parameters: Option<Node>,
    ) -> Result<Box<Self>, &'static str> {
        if approximate.is_some()
            && (order.is_empty()
                || limit.is_none()
                || self.from.is_empty()
                || self.parenthesized
                || self.set.is_some())
        {
            return Err("APPROX requires a simple table query with ORDER BY and LIMIT");
        }
        if self.parenthesized && (!order.is_empty() || limit.is_some()) {
            let table = Node::table(Node::select(self)?, None)?;
            self = Box::<Self>::default();
            self.projection = vec![Node::new(
                Kind::Projection,
                vec![Node::new(Kind::Star, vec![])],
            )];
            self.from = vec![table];
        }
        if !order.is_empty() {
            self.order_by = order;
        }
        self.limit = limit.or(self.limit);
        self.lock = lock.or(self.lock);
        self.into_end = into.or(self.into_end);
        self.approximate = approximate.or(self.approximate);
        self.vector_parameters = vector_parameters.or(self.vector_parameters);
        Ok(self)
    }
}

impl Node {
    pub fn kind(&self) -> Kind { self.kind }
    pub fn as_text(&self) -> &str { &self.text }
    pub fn span(&self) -> ffi::Span { ffi::Span { start: self.start, end: self.end } }

    fn contains_user_variable(&self) -> bool {
        let mut pending = vec![self];
        while let Some(node) = pending.pop() {
            if matches!(node.kind, Kind::GetUserVariable | Kind::UserAssignment) {
                return true;
            }
            pending.extend(&node.children);
        }
        false
    }

    pub(crate) fn into_children(mut self) -> Vec<Self> {
        std::mem::take(&mut self.children)
    }

    pub(crate) fn with_kind(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }

    pub(crate) fn pl_expression(self) -> Result<Self, &'static str> {
        if !self.contains_user_variable() {
            return Ok(self);
        }
        let (start, end) = (self.start, self.end);
        let alias = Self::new(Kind::Ident, vec![]).text("SUBQUERY".into());
        let projection = Self::new(
            Kind::Projection,
            vec![Self::new(Kind::Alias, vec![self, alias])],
        );
        let mut query = Box::<Select>::default();
        query.projection = vec![projection];
        let mut query = Self::select(query)?;
        query.kind = Kind::PlSubquery;
        Ok(query.at(start, end))
    }

    fn setting_identifier(&self) -> Option<&str> {
        let mut value = self;
        while value.kind == Kind::Parenthesized {
            value = crate::expression::parenthesized_expression(value);
        }
        if value.kind == Kind::Column && !value.is_wildcard() {
            if let [name] = crate::query::name_parts(value) {
                return Some(&name.text);
            }
        }
        None
    }

    pub(crate) fn setting_value(variable: &Self, mut value: Self) -> Self {
        if variable.kind != Kind::UserVariable {
            if let Some(name) = value.setting_identifier() {
                value = Self::new(Kind::String, vec![])
                    .text(name.into())
                    .at(value.start, value.end);
            }
        }
        value
    }

    pub(crate) fn routine_setting_value(variable: &Self, value: Self) -> Result<Self, &'static str> {
        let off = value.setting_identifier().is_some_and(|name| name.eq_ignore_ascii_case("off"));
        let mut value = if variable.kind == Kind::SystemVariable
            || (variable.kind != Kind::UserVariable && off)
        {
            Self::setting_value(variable, value)
        } else {
            value
        };
        if off && value.kind == Kind::String {
            value.text = "OFF".into();
        }
        value.pl_expression()
    }

    pub(crate) fn optional(node: Option<Self>) -> Self {
        node.unwrap_or_else(Self::missing)
    }

    pub(crate) fn list(kind: Kind, children: Vec<Self>) -> Self {
        if children.is_empty() {
            Self::missing()
        } else {
            Self::new(kind, children)
        }
    }

    pub(crate) fn where_clause(value: Option<Self>) -> Self {
        Self::optional(value.map(|value| Self::new(Kind::Where, vec![value])))
    }

    pub(crate) fn insert(
        replace: bool,
        ignore: bool,
        table: Self,
        columns: Vec<Self>,
        values: Self,
        assignments: Vec<Self>,
        hints: Vec<Self>,
    ) -> Self {
        let source = match values.kind {
            Kind::ValueList => Self::new(Kind::InsertRows, values.into_children()),
            Kind::AssignList => Self::new(Kind::InsertAssignments, values.into_children()),
            Kind::Select => Self::new(Kind::InsertQuery, vec![values]),
            _ => unreachable!(),
        };
        let mut children = vec![Self::new(Kind::InsertTarget, vec![table]), source];
        if !columns.is_empty() {
            children.push(Self::new(Kind::InsertColumns, columns));
        }
        if !assignments.is_empty() {
            children.push(Self::new(Kind::DuplicateKeyAssignments, assignments));
        }
        if !hints.is_empty() {
            children.push(Self::new(Kind::Hints, hints));
        }
        if replace {
            children.push(Self::new(Kind::Replace, vec![]));
        }
        if ignore {
            children.push(Self::new(Kind::Ignore, vec![]));
        }
        Self::new(Kind::Insert, children)
    }

    pub(crate) fn data_type(
        name: &str,
        parameters: Vec<Self>,
        unsigned: Option<bool>,
        zerofill: bool,
        start: usize,
        end: usize,
    ) -> Result<Self, &'static str> {
        let numeric = matches!(
            name,
            "tinyint"
                | "smallint"
                | "mediumint"
                | "int"
                | "bigint"
                | "float"
                | "double"
                | "decimal"
        );
        let max_parameters = if matches!(name, "float" | "double" | "decimal") {
            2
        } else {
            1
        };
        if parameters.len() > max_parameters
            || parameters
                .iter()
                .any(|parameter| !matches!(parameter.kind, Kind::Int | Kind::Uint))
            || (name == "double" && parameters.len() == 1)
            || (matches!(name, "varchar" | "varbinary") && parameters.is_empty())
            || (matches!(name, "bool" | "boolean" | "date" | "json") && !parameters.is_empty())
            || (matches!(
                name,
                "tinytext"
                    | "tinyblob"
                    | "mediumtext"
                    | "mediumblob"
                    | "longtext"
                    | "longblob"
                    | "string"
            ) && !parameters.is_empty())
            || (!numeric && (unsigned.is_some() || zerofill))
        {
            return Err("invalid data type parameters");
        }
        Ok(Self::new(Kind::DataType, parameters)
            .text(name.into())
            .value(u64::from(unsigned.unwrap_or(false)) | (u64::from(zerofill) << 1))
            .at(start, end))
    }

    pub(crate) fn new(kind: Kind, children: Vec<Self>) -> Self {
        let depth = children.iter().map(|child| child.depth).max().unwrap_or(0);
        Self {
            kind,
            children,
            text: String::new(),
            start: 0,
            end: 0,
            value: 0,
            depth,
            operator: None,
        }
    }

    pub(crate) fn missing() -> Self {
        Self::new(Kind::Missing, vec![])
    }

    pub(crate) fn text(mut self, text: String) -> Self {
        self.text = text;
        self
    }
    pub(crate) fn value(mut self, value: u64) -> Self {
        self.value = value;
        self
    }
    pub(crate) fn at(mut self, start: usize, end: usize) -> Self {
        self.start = start;
        self.end = end;
        self
    }

    pub(crate) fn expression(mut self, start: usize, end: usize) -> Result<Self, &'static str> {
        if self.depth >= MAX_DEPTH {
            return Err("expression depth below 1024 levels");
        }
        self.depth += 1;
        Ok(self.at(start, end))
    }

    pub(crate) fn name(name: Name) -> Self {
        Self::new(Kind::Ident, vec![])
            .text(name.text)
            .value(u64::from(name.quoted))
            .at(name.start, name.end)
    }

    pub(crate) fn number(text: &str, start: usize, end: usize) -> Result<Self, &'static str> {
        let mut value = 0;
        let kind = if text.contains(['e', 'E']) {
            Kind::Double
        } else if let Ok(integer) = text.parse::<u64>() {
            value = integer;
            if integer <= i64::MAX as u64 {
                Kind::Int
            } else {
                Kind::Uint
            }
        } else {
            Kind::Number
        };
        Self::new(kind, vec![])
            .text(text.into())
            .value(value)
            .expression(start, end)
    }

    pub(crate) fn column(
        names: Vec<Name>,
        star: bool,
        start: usize,
        end: usize,
    ) -> Result<Self, &'static str> {
        let count = names.len() + usize::from(star);
        if count > 3 {
            return Err("column names have at most three parts");
        }
        let mut children: Vec<_> = names.into_iter().map(Self::name).collect();
        if star {
            children.push(Self::new(Kind::Star, vec![]).at(end - 1, end));
        }
        Self::new(Kind::Column, children).expression(start, end)
    }

    pub(crate) fn unary(op: &str, value: Self, start: usize) -> Result<Self, &'static str> {
        let end = value.end;
        if op == "binary" {
            let target = Self::new(Kind::CastType, vec![])
                .text("varbinary".into())
                .at(start, start + 6);
            return Self::call(
                vec![Name {
                    text: "cast".into(),
                    quoted: false,
                    start,
                    end: start + 6,
                }],
                None,
                vec![value, target],
                None,
                false,
                start,
                end,
            );
        }
        let operator = match op {
            "+" => ffi::UnaryOperator::Plus,
            "-" => ffi::UnaryOperator::Minus,
            "!" | "not" => ffi::UnaryOperator::LogicalNot,
            "~" => ffi::UnaryOperator::BitwiseNot,
            _ => unreachable!(),
        };
        let mut node = Self::new(Kind::Unary, vec![value]).text(op.into());
        node.operator = Some(Operator::Unary(operator));
        node.expression(start, end)
    }

    pub(crate) fn binary(op: String, left: Self, right: Self) -> Result<Self, &'static str> {
        let (start, end) = (left.start, right.end);
        Self::binary_operands(op, vec![left, right]).expression(start, end)
    }

    pub(crate) fn binary_operands(op: String, operands: Vec<Self>) -> Self {
        let operator = match op.as_str() {
            "+" => ffi::BinaryOperator::Add,
            "-" => ffi::BinaryOperator::Subtract,
            "*" => ffi::BinaryOperator::Multiply,
            "/" => ffi::BinaryOperator::Divide,
            "div" => ffi::BinaryOperator::IntegerDivide,
            "%" | "mod" => ffi::BinaryOperator::Modulo,
            "|" => ffi::BinaryOperator::BitOr,
            "&" => ffi::BinaryOperator::BitAnd,
            "^" => ffi::BinaryOperator::BitXor,
            "<<" => ffi::BinaryOperator::ShiftLeft,
            ">>" => ffi::BinaryOperator::ShiftRight,
            "concat" => ffi::BinaryOperator::Concat,
            "=" => ffi::BinaryOperator::Equal,
            "<=>" => ffi::BinaryOperator::NullSafeEqual,
            "!=" | "<>" => ffi::BinaryOperator::NotEqual,
            "<" => ffi::BinaryOperator::Less,
            "<=" => ffi::BinaryOperator::LessEqual,
            ">" => ffi::BinaryOperator::Greater,
            ">=" => ffi::BinaryOperator::GreaterEqual,
            "and" => ffi::BinaryOperator::LogicalAnd,
            "or" => ffi::BinaryOperator::LogicalOr,
            "xor" => ffi::BinaryOperator::LogicalXor,
            "like" => ffi::BinaryOperator::Like,
            "not like" => ffi::BinaryOperator::NotLike,
            "regexp" => ffi::BinaryOperator::Regexp,
            "not regexp" => ffi::BinaryOperator::NotRegexp,
            _ => unreachable!(),
        };
        let mut node = Self::new(Kind::Binary, operands).text(op);
        node.operator = Some(Operator::Binary(operator));
        node
    }

    pub(crate) fn unary_operator(&self) -> ffi::UnaryOperator {
        match self.operator {
            Some(Operator::Unary(operator)) => operator,
            _ => unreachable!(),
        }
    }

    pub(crate) fn binary_operator(&self) -> ffi::BinaryOperator {
        match self.operator {
            Some(Operator::Binary(operator)) => operator,
            _ => unreachable!(),
        }
    }

    pub(crate) fn is_wildcard(&self) -> bool {
        self.kind == Kind::Star
            || (self.kind == Kind::Column
                && self
                    .children
                    .last()
                    .is_some_and(|child| child.kind == Kind::Star))
    }

    pub(crate) fn cast_array(
        mut self,
        target: Self,
        start: usize,
        end: usize,
    ) -> Result<Self, &'static str> {
        while self.kind == Kind::Parenthesized {
            self = self.children.pop().unwrap();
        }
        let unquoted = self.kind == Kind::Function
            && crate::expression::call_qualifiers(&self).is_empty()
            && crate::expression::call_name(&self).eq_ignore_ascii_case("json_unquote")
            && crate::expression::call_arguments(&self).len() == 1;
        if unquoted {
            self = self
                .into_children()
                .into_iter()
                .find(|child| child.kind == Kind::CallArguments)
                .unwrap()
                .children
                .pop()
                .unwrap();
            while self.kind == Kind::Parenthesized {
                self = self.children.pop().unwrap();
            }
        }
        let mut children = if self.kind == Kind::Column && !unquoted {
            vec![self, Self::new(Kind::HiddenString, vec![])]
        } else if self.kind == Kind::JsonValue {
            self.into_children().into_iter().take(2).collect()
        } else if self.kind == Kind::Function
            && crate::expression::call_qualifiers(&self).is_empty()
            && crate::expression::call_name(&self).eq_ignore_ascii_case("json_extract")
            && crate::expression::call_arguments(&self).len() == 2
        {
            self.into_children()
                .into_iter()
                .find(|child| child.kind == Kind::CallArguments)
                .unwrap()
                .into_children()
        } else {
            return Err("CAST ARRAY requires a JSON column or extracted path");
        };
        children.push(target);
        // JSON_QUERY: truncate, scalars, pretty, ascii, wrapper, asis, error, empty, mismatch, multivalue.
        children.extend(
            [0, 2, 0, 0, 1, 1, 0, 1, 1, 0]
                .into_iter()
                .map(|value| Self::new(Kind::HiddenInt, vec![]).value(value)),
        );
        Self::new(Kind::JsonQuery, children).expression(start, end)
    }

    pub(crate) fn call(
        mut name: Vec<Name>,
        modifier: Option<Kind>,
        mut arguments: Vec<Self>,
        window: Option<Self>,
        followed_by_over: bool,
        start: usize,
        end: usize,
    ) -> Result<Self, &'static str> {
        if name.len() > 1 {
            if name.len() > 3 || modifier.is_some() || window.is_some() {
                return Err("qualified function call");
            }
            return Self::new(
                Kind::Function,
                vec![
                    Self::new(Kind::CallName, name.into_iter().map(Self::name).collect()),
                    Self::new(Kind::CallArguments, arguments),
                ],
            )
            .expression(start, end);
        }
        let mut name = name.pop().unwrap();
        let distinct = modifier == Some(Kind::Distinct);
        let function = if !name.quoted {
            name.text.to_ascii_lowercase()
        } else {
            String::new()
        };
        if matches!(function.as_str(), "any" | "some") {
            return Err("quantified comparison");
        }
        if function == "cast" && (arguments.len() != 2 || arguments[1].kind != Kind::CastType) {
            return Err("CAST requires an expression and AS type");
        }
        if matches!(function.as_str(), "substr" | "substring") {
            return Err("substring syntax");
        }
        if matches!(
            function.as_str(),
            "first_value" | "last_value" | "nth_value" | "lead" | "lag"
        ) {
            return Err("window function requires OVER");
        }
        if matches!(
            function.as_str(),
            "date_add"
                | "date_sub"
                | "adddate"
                | "subdate"
                | "timestampadd"
                | "timestampdiff"
                | "extract"
        ) {
            return Err("date function syntax");
        }
        if function == "mod" {
            if window.is_some() || modifier.is_some() || arguments.len() != 2 {
                return Err("MOD requires two arguments");
            }
            return Self::binary_operands("mod".into(), arguments)
                .expression(start, end);
        }
        let geometry = match function.as_str() {
            "linestring" => Some(Kind::LineString),
            "polygon" => Some(Kind::Polygon),
            "multipoint" => Some(Kind::MultiPoint),
            "multilinestring" => Some(Kind::MultiLineString),
            "multipolygon" => Some(Kind::MultiPolygon),
            "geometrycollection" | "geomcollection" => Some(Kind::GeometryCollection),
            _ => None,
        };
        if let Some(kind) = geometry {
            if modifier.is_some()
                || window.is_some()
                || (kind != Kind::GeometryCollection && arguments.is_empty())
            {
                return Err("geometry constructor arguments");
            }
            return Self::named_call(kind, Self::name(name), arguments).expression(start, end);
        }
        if matches!(function.as_str(), "ln" | "log") {
            if modifier.is_some()
                || window.is_some()
                || arguments.is_empty()
                || arguments.len() > if function == "ln" { 1 } else { 2 }
            {
                return Err("logarithm arguments");
            }
            name.text = "log".into();
            if arguments.len() == 1 {
                arguments.insert(
                    0,
                    Self::new(Kind::HiddenDouble, vec![]).text("2.718281828459045".into()),
                );
            }
        }
        let kind = match function.as_str() {
            "cast" => Kind::Cast,
            "count" => Kind::Count,
            "sum" => Kind::Sum,
            "min" => Kind::Min,
            "max" => Kind::Max,
            "avg" => Kind::Avg,
            "grouping" => Kind::Grouping,
            "json_arrayagg" => Kind::JsonArrayAgg,
            "json_objectagg" => Kind::JsonObjectAgg,
            "std" | "stddev" => Kind::Stddev,
            "stddev_pop" => Kind::StddevPop,
            "stddev_samp" => Kind::StddevSamp,
            "variance" => Kind::Variance,
            "var_pop" => Kind::VarPop,
            "var_samp" => Kind::VarSamp,
            "bit_and" => Kind::BitAnd,
            "bit_or" => Kind::BitOr,
            "bit_xor" => Kind::BitXor,
            "approx_count_distinct" => Kind::ApproxCountDistinct,
            "approx_count_distinct_synopsis" => Kind::ApproxCountDistinctSynopsis,
            "approx_count_distinct_synopsis_merge" => Kind::ApproxCountDistinctSynopsisMerge,
            "sum_opnsize" => Kind::SumOpnsize,
            "top_k_fre_hist" => Kind::TopFreHist,
            "hybrid_hist" => Kind::HybridHist,
            "row_number" => Kind::RowNumber,
            "rank" => Kind::Rank,
            "dense_rank" => Kind::DenseRank,
            "percent_rank" => Kind::PercentRank,
            "cume_dist" => Kind::CumeDist,
            "ntile" => Kind::Ntile,
            "if" => Kind::If,
            "isnull" => Kind::IsNull,
            "now" | "current_timestamp" | "localtime" | "localtimestamp" => Kind::CurrentTimestamp,
            "curtime" | "current_time" => Kind::CurrentTime,
            "curdate" | "current_date" => Kind::CurrentDate,
            "utc_timestamp" => Kind::UtcTimestamp,
            "utc_time" => Kind::UtcTime,
            "utc_date" => Kind::UtcDate,
            "sysdate" => Kind::Sysdate,
            "current_user" => Kind::CurrentUser,
            _ => Kind::Function,
        };
        if matches!(function.as_str(), "substr" | "substring") {
            name.text = "substr".into();
        } else if function == "schema" {
            name.text = "database".into();
        } else if matches!(function.as_str(), "session_user" | "system_user") {
            name.text = "user".into();
        }
        let timestamp = matches!(
            kind,
            Kind::CurrentTimestamp
                | Kind::CurrentTime
                | Kind::UtcTimestamp
                | Kind::UtcTime
                | Kind::Sysdate
        );
        let no_arguments = matches!(kind, Kind::CurrentDate | Kind::UtcDate | Kind::CurrentUser);
        if (kind == Kind::If && arguments.len() != 3)
            || (matches!(kind, Kind::IsNull | Kind::Grouping) && arguments.len() != 1)
            || (no_arguments && !arguments.is_empty())
        {
            return Err("invalid function argument count");
        }
        if timestamp
            && (arguments.len() > 1
                || arguments.first().is_some_and(|arg| {
                    !matches!(arg.kind, Kind::Int | Kind::Uint)
                }))
        {
            return Err("time precision must be an integer");
        }
        if timestamp {
            name.text = match kind {
                Kind::CurrentTimestamp => "now",
                Kind::CurrentTime => "current_time",
                _ => function.as_str(),
            }.into();
        }
        let approximate = matches!(
            kind,
            Kind::ApproxCountDistinct | Kind::ApproxCountDistinctSynopsis
        );
        let histogram = matches!(kind, Kind::TopFreHist | Kind::HybridHist);
        let non_distinct_aggregate = matches!(
            kind,
            Kind::Stddev
                | Kind::StddevPop
                | Kind::StddevSamp
                | Kind::Variance
                | Kind::VarPop
                | Kind::VarSamp
                | Kind::BitAnd
                | Kind::BitOr
                | Kind::BitXor
        );
        let aggregate = approximate
            || histogram
            || non_distinct_aggregate
            || matches!(
                kind,
                Kind::Count
                    | Kind::Sum
                    | Kind::Min
                    | Kind::Max
                    | Kind::Avg
                    | Kind::JsonArrayAgg
                    | Kind::JsonObjectAgg
                    | Kind::ApproxCountDistinctSynopsisMerge
                    | Kind::SumOpnsize
            );
        let ranking = matches!(
            kind,
            Kind::RowNumber
                | Kind::Rank
                | Kind::DenseRank
                | Kind::PercentRank
                | Kind::CumeDist
                | Kind::Ntile
        );
        let supports_window = ranking || aggregate && kind != Kind::SumOpnsize;
        if (ranking && (window.is_none() || arguments.len() != usize::from(kind == Kind::Ntile)))
            || (window.is_some() && !supports_window)
            || (window.is_none() && followed_by_over && supports_window)
        {
            return Err("invalid window function");
        }
        let star = arguments.iter().any(|arg| arg.kind == Kind::Star);
        let count_star = kind == Kind::Count
            && !distinct
            && arguments.len() == 1
            && arguments[0].kind == Kind::Star;
        if star && !count_star {
            return Err("only COUNT(*) accepts a wildcard argument");
        }
        if modifier.is_some()
            && (!aggregate
                || approximate
                || histogram
                || distinct && non_distinct_aggregate
                || matches!(
                    kind,
                    Kind::ApproxCountDistinctSynopsisMerge | Kind::SumOpnsize | Kind::JsonObjectAgg
                ))
        {
            return Err("invalid aggregate modifier");
        }
        let argument_count = match kind {
            Kind::TopFreHist => 4,
            Kind::HybridHist | Kind::JsonObjectAgg => 2,
            _ => 1,
        };
        if aggregate
            && !count_star
            && (arguments.is_empty()
                || (!(approximate || kind == Kind::Count && distinct)
                    && arguments.len() != argument_count))
        {
            return Err("invalid aggregate argument count");
        }
        if (kind == Kind::TopFreHist
            && (arguments[0].kind != Kind::Number || arguments[2].kind != Kind::Int))
            || (kind == Kind::HybridHist && arguments[1].kind != Kind::Int)
        {
            return Err("histogram rate and bucket count must be numeric constants");
        }
        let mut function = Self::named_call(kind, Self::name(name), arguments);
        if let Some(modifier) = modifier {
            function.children.push(Self::new(modifier, vec![]));
        }
        match window {
            Some(window) => Self::new(Kind::WindowFunction, vec![function, window]),
            None => function,
        }
        .expression(start, end)
    }

    pub(crate) fn callable(kind: Kind, arguments: Vec<Self>) -> Self {
        Self::new(kind, vec![Self::new(Kind::CallArguments, arguments)])
    }

    pub(crate) fn named_call(kind: Kind, name: Self, arguments: Vec<Self>) -> Self {
        Self::new(
            kind,
            vec![
                Self::new(Kind::CallName, vec![name]),
                Self::new(Kind::CallArguments, arguments),
            ],
        )
    }

    pub(crate) fn relation(names: Vec<Name>) -> Result<Self, &'static str> {
        if names.len() > 2 {
            return Err("table names have at most two parts");
        }
        let text = names.last().unwrap().text.clone();
        let mut children: Vec<_> = (names.len()..2).map(|_| Self::missing()).collect();
        children.extend(names.into_iter().map(Self::name));
        Ok(Self::new(Kind::Relation, children).text(text))
    }

    pub(crate) fn table(source: Self, alias: Option<Name>) -> Result<Self, &'static str> {
        let mut children = vec![source];
        if let Some(alias) = alias {
            children.push(Self::new(Kind::TableAlias, vec![Self::name(alias)]));
        }
        Ok(Self::new(Kind::Table, children))
    }

    pub(crate) fn projection(mut value: Self, alias: Option<Self>) -> Self {
        let (start, end) = (value.start, value.end);
        if let Some(mut alias) = alias {
            let text = alias.text.clone();
            let string = alias.kind == Kind::String;
            let span = if string {
                (alias.start, alias.end)
            } else {
                (start, end)
            };
            alias.kind = Kind::Ident;
            alias.children.clear();
            value = Self::new(
                if string {
                    Kind::StringAlias
                } else {
                    Kind::Alias
                },
                vec![value, alias],
            )
            .text(text)
            .at(span.0, span.1);
        }
        Self::new(Kind::Projection, vec![value]).at(start, end)
    }

    pub(crate) fn select(query: Box<Select>) -> Result<Self, &'static str> {
        if query.into.is_some() && query.into_end.is_some() {
            return Err("one INTO clause");
        }
        let mut children = Vec::new();
        if !query.options.is_empty() {
            children.push(Self::new(Kind::SelectOptions, query.options));
        }
        if !query.projection.is_empty() {
            children.push(Self::new(Kind::ProjectList, query.projection));
        }
        children.extend(query.into);
        if !query.from.is_empty() {
            children.push(Self::new(Kind::FromList, query.from));
        }
        if let Some(filter) = query.filter {
            children.push(Self::new(Kind::Where, vec![filter]));
        }
        children.extend(query.group_by);
        if let Some(having) = query.having {
            children.push(Self::new(Kind::Having, vec![having]));
        }
        if !query.windows.is_empty() {
            children.push(Self::new(Kind::NamedWindows, query.windows));
        }
        children.extend(query.set);
        if !query.order_by.is_empty() {
            children.push(Self::new(Kind::Order, query.order_by));
        }
        children.extend(query.limit);
        if let Some(lock) = query.lock {
            children.push(Self::new(Kind::QueryLock, vec![lock]));
        }
        if !query.hints.is_empty() {
            children.push(Self::new(Kind::Hints, query.hints));
        }
        if let Some(into) = query.into_end {
            let kind = match into.kind {
                Kind::IntoVariables => Kind::TrailingIntoVariables,
                Kind::IntoOutfile => Kind::TrailingIntoOutfile,
                Kind::IntoDumpfile => Kind::TrailingIntoDumpfile,
                _ => unreachable!(),
            };
            children.push(into.with_kind(kind));
        }
        children.extend(query.approximate);
        children.extend(query.vector_parameters);
        children.extend(query.with);
        Self::new(Kind::Select, children).expression(0, 0)
    }
}
