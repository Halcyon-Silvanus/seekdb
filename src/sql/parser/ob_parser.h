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

#ifndef _OB_PARSER_H
#define _OB_PARSER_H 1
#include "sql/parser/parse_node.h"
#include "rust/sql_parser.h"
#include "sql/parser/parse_define.h"
#include "lib/allocator/ob_allocator.h"
#include "lib/container/ob_iarray.h"
#include "lib/string/ob_string.h"
#include "lib/charset/ob_charset.h"
#include "sql/parser/ob_parser_utils.h"

namespace oceanbase
{
namespace sql
{
class ObSQLSessionInfo;
class ObMPParseStat
{
public:
  ObMPParseStat() : parse_fail_(false), fail_ret_(common::OB_SUCCESS), fail_query_idx_(-1) {}
  void reset()
  {
    parse_fail_ = false;
    fail_ret_ = common::OB_SUCCESS;
    fail_query_idx_ = -1;
  }
  bool parse_fail_;
  int fail_ret_;
  int64_t fail_query_idx_;
};

struct PreParseResult
{
  common::ObString trace_id_;
};

class ParsedSql;

class ObParser
{
public:
  explicit ObParser(common::ObIAllocator &allocator, ObSQLMode mode,
                    ObCharsets4Parser charsets4parser = ObCharsets4Parser());
  virtual ~ObParser();
  /// @param queries Note that all three members of ObString is valid, size() is the length
  ///                of the single statement, length() is the length of remainer statements
  bool is_single_stmt(const common::ObString &stmt);
  int split_multiple_stmt(const common::ObString &stmt,
                          common::ObIArray<common::ObString> &queries,
                          ObMPParseStat &parse_fail,
                          bool is_ret_first_stmt=false,
                          bool is_prepare = false);

  //@param:
  //  no_throw_parser_error is used to mark not throw parser error. in the split multi stmt
  //  situation we will try find ';' delimiter to parser part of string in case of save memory,
  //  but this maybe parser error and throw error info. However, we will still try parser remain
  //  string when parse part of string failed, if we throw parse part error info, maybe will let
  //  someone misunderstand have bug, So, we introduce this mark to decide to throw parser error.
  //  eg: select '123;' from dual; select '123' from dual;
  int parse_sql(const common::ObString &stmt,
                ParseResult &parse_result,
                const bool no_throw_parser_error);

private:
  int parse_(const common::ObString &stmt,
                    ParseResult &parse_result,
                    ParseMode mode=STD_MODE,
                    const bool is_batched_multi_stmt_split_on = false,
                    const bool no_throw_parser_error = false,
                    const bool is_pl_inner_parse = false,
                    const bool is_parser_dynamic_sql = false,
                    ParsedSql *owner = nullptr);
public:
  virtual int parse(const common::ObString &stmt,
                    ParseResult &parse_result,
                    ParseMode mode=STD_MODE,
                    const bool is_batched_multi_stmt_split_on = false,
                    const bool no_throw_parser_error = false,
                    const bool is_pl_inner_parse = false,
                    const bool is_parser_dynamic_sql = false);

  int parse(const common::ObString &stmt, ParsedSql &parsed,
            ParseMode mode = STD_MODE,
            bool is_batched_multi_stmt_split_on = false,
            bool no_throw_parser_error = false,
            bool is_pl_inner_parse = false,
            bool is_parser_dynamic_sql = false);

  int parse_routine_definition(const common::ObString &definition, ParsedSql &parsed);

  virtual void free_result(ParseResult &parse_result);
  int parse_expression(const common::ObString &text, ParsedSql &parsed,
                       bool is_for_trigger = false);
  static int pre_parse(const common::ObString &stmt,
                       PreParseResult &res);

  static bool is_pl_stmt(const common::ObString &stmt,
                         bool *is_create_func = nullptr,
                         bool *is_call_procedure = nullptr);
  static bool is_explain_stmt(const common::ObString &stmt,
                              const char *&body);
  // types and constants
private:
  // disallow copy
  DISALLOW_COPY_AND_ASSIGN(ObParser);
  // function members
private:
  // data members
  common::ObIAllocator *allocator_;
  // when sql_mode = "ANSI_QUOTES", Treat " as an identifier quote character
  // we don't use it in parser now
  ObSQLMode sql_mode_;

  ObCharsets4Parser charsets4parser_;
};

} // end namespace sql
} // end namespace oceanbase

#endif /* _OB_PARSER_H */
