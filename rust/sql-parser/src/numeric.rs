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

use crate::bridge::ffi::{LiteralKind, ParameterLoweringKind, UnaryOperator};
use crate::expression::{expression_literal, is_literal, parenthesized_expression, unary_operand, unary_operator};
use crate::node::{Kind, Node};
use crate::parameterize::ParameterEffects;

pub(crate) struct UnaryPath<'a> {
    operand: &'a Node,
}

pub(crate) struct UnaryStep<'a> {
    pub node: &'a Node,
    pub operator: Option<UnaryOperator>,
    pub operand: &'a Node,
}

impl<'a> UnaryPath<'a> {
    pub fn new(node: &'a Node) -> Self {
        Self { operand: node }
    }

    pub fn operand(&self) -> &'a Node {
        self.operand
    }
}

impl<'a> Iterator for UnaryPath<'a> {
    type Item = UnaryStep<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.operand;
        let (operator, operand) = match node.kind {
            Kind::Parenthesized => (None, parenthesized_expression(node)),
            Kind::Unary if matches!(unary_operator(node), UnaryOperator::Plus | UnaryOperator::Minus) => {
                (Some(unary_operator(node)), unary_operand(node))
            }
            _ => return None,
        };
        self.operand = operand;
        Some(UnaryStep { node, operator, operand })
    }
}

pub(crate) enum NegativeOperand<'a> {
    DirectLiteral(&'a Node),
    ParenthesizedUnsigned(&'a Node),
    Other,
}

pub(crate) fn negative_operand(mut operand: &Node) -> NegativeOperand<'_> {
    if is_literal(operand) {
        return NegativeOperand::DirectLiteral(operand);
    }
    while operand.kind == Kind::Parenthesized {
        operand = parenthesized_expression(operand);
    }
    if operand.kind == Kind::Uint {
        NegativeOperand::ParenthesizedUnsigned(operand)
    } else {
        NegativeOperand::Other
    }
}

pub(crate) fn negative_unsigned_kind(integer: u64) -> LiteralKind {
    if integer == 1 << 63 { LiteralKind::Integer } else { LiteralKind::Decimal }
}

pub enum UnaryOperand<'a> {
    Resolve(&'a Node),
    Literal { kind: LiteralKind, integer: u64, text: &'a [u8] },
}

pub struct UnaryResolution<'a> {
    pub operand: UnaryOperand<'a>,
    pub negations: usize,
}

pub fn resolve_unary<'a>(
    node: &'a Node,
    syntax_sql: &'a [u8],
    effects: Option<&ParameterEffects>,
) -> UnaryResolution<'a> {
    let mut operand = node;
    let mut last_sign = None;
    let mut negations = 0;
    for step in UnaryPath::new(node) {
        if !std::ptr::eq(step.node, node)
            && effects.is_some_and(|effects| {
                effects.binding(step.node).is_some()
                    || effects.lowering(step.node).is_some_and(|lowering| {
                        lowering.operation == ParameterLoweringKind::FoldUnaryExpression
                    })
            })
        {
            break;
        }
        if let Some(operator) = step.operator {
            last_sign = Some((step.node, operator, step.operand));
            negations += usize::from(operator == UnaryOperator::Minus);
        }
        operand = step.operand;
    }
    let negative = negative_operand(operand);
    let literal = match negative {
        NegativeOperand::DirectLiteral(literal) if literal.kind == Kind::Uint => Some(literal),
        NegativeOperand::ParenthesizedUnsigned(literal) => Some(literal),
        _ => None,
    };
    if let (Some((sign, UnaryOperator::Minus, sign_operand)), Some(literal)) = (last_sign, literal) {
        if !effects.is_some_and(|effects| effects.binding(literal).is_some() || effects.lowering(literal).is_some()) {
            let value = expression_literal(literal);
            let (kind, integer, text) = if std::ptr::eq(sign_operand, literal)
                && negative_unsigned_kind(value.integer) == LiteralKind::Integer
            {
                negations -= 1;
                (LiteralKind::Integer, value.integer, &syntax_sql[sign.start..sign.end])
            } else {
                (LiteralKind::Decimal, 0, value.text.as_bytes())
            };
            return UnaryResolution { operand: UnaryOperand::Literal { kind, integer, text }, negations };
        }
    }
    UnaryResolution { operand: UnaryOperand::Resolve(operand), negations }
}
