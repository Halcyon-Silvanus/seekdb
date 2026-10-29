/*
 * Copyright (c) 2025 OceanBase.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#define USING_LOG_PREFIX SQL_PARSER
#include "sql/parser/rust_sql_parser.h"
#include "sql/parser/parsed_sql.h"
#include "ob_parser.h"
#include "lib/utility/ob_smart_call.h"
#include "lib/oblog/ob_log.h"
#include "parse_node.h"
#include "ob_fast_parser.h"
#include "rust/sql_parser.h"
#include "lib/container/ob_se_array.h"
#include "lib/string/ob_sql_string.h"
#include <algorithm>
using namespace oceanbase::sql;
using namespace oceanbase::common;

ObParser::ObParser(common::ObIAllocator &allocator,
                   ObSQLMode mode,
                   ObCharsets4Parser charsets4parser)
    :allocator_(&allocator),
     sql_mode_(mode),
     charsets4parser_(charsets4parser)
{}

ObParser::~ObParser()
{}

namespace {
rust::Slice<const uint8_t> sql_bytes(const ObString &sql)
{
  return rust::Slice<const uint8_t>(reinterpret_cast<const uint8_t *>(sql.ptr()), sql.length());
}

peg::SqlMode parser_mode(ObSQLMode mode)
{
  return peg::SqlMode{bool(mode & SMO_ANSI_QUOTES), bool(mode & SMO_NO_BACKSLASH_ESCAPES),
                      bool(mode & SMO_PIPES_AS_CONCAT), bool(mode & SMO_HIGH_NOT_PRECEDENCE)};
}
}

bool ObParser::is_pl_stmt(const ObString &stmt, bool *is_create_func, bool *is_call_procedure)
{
  const auto info = peg::classify_statement(sql_bytes(stmt), peg::SqlMode{});
  if (is_create_func != nullptr) *is_create_func = info.pl == peg::PlPrefix::Function;
  if (is_call_procedure != nullptr) *is_call_procedure = info.pl == peg::PlPrefix::Call;
  return info.pl != peg::PlPrefix::None;
}

bool ObParser::is_explain_stmt(const ObString &stmt, const char *&body)
{
  const auto info = peg::classify_statement(sql_bytes(stmt), peg::SqlMode{});
  const bool explained = info.kind == peg::StatementClass::Explain
      && info.explain_offset < static_cast<size_t>(stmt.length());
  body = explained ? stmt.ptr() + info.explain_offset : nullptr;
  return explained;
}

bool ObParser::is_single_stmt(const ObString &stmt)
{
  return peg::is_single_statement(sql_bytes(stmt));
}

int ObParser::split_multiple_stmt(const ObString &stmt,
                                  ObIArray<ObString> &queries,
                                  ObMPParseStat &parse_stat,
                                  bool is_ret_first_stmt,
                                  bool is_prepare)
{
  UNUSED(is_prepare);
  int ret = OB_SUCCESS;
  parse_stat.reset();
  const auto split = peg::split_sql(sql_bytes(stmt), parser_mode(sql_mode_), is_ret_first_stmt);
  size_t offset = 0;
  for (size_t end : split.ends) {
    if (OB_FAIL(queries.push_back(ObString(end - offset, stmt.ptr() + offset)))) break;
    offset = end;
  }
  if (OB_SUCC(ret) && split.failed) {
    parse_stat.parse_fail_ = true;
    parse_stat.fail_query_idx_ = queries.count() - 1;
    parse_stat.fail_ret_ = OB_ERR_PARSE_SQL;
  }
  return ret;
}

int ObParser::parse_sql(const ObString &stmt,
                        ParseResult &parse_result,
                        const bool no_throw_parser_error)
{
  FPContext context(charsets4parser_);
  context.sql_mode_ = sql_mode_;
  context.enable_batched_multi_stmt_ = parse_result.is_batched_multi_enabled_split_;
  context.parameterize_ = parse_result.need_parameterize_;
  context.ignore_hint_ = parse_result.is_ignore_hint_;
  ObFastParserResult fast;
  int ret = ObFastParser::parse(stmt, context, *allocator_, fast);
  parse_result.no_param_sql_ = fast.pc_key_.name_.ptr();
  parse_result.no_param_sql_len_ = fast.pc_key_.name_.length();
  parse_result.question_mark_ctx_ = fast.question_mark_ctx_;
  parse_result.input_sql_ = stmt.ptr();
  parse_result.input_sql_len_ = stmt.length();
  if (ret == OB_ERR_PARSER_SYNTAX) ret = OB_ERR_PARSE_SQL;
  return ret;
}

int ObParser::parse_(const ObString &query,
                    ParseResult &parse_result,
                    ParseMode parse_mode,
                    const bool is_batched_multi_stmt_split_on,
                    const bool no_throw_parser_error,
                    const bool is_pl_inner_parse,
                    const bool is_parse_dynamic_sql,
                    ParsedSql *owner)
{
  int ret = OB_SUCCESS;
  const int64_t len = peg::normalized_sql_length(sql_bytes(query));
  ObString stmt(len, query.ptr());
  memset(&parse_result, 0, sizeof(ParseResult));
  parse_result.is_method_opt_parser_ = (METHOD_OPT_MODE == parse_mode);
  parse_result.is_fp_ = (FP_MODE == parse_mode
                         || FP_PARAMERIZE_AND_FILTER_HINT_MODE == parse_mode
                         || FP_NO_PARAMERIZE_AND_FILTER_HINT_MODE== parse_mode);
  parse_result.is_ignore_hint_ = (FP_PARAMERIZE_AND_FILTER_HINT_MODE == parse_mode
                                  || FP_NO_PARAMERIZE_AND_FILTER_HINT_MODE == parse_mode);
  parse_result.is_for_trigger_ = (TRIGGER_MODE == parse_mode);
  parse_result.is_dynamic_sql_ = (DYNAMIC_SQL_MODE == parse_mode);
  parse_result.is_batched_multi_enabled_split_ = is_batched_multi_stmt_split_on;
  parse_result.malloc_pool_ = allocator_;
  parse_result.sql_mode_ = sql_mode_;
  parse_result.need_parameterize_ = (FP_MODE == parse_mode
                         || FP_PARAMERIZE_AND_FILTER_HINT_MODE == parse_mode);
  parse_result.charset_info_ = ObCharset::get_charset(charsets4parser_.string_collation_);


  if (OB_SUCC(ret) && stmt.empty()) {
    ret = OB_ERR_EMPTY_QUERY;
  }

  if (OB_SUCC(ret) && OB_ISNULL(parse_result.charset_info_)) {
    ret = OB_ERR_UNEXPECTED;
    LOG_WARN("charset info is null", K(ret),
             "connection charset", ObCharset::charset_name(charsets4parser_.string_collation_));
  } else {
    LOG_DEBUG("check parse_result param",
              "connection charset", ObCharset::charset_name(charsets4parser_.string_collation_));
  }
  if (OB_SUCC(ret) && (parse_mode == STD_MODE || parse_mode == DYNAMIC_SQL_MODE
      || parse_mode == TRIGGER_MODE || parse_mode == METHOD_OPT_MODE)) {
    return parse_rust_sql(stmt, parse_result, false, owner);
  }
  if (OB_SUCC(ret)) {
    const auto info = peg::classify_statement(sql_bytes(stmt), parser_mode(sql_mode_));
    if (info.pl == peg::PlPrefix::None || info.pl == peg::PlPrefix::Call) {
      ret = parse_sql(stmt, parse_result, no_throw_parser_error);
    } else {
      parse_result.no_param_sql_ = const_cast<char *>(stmt.ptr());
      parse_result.no_param_sql_len_ = stmt.length();
      parse_result.end_col_ = stmt.length();
    }
  }
  return ret;
}

int ObParser::parse(const ObString &query,
                    ParseResult &parse_result,
                    ParseMode parse_mode,
                    const bool is_batched_multi_stmt_split_on,
                    const bool no_throw_parser_error,
                    const bool is_pl_inner_parse,
                    const bool is_parse_dynamic_sql)
{
  return SMART_CALL(parse_(query,
                          parse_result,
                          parse_mode,
                          is_batched_multi_stmt_split_on,
                          no_throw_parser_error,
                          is_pl_inner_parse,
                          is_parse_dynamic_sql));
}

int ObParser::parse(const ObString &query, ParsedSql &parsed, ParseMode mode,
                    bool split, bool no_throw, bool pl_inner, bool dynamic_sql)
{
  parsed.reset();
  return SMART_CALL(parse_(query, parsed.metadata(), mode, split, no_throw,
                          pl_inner, dynamic_sql, &parsed));
}

int ObParser::parse_routine_definition(const ObString &definition, ParsedSql &parsed)
{
  int ret = OB_SUCCESS;
  constexpr char prefix[] = "CREATE\n";
  const int64_t prefix_length = sizeof(prefix) - 1;
  const int64_t length = prefix_length + definition.length();
  char *source = static_cast<char *>(allocator_->alloc(length));
  if (OB_ISNULL(source)) {
    ret = OB_ALLOCATE_MEMORY_FAILED;
  } else {
    MEMCPY(source, prefix, prefix_length);
    MEMCPY(source + prefix_length, definition.ptr(), definition.length());
    OZ (parse(ObString(length, source), parsed));
    if (OB_SUCC(ret) && (parsed.statement() == nullptr
        || (parsed.statement()->kind != peg::NodeKind::CreateProcedure
            && parsed.statement()->kind != peg::NodeKind::CreateFunction))) {
      ret = OB_ERR_UNEXPECTED;
    }
  }
  return ret;
}

int ObParser::pre_parse(const ObString &stmt, PreParseResult &res)
{
  const auto info = peg::classify_statement(sql_bytes(stmt), peg::SqlMode{});
  res.trace_id_.reset();
  if (info.trace_end > info.trace_start) {
    const size_t length = std::min(info.trace_end - info.trace_start,
                                  static_cast<size_t>(OB_MAX_TRACE_ID_BUFFER_SIZE));
    res.trace_id_.assign_ptr(stmt.ptr() + info.trace_start, length);
  }
  return OB_SUCCESS;
}

void ObParser::free_result(ParseResult &parse_result)
{
  UNUSED(parse_result);
}

int ObParser::parse_expression(const ObString &text, ParsedSql &parsed, bool is_for_trigger)
{
  parsed.reset();
  ParseResult &result = parsed.metadata();
  result.malloc_pool_ = allocator_;
  result.sql_mode_ = sql_mode_;
  result.charset_info_ = ObCharset::get_charset(charsets4parser_.string_collation_);
  result.is_expression_ = true;
  result.is_for_trigger_ = is_for_trigger;
  return parse_rust_sql(text, result, false, &parsed);
}
