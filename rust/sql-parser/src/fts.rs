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

use crate::bridge::ffi::{SearchNode, SearchNodeKind as Kind, SearchTree};
use crate::node::MAX_DEPTH;

impl SearchNode {
    fn new(kind: Kind, children: Vec<Self>) -> Self {
        Self {
            kind,
            text: String::new(),
            children,
        }
    }
}

peg::parser! {
    grammar search() for str {
        // Tabs inside a token remain part of it, matching the existing search tokenizer.
        rule separator() = ([' '|'\n'] / "\t"+ [' '|'\n']) [' '|'\t'|'\n']*
            / "\t"+ &(['"'|'*'|'('|')'|'+'|'-'|'<'|'>'|'~'|'@'] / ![_])
            / "\""
        rule _() = separator()*
        rule depth(level: usize) = {? if level < MAX_DEPTH { Ok(()) } else { Err("search expression depth below 128 levels") } }
        rule terms(level: usize) -> Vec<SearchNode> = values:(value:expression(level) _ { value })* {
            values.into_iter().flatten().collect()
        }
        rule expression(level: usize) -> Option<SearchNode> = depth(level)
            prefix:("+" { Kind::Required } / "-" { Kind::Excluded })? _
            value:("(" _ values:terms(level+1) ")" {
                if values.is_empty() { None }
                else { Some(SearchNode::new(Kind::Group,vec![SearchNode::new(Kind::List,values)])) }
            } / text:$([^'"'|' '|'\n'|'*'|'('|')'|'+'|'-'|'<'|'>'|'~'|'@']+) {
                Some(SearchNode { kind:Kind::Term,text:text.into(),children:vec![] })
            }) {
                value.map(|value| match prefix {
                    Some(kind) => SearchNode::new(Kind::List,vec![SearchNode::new(kind,vec![]),value]),
                    None => value,
                })
            }
        pub rule query() -> Vec<SearchNode> = _ values:terms(0) { values }
    }
}

pub fn parse_search_query(input: &[u8]) -> SearchTree {
    let parsed = std::str::from_utf8(input)
        .map_err(|error| error.to_string())
        .and_then(|input| search::query(input).map_err(|error| error.to_string()));
    match parsed {
        Ok(children) => SearchTree {
            root: SearchNode::new(Kind::List, children),
            error: String::new(),
        },
        Err(error) => SearchTree {
            root: SearchNode::new(Kind::List, vec![]),
            error,
        },
    }
}
