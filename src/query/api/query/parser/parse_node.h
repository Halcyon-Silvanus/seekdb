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

#ifndef OCEANBASE_SQL_PARSER_PARSE_NODE_H_
#define OCEANBASE_SQL_PARSER_PARSE_NODE_H_

#include <assert.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdbool.h>
#include "common/sql_mode/ob_sql_mode.h"
#include "query/parser/ob_item_type.h"

#ifdef __cplusplus
extern "C" {
#endif

#define MAX_ERROR_MSG 1024

struct ObCharsetInfo;

enum ParseMode
{
  STD_MODE = 0,
  FP_MODE, /* fast parse, retain hint, and do parameterization */
  FP_PARAMERIZE_AND_FILTER_HINT_MODE,/*Filter out hint, and do parameterization*/
  FP_NO_PARAMERIZE_AND_FILTER_HINT_MODE,/*Filter out hint, and do not parameterize*/
  TRIGGER_MODE, /* treat ':xxx' as identifier */
  DYNAMIC_SQL_MODE, /*Parse dynamic SQL, :idx and :identifier need to be determined whether to check the placeholder name based on the statement type*/
  DBMS_SQL_MODE,
  METHOD_OPT_MODE,
};

typedef struct
{
  int err_code_;
  char err_msg_[MAX_ERROR_MSG];
} ErrStat;

typedef struct _ObStmtLoc
{
  int first_column_;
  int last_column_;
  int first_line_;
  int last_line_;
} ObStmtLoc;

// Discuss with @Ruidian, the definition here will be changed to dynamic later, define 128 for now
#define MAX_QUESTION_MARK 128

typedef struct _ObQuestionMarkCtx
{
  char **name_;
  int count_;
  int capacity_;
  bool by_ordinal_;
  bool by_name_;
  bool by_defined_name_;
} ObQuestionMarkCtx;

typedef struct ObParseResult
{
  const char *input_sql_;
  int input_sql_len_;
  void *malloc_pool_; // ObIAllocator
  ObQuestionMarkCtx question_mark_ctx_;
  ObSQLMode sql_mode_;
  const struct ObCharsetInfo *charset_info_; //client charset
  struct {
    uint32_t is_fp_                            : 1;
    uint32_t is_ignore_hint_                   : 1;//used for outline
    uint32_t need_parameterize_                : 1;//used for outline, to support signature of outline can contain hint
    uint32_t is_for_trigger_                   : 1;
    uint32_t is_dynamic_sql_                   : 1;
    uint32_t is_batched_multi_enabled_split_   : 1;
    uint32_t contain_sensitive_data_           : 1;
    uint32_t is_expression_                    : 1;
    uint32_t is_method_opt_parser_             : 1;
  };

  char *error_msg_;
  int start_col_;
  int end_col_;
  int line_;
  char *no_param_sql_;
  int no_param_sql_len_;

} ParseResult;

extern char *str_tolower(char *buff, int64_t len);
extern char *str_toupper(char *buff, int64_t len);

/// convert x'42ab' to binary string
void ob_parse_binary(const char *src, int64_t len, char* dest);
int64_t ob_parse_binary_len(int64_t len);

// convert b'10010110' to binary string
// @pre dest buffer is enough
void ob_parse_bit_string(const char* src, int64_t len, char* dest);
int64_t ob_parse_bit_string_len(int64_t len);

extern int64_t get_question_mark(ObQuestionMarkCtx *ctx, void *malloc_pool, const char *name);

#ifdef __cplusplus
}
#endif

#endif //OCEANBASE_SQL_PARSER_PARSE_NODE_H_
