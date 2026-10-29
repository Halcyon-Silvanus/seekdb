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

#ifndef OCEANBASE_SQL_PARSER_FAST_PARSER_
#define OCEANBASE_SQL_PARSER_FAST_PARSER_

#include "lib/allocator/ob_allocator.h"
#include "lib/string/ob_string.h"
#include "lib/charset/ob_charset.h"
#include "sql/parser/ob_parser_utils.h"
#include "sql/plan_cache/ob_plan_cache_struct.h"

namespace oceanbase
{
namespace sql
{
struct FPContext
{
public:
	bool enable_batched_multi_stmt_;
	ObCharsets4Parser charsets4parser_;
	ObSQLMode sql_mode_;
  bool is_format_;
  bool parameterize_ = true;
  bool ignore_hint_ = false;

	FPContext()
		: enable_batched_multi_stmt_(false),
			sql_mode_(0),
      is_format_(false)
	{}
	FPContext(ObCharsets4Parser charsets4parser)
		: enable_batched_multi_stmt_(false),
			charsets4parser_(charsets4parser),
			sql_mode_(0),
      is_format_(false)
	{}
};

struct ObFastParser final
{
public:
  static int parse(const common::ObString &stmt,
                   const FPContext &context,
                   common::ObIAllocator &allocator,
                   ObFastParserResult &result);
  static int compress_insert(common::ObIAllocator &allocator,
                                        int64_t values_token_pos,
                                        const ObString &old_no_param_sql,
                                        ObString &new_truncated_sql,
                                        bool &can_batch_opt,
                                        int64_t &params_count,
                                        int64_t &upd_params_count,
                                        int64_t &lenth_delta,
                                        int64_t &row_count,
                                        bool &is_insert_values);

};

} // namespace sql
} // namespace oceanbase

#endif
