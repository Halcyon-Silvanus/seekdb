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

#define USING_LOG_PREFIX SQL_RESV
#include "sql/resolver/cmd/ob_show_resolver.h"
#include "sql/resolver/ob_resolver_utils.h"
#include "sql/resolver/cmd/ob_show_table_status_sql.h"
#include "sql/printer/ob_schema_printer.h"

using namespace oceanbase::common;
using namespace oceanbase::share;
using namespace oceanbase::share::schema;
namespace oceanbase
{
namespace sql
{

ObShowResolver::ObShowResolver(ObResolverParams &params)
    : ObSelectResolver(params)
{
  params_.is_from_show_resolver_ = true;
}

ObShowResolver::~ObShowResolver()
{
}

#define GEN_SQL_STEP_1(SHOW_STMT_TYPE, args...)                         \
  do {                                                                  \
    if (OB_SUCC(ret)) {                                                 \
      show_resv_ctx.like_column_ = SHOW_STMT_TYPE##_LIKE == NULL ? ObString::make_string("") : \
          ObString::make_string(SHOW_STMT_TYPE##_LIKE);                 \
      if (OB_SUCC(ret) && OB_FAIL(sql_gen.init(params_.allocator_))) {        \
        LOG_WARN("fail to init sql string generator", K(ret));          \
      } else if (OB_FAIL(sql_gen.gen_select_str(SHOW_STMT_TYPE##_SELECT, ##args))) { \
        LOG_WARN("fail to generate select string", K(ret));             \
      } else { /*do nothing*/ }                                         \
    }                                                                   \
  } while(0)

#define GEN_SQL_STEP_2(SHOW_STMT_TYPE, args...)                         \
  do {                                                                  \
    if (OB_SUCC(ret)) {                                                 \
      if (OB_FAIL(sql_gen.gen_from_str(SHOW_STMT_TYPE##_SUBQUERY, ##args))) { \
        LOG_WARN("fail to generate from string", K(ret));               \
      } else {                                                          \
        sql_gen.assign_sql_str(select_sql);                             \
      }                                                                 \
    }                                                                   \
  } while(0)

int ObShowResolver::check_db_access_for_show_sql(const ObShowResolverContext &show_resv_ctx,
                                                 ObSessionPrivInfo &session_priv,
                                                 const common::ObIArray<uint64_t> &enable_role_id_array)
{
  int ret = OB_SUCCESS;
  ObString show_db_name = show_resv_ctx.show_database_name_;
  if (OB_FAIL(schema_checker_->check_db_access(session_priv, enable_role_id_array, show_db_name))) {
    if (OB_ERR_NO_DB_PRIVILEGE == ret) {
      LOG_USER_ERROR(OB_ERR_NO_DB_PRIVILEGE, session_priv.user_name_.length(), session_priv.user_name_.ptr(),
                      session_priv.host_name_.length(),session_priv.host_name_.ptr(),
                      show_db_name.length(), show_db_name.ptr());
    } else {
    }
  } else { /* do nothing */ }
  return ret;
}

int ObShowResolver::resolve(const peg::Node &statement)
{
  int ret = OB_SUCCESS;
  ObShowResolverContext show_resv_ctx;
  ObString select_sql;
  ObSessionPrivInfo session_priv;
  if (OB_ISNULL(session_info_) || OB_ISNULL(allocator_) || OB_ISNULL(schema_checker_)
      || OB_ISNULL(schema_checker_->get_schema_guard())) ret = OB_NOT_INIT;
  else OZ(session_info_->get_session_priv_info(session_priv));
  if (OB_FAIL(ret)) return ret;
  const auto &roles = session_info_->get_enable_role_array();
  const ObString database_name = session_info_->get_database_name();
  show_resv_ctx.parse_tree_ = &statement;
  ObSqlStrGenerator sql_gen;
  ObArenaAllocator privilege_allocator;
  ObStmtNeedPrivs required;
  required.need_privs_.set_allocator(&privilege_allocator);
  switch (statement.kind()) {
    case peg::NodeKind::ShowTables: {
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_TABLES;
      uint64_t database_id = OB_INVALID_ID;
      OZ(get_database_info(peg::show_has_database(statement) ? &peg::show_database(statement) : nullptr,
          database_name, show_resv_ctx, database_id));
      OZ(check_db_access_for_show_sql(show_resv_ctx, session_priv, roles));
      const ObString &name = show_resv_ctx.show_database_name_;
      const bool full = peg::show_full(statement);
      if (peg::show_filter(statement) == peg::ShowFilter::Like) {
        const auto pattern = peg::expression_literal(peg::show_like_pattern(statement)).text;
        if (full) {
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_FULL_TABLES_LIKE, name.length(), name.ptr(), int(pattern.size()), pattern.data());
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_FULL_TABLES_LIKE, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_TABLES_TNAME, database_id);
        } else {
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_TABLES_LIKE, name.length(), name.ptr(), int(pattern.size()), pattern.data());
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_TABLES_LIKE, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_TABLES_TNAME, database_id);
        }
      } else {
        if (full) {
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_FULL_TABLES, name.length(), name.ptr());
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_FULL_TABLES, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_TABLES_TNAME, database_id);
        } else {
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_TABLES, name.length(), name.ptr());
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_TABLES, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_TABLES_TNAME, database_id);
        }
        if (OB_SUCC(ret) && peg::show_filter(statement) == peg::ShowFilter::Where) {
          ObSqlString alias;
          OZ(alias.append_fmt("tables_in_%.*s", name.length(), name.ptr()));
          OZ(ob_write_string(*allocator_, alias.string(), show_resv_ctx.column_name_));
        }
      }
      break;
    }
    case peg::NodeKind::ShowTableStatus: {
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_TABLE_STATUS;
      uint64_t database_id = OB_INVALID_ID;
      OZ(get_database_info(peg::show_has_database(statement) ? &peg::show_database(statement) : nullptr,
          database_name, show_resv_ctx, database_id));
      OZ(check_db_access_for_show_sql(show_resv_ctx, session_priv, roles));
      GEN_SQL_STEP_1(ObShowSqlSet::SHOW_TABLE_STATUS);
      GEN_SQL_STEP_2(ObShowSqlSet::SHOW_TABLE_STATUS, NEW_TABLE_STATUS_SQL, database_id);
      break;
    }
    case peg::NodeKind::ShowDatabases: {
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_DATABASES;
      if (peg::show_filter(statement) == peg::ShowFilter::Like) {
        const auto pattern = peg::expression_literal(peg::show_like_pattern(statement)).text;
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_DATABASES_LIKE, int(pattern.size()), pattern.data());
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_DATABASES_LIKE, OB_SYS_DATABASE_NAME, OB_ALL_DATABASE_TNAME,
            OB_RECYCLEBIN_SCHEMA_NAME, OB_PUBLIC_SCHEMA_NAME, OB_PUBLIC_SCHEMA_NAME);
      } else {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_DATABASES);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_DATABASES, OB_SYS_DATABASE_NAME, OB_ALL_DATABASE_TNAME,
            OB_RECYCLEBIN_SCHEMA_NAME, OB_PUBLIC_SCHEMA_NAME, OB_PUBLIC_SCHEMA_NAME);
      }
      break;
    }
    case peg::NodeKind::ShowVariables: {
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_VARIABLES;
      if (peg::show_global(statement)) {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_GLOBAL_VARIABLES);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_GLOBAL_VARIABLES, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_GLOBAL_VARIABLE_TNAME);
      } else {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_VARIABLES);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_VARIABLES, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SESSION_VARIABLE_TNAME);
      }
      break;
    }
    case peg::NodeKind::ShowStatus: {
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_STATUS;
      if (peg::show_global(statement)) {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_GLOBAL_STATUS);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_GLOBAL_STATUS, OB_INFORMATION_SCHEMA_NAME, OB_SESSION_STATUS_TNAME);
      } else {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_SESSION_STATUS);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_SESSION_STATUS, OB_INFORMATION_SCHEMA_NAME, OB_SESSION_STATUS_TNAME);
      }
      break;
    }
    case peg::NodeKind::ShowColumns:
    case peg::NodeKind::ShowIndexes:
    case peg::NodeKind::ShowCreateTable:
    case peg::NodeKind::ShowCreateView: {
      ObString database, table;
      uint64_t database_id = OB_INVALID_ID, table_id = OB_INVALID_ID;
      bool is_view = false;
      OZ(resolve_show_from_table(peg::show_object(statement),
          peg::show_has_database(statement) ? &peg::show_database(statement) : nullptr,
          statement.kind(), database, database_id, table, table_id, is_view));
      if (statement.kind() == peg::NodeKind::ShowColumns) {
        show_resv_ctx.stmt_type_ = stmt::T_SHOW_COLUMNS;
        if (peg::show_full(statement)) {
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_FULL_COLUMNS);
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_FULL_COLUMNS, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_TABLE_COLUMN_TNAME, table_id);
        } else {
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_COLUMNS);
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_COLUMNS, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_TABLE_COLUMN_TNAME, table_id);
        }
      } else if (statement.kind() == peg::NodeKind::ShowIndexes) {
        show_resv_ctx.stmt_type_ = stmt::T_SHOW_INDEXES;
        OZ(required.need_privs_.init(3));
        ObNeedPriv privilege;
        privilege.priv_level_ = OB_PRIV_USER_LEVEL;
        privilege.priv_set_ = OB_PRIV_SELECT;
        OZ(required.need_privs_.push_back(privilege));
        privilege.priv_level_ = OB_PRIV_DB_LEVEL;
        privilege.db_ = database;
        OZ(required.need_privs_.push_back(privilege));
        privilege.priv_level_ = OB_PRIV_TABLE_LEVEL;
        privilege.priv_set_ = OB_PRIV_TABLE_ACC;
        privilege.table_ = table;
        OZ(required.need_privs_.push_back(privilege));
        if (OB_SUCC(ret) && OB_FAIL(schema_checker_->check_priv_or(session_priv, roles, required))) {
          if (ret == OB_ERR_NO_TABLE_PRIVILEGE) {
            ret = OB_SUCCESS;
            bool allowed = false;
            OZ(schema_checker_->get_schema_guard()->check_priv_any_column_priv(session_priv, roles, database, table, allowed));
            if (OB_SUCC(ret) && !allowed) {
              ret = OB_ERR_NO_TABLE_PRIVILEGE;
              LOG_USER_ERROR(OB_ERR_NO_TABLE_PRIVILEGE, int(strlen("SELECT")), "SELECT", session_priv.user_name_.length(),
                  session_priv.user_name_.ptr(), session_priv.host_name_.length(), session_priv.host_name_.ptr(), table.length(), table.ptr());
            }
          }
        }
        if (peg::show_extended(statement)) {
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_EXTENDED_INDEXES);
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_EXTENDED_INDEXES, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_TABLE_INDEX_TNAME, table_id);
        } else {
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_INDEXES);
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_INDEXES, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_TABLE_INDEX_TNAME, table_id);
        }
      } else {
        show_resv_ctx.stmt_type_ = statement.kind() == peg::NodeKind::ShowCreateTable ? stmt::T_SHOW_CREATE_TABLE : stmt::T_SHOW_CREATE_VIEW;
        if (statement.kind() == peg::NodeKind::ShowCreateView || is_view) {
          ObNeedPriv privilege;
          privilege.db_ = database;
          privilege.table_ = table;
          privilege.priv_set_ = OB_PRIV_SHOW_VIEW | OB_PRIV_SELECT;
          privilege.priv_level_ = OB_PRIV_TABLE_LEVEL;
          OZ(required.need_privs_.init(1));
          OZ(required.need_privs_.push_back(privilege));
          OZ(schema_checker_->check_priv(session_priv, roles, required));
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_CREATE_VIEW);
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_CREATE_VIEW, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_CREATE_TABLE_TNAME, table_id);
        } else {
          bool allowed = false;
          OZ(schema_checker_->check_table_show(session_priv, roles, database, table, allowed));
          if (OB_SUCC(ret) && !allowed) {
            ret = OB_ERR_NO_TABLE_PRIVILEGE;
            LOG_USER_ERROR(OB_ERR_NO_TABLE_PRIVILEGE, int(strlen("SHOW")), "SHOW", session_priv.user_name_.length(),
                session_priv.user_name_.ptr(), session_priv.host_name_.length(), session_priv.host_name_.ptr(), table.length(), table.ptr());
          }
          GEN_SQL_STEP_1(ObShowSqlSet::SHOW_CREATE_TABLE);
          GEN_SQL_STEP_2(ObShowSqlSet::SHOW_CREATE_TABLE, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_CREATE_TABLE_TNAME, table_id);
        }
      }
      break;
    }
    case peg::NodeKind::ShowCreateDatabase: {
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_CREATE_DATABASE;
      uint64_t database_id = OB_INVALID_ID;
      OZ(resolve_show_from_database(peg::show_database(statement), database_id, show_resv_ctx.show_database_name_));
      OZ(check_db_access_for_show_sql(show_resv_ctx, session_priv, roles));
      if (peg::show_if_not_exists(statement)) {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_CREATE_DATABASE_EXISTS);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_CREATE_DATABASE_EXISTS, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_CREATE_DATABASE_TNAME, database_id);
      } else {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_CREATE_DATABASE);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_CREATE_DATABASE, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_CREATE_DATABASE_TNAME, database_id);
      }
      break;
    }
    case peg::NodeKind::ShowCreateProcedure:
    case peg::NodeKind::ShowCreateFunction: {
      const bool procedure = statement.kind() == peg::NodeKind::ShowCreateProcedure;
      show_resv_ctx.stmt_type_ = procedure ? stmt::T_SHOW_CREATE_PROCEDURE : stmt::T_SHOW_CREATE_FUNCTION;
      ObString database, routine;
      uint64_t database_id = OB_INVALID_ID, routine_id = OB_INVALID_ID;
      int64_t routine_type = -1;
      bool allowed = false;
      OZ(resolve_show_from_routine(peg::show_object(statement), procedure, database, database_id, routine, routine_id, routine_type));
      OZ(schema_checker_->check_routine_show(session_priv, database, routine, allowed));
      if (OB_SUCC(ret) && !allowed) {
        ret = OB_ERR_NO_TABLE_PRIVILEGE;
        LOG_USER_ERROR(OB_ERR_NO_TABLE_PRIVILEGE, int(strlen("SHOW")), "SHOW", session_priv.user_name_.length(),
            session_priv.user_name_.ptr(), session_priv.host_name_.length(), session_priv.host_name_.ptr(), routine.length(), routine.ptr());
      }
      if (procedure) {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_CREATE_PROCEDURE);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_CREATE_PROCEDURE, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_CREATE_PROCEDURE_TNAME, routine_id, routine_type);
      } else {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_CREATE_FUNCTION);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_CREATE_FUNCTION, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_CREATE_PROCEDURE_TNAME, routine_id, routine_type);
      }
      break;
    }
    case peg::NodeKind::ShowCreateTrigger: {
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_CREATE_TRIGGER;
      ObString database, trigger, table;
      uint64_t database_id = OB_INVALID_ID, trigger_id = OB_INVALID_ID;
      bool allowed = false;
      OZ(resolve_show_from_trigger(peg::show_object(statement), database, database_id, trigger, trigger_id, table));
      OZ(schema_checker_->check_trigger_show(session_priv, roles, database, trigger, allowed, table));
      if (OB_SUCC(ret) && !allowed) {
        ret = OB_ERR_NO_PRIVILEGE;
        LOG_USER_ERROR(OB_ERR_NO_PRIVILEGE, "TRIGGER");
      }
      GEN_SQL_STEP_1(ObShowSqlSet::SHOW_CREATE_TRIGGER);
      GEN_SQL_STEP_2(ObShowSqlSet::SHOW_CREATE_TRIGGER, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_SHOW_CREATE_TRIGGER_TNAME, trigger_id);
      break;
    }
    case peg::NodeKind::ShowCharset:
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_CHARSET;
      GEN_SQL_STEP_1(ObShowSqlSet::SHOW_CHARSET);
      GEN_SQL_STEP_2(ObShowSqlSet::SHOW_CHARSET, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_CHARSET_TNAME);
      break;
    case peg::NodeKind::ShowCollation:
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_COLLATION;
      GEN_SQL_STEP_1(ObShowSqlSet::SHOW_COLLATION);
      GEN_SQL_STEP_2(ObShowSqlSet::SHOW_COLLATION, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_COLLATION_TNAME);
      break;
    case peg::NodeKind::ShowGrants: {
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_GRANTS;
      uint64_t user_id = session_info_->get_user_id();
      ObString user = session_info_->get_user_name(), host = session_info_->get_host_name();
      if (peg::show_has_account(statement)) {
        const auto name = peg::show_account_name(statement);
        user.assign_ptr(name.data(), name.size());
        if (peg::show_account_has_host(statement)) {
          const auto hostname = peg::show_account_host(statement);
          host.assign_ptr(hostname.data(), hostname.size());
        } else host = ObString::make_string(OB_DEFAULT_HOST_NAME);
        OZ(schema_checker_->get_user_id(user, host, user_id));
      }
      GEN_SQL_STEP_1(ObShowSqlSet::SHOW_GRANTS, user.length(), user.ptr(), host.length(), host.ptr());
      GEN_SQL_STEP_2(ObShowSqlSet::SHOW_GRANTS, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_PRIVILEGE_GRANT_TNAME, user_id);
      break;
    }
    case peg::NodeKind::ShowProcesslist:
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_PROCESSLIST;
      if (peg::show_full(statement)) {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_SYS_FULL_PROCESSLIST);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_SYS_FULL_PROCESSLIST, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_PROCESSLIST_TNAME);
      } else {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_SYS_PROCESSLIST);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_SYS_PROCESSLIST, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_PROCESSLIST_TNAME);
      }
      break;
    case peg::NodeKind::ShowWarnings:
    case peg::NodeKind::ShowErrors:
      show_resv_ctx.stmt_type_ = statement.kind() == peg::NodeKind::ShowWarnings ? stmt::T_SHOW_WARNINGS : stmt::T_SHOW_ERRORS;
      if (statement.kind() == peg::NodeKind::ShowWarnings) {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_WARNINGS);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_WARNINGS, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_WARNING_TNAME);
      } else {
        GEN_SQL_STEP_1(ObShowSqlSet::SHOW_ERRORS);
        GEN_SQL_STEP_2(ObShowSqlSet::SHOW_ERRORS, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_WARNING_TNAME);
      }
      if (peg::show_has_limit(statement)) {
        OZ(sql_gen.gen_limit_str(static_cast<int64_t>(peg::show_limit_offset(statement)),
            static_cast<int64_t>(peg::show_limit_count(statement))));
        OX(sql_gen.assign_sql_str(select_sql));
      }
      break;
    case peg::NodeKind::ShowParameters:
      show_resv_ctx.stmt_type_ = stmt::T_SHOW_PARAMETERS;
      GEN_SQL_STEP_1(ObShowSqlSet::SHOW_PARAMETERS_WITH_DEFAULT_VALUE);
      GEN_SQL_STEP_2(ObShowSqlSet::SHOW_PARAMETERS_WITH_DEFAULT_VALUE, OB_SYS_DATABASE_NAME, OB_ALL_VIRTUAL_PARAMETER_STAT_TNAME);
      break;
    default: ret = OB_NOT_IMPLEMENT; break;
  }
  OZ(parse_and_resolve_select_sql(select_sql));
  OZ(resolve_like_or_where_clause(show_resv_ctx));
  if (OB_SUCC(ret)) {
    ObSelectStmt *select = get_select_stmt();
    CK(OB_NOT_NULL(select));
    OX(select->set_is_from_show_stmt(true));
    if (OB_SUCC(ret) && select->get_query_ctx() != nullptr) select->get_query_ctx()->set_literal_stmt_type(show_resv_ctx.stmt_type_);
    OZ(process_select_type(select, show_resv_ctx.stmt_type_));
    OZ(select->formalize_stmt(session_info_));
  }
  return ret;
}

int ObShowResolver::get_database_info(const peg::Node *database, const ObString &session_database,
                                     ObShowResolverContext &context, uint64_t &id)
{
  int ret = OB_SUCCESS;
  if (database != nullptr) OZ(resolve_show_from_database(*database, id, context.show_database_name_));
  else if (session_database.empty()) ret = OB_ERR_NO_DB_SELECTED;
  else {
    context.show_database_name_ = session_database;
    OZ(schema_checker_->get_database_id(session_database, id));
  }
  if (OB_SUCC(ret) && id == OB_INVALID_ID) ret = OB_ERR_UNEXPECTED;
  return ret;
}

int ObShowResolver::process_select_type(ObSelectStmt *select, stmt::StmtType type)
{
  switch (type) {
    case stmt::T_SHOW_CREATE_TABLE:
    case stmt::T_SHOW_CREATE_DATABASE:
    case stmt::T_SHOW_CREATE_VIEW:
    case stmt::T_SHOW_ERRORS:
    case stmt::T_SHOW_GRANTS:
    case stmt::T_SHOW_PROCESSLIST:
    case stmt::T_SHOW_WARNINGS: select->set_select_type(NOT_AFFECT_FOUND_ROWS); break;
    default: select->set_select_type(AFFECT_FOUND_ROWS); break;
  }
  return OB_SUCCESS;
}

int ObShowResolver::resolve_show_from_table(const peg::Node &object, const peg::Node *database,
    peg::NodeKind kind, ObString &database_name, uint64_t &database_id,
    ObString &table_name, uint64_t &table_id, bool &is_view)
{
  int ret = OB_SUCCESS;
  const ObTableSchema *table = nullptr;
  if (database == nullptr) {
    if (session_info_->get_database_name().empty() && !peg::relation_has_database(object)) ret = OB_ERR_NO_DB_SELECTED;
    else if (OB_FAIL(resolve_table_relation_factor_normal(&object, database_id, table_name, database_name))) {
      if (ret == OB_TABLE_NOT_EXIST) {
        if (is_information_schema_database_id(database_id)) {
          ret = OB_ERR_UNKNOWN_TABLE;
          LOG_USER_ERROR(OB_ERR_UNKNOWN_TABLE, table_name.length(), table_name.ptr(), database_name.length(), database_name.ptr());
        } else {
          ObCStringHelper helper;
          LOG_USER_ERROR(OB_TABLE_NOT_EXIST, helper.convert(database_name), helper.convert(table_name));
        }
      }
    }
  } else {
    const auto &name = peg::relation_name(object);
    const auto name_text = name.text();
    OZ(ob_write_string(*allocator_, ObString(name_text.size(), name_text.data()), table_name));
    if (OB_SUCC(ret) && table_name.empty()) ret = OB_WRONG_TABLE_NAME;
    OZ(resolve_show_from_database(*database, database_id, database_name));
  }
  uint64_t previous_session = OB_INVALID_ID;
  if (kind == peg::NodeKind::ShowCreateView && schema_checker_->get_schema_mgr() != nullptr) {
    previous_session = schema_checker_->get_schema_mgr()->get_session_id();
    schema_checker_->get_schema_mgr()->set_session_id(0);
  }
  OZ(schema_checker_->get_table_schema(database_id, table_name, false, false, false, table));
  CK(OB_NOT_NULL(table));
  if (OB_SUCC(ret) && kind == peg::NodeKind::ShowCreateView && !table->is_view_table()) {
    ret = OB_ERR_WRONG_OBJECT;
    ObCStringHelper helper;
    LOG_USER_ERROR(OB_ERR_WRONG_OBJECT, helper.convert(database_name), helper.convert(table_name), "VIEW");
  }
  if (OB_SUCC(ret)) {
    table_id = table->get_table_id();
    is_view = table->is_view_table();
  }
  if (previous_session != OB_INVALID_ID) schema_checker_->get_schema_mgr()->set_session_id(previous_session);
  return ret;
}

int ObShowResolver::resolve_show_from_database(const peg::Node &database, uint64_t &id, ObString &name)
{
  int ret = resolve_database_factor(&database, id, name);
  if (ret == OB_ERR_BAD_DATABASE) LOG_USER_ERROR(OB_ERR_BAD_DATABASE, name.length(), name.ptr());
  return ret;
}

int ObShowResolver::resolve_schema_object_name(const peg::Node &object, ObString &database,
                                              uint64_t &database_id, ObString &name)
{
  int ret = OB_SUCCESS;
  const auto &identifier = peg::relation_name(object);
  const auto identifier_text = identifier.text();
  OZ(ob_write_string(*allocator_, ObString(identifier_text.size(), identifier_text.data()), name));
  if (peg::relation_has_database(object)) {
    const auto &qualifier = peg::relation_database(object);
    const auto qualifier_text = qualifier.text();
    OZ(ob_write_string(*allocator_, ObString(qualifier_text.size(), qualifier_text.data()), database));
  } else database = session_info_->get_database_name();
  if (OB_SUCC(ret) && database.empty()) ret = OB_ERR_NO_DB_SELECTED;
  OZ(schema_checker_->get_database_id(database, database_id));
  return ret;
}

int ObShowResolver::resolve_show_from_routine(const peg::Node &object, bool procedure,
    ObString &database, uint64_t &database_id, ObString &name, uint64_t &id, int64_t &type)
{
  int ret = OB_SUCCESS;
  const ObRoutineInfo *routine = nullptr;
  OZ(resolve_schema_object_name(object, database, database_id, name));
  if (procedure) OZ(schema_checker_->get_standalone_procedure_info(database, name, routine));
  else OZ(schema_checker_->get_standalone_function_info(database, name, routine));
  if (OB_SUCC(ret) && routine == nullptr) ret = OB_ERR_SP_DOES_NOT_EXIST;
  if (OB_SUCC(ret)) {
    id = routine->get_routine_id();
    type = routine->get_routine_type();
  }
  return ret;
}

int ObShowResolver::resolve_show_from_trigger(const peg::Node &object, ObString &database,
    uint64_t &database_id, ObString &name, uint64_t &id, ObString &table_name)
{
  int ret = OB_SUCCESS;
  const ObTriggerInfo *trigger = nullptr;
  const ObTableSchema *table = nullptr;
  OZ(resolve_schema_object_name(object, database, database_id, name));
  OZ(schema_checker_->get_trigger_info(database, name, trigger));
  if (OB_SUCC(ret) && trigger == nullptr) ret = OB_ERR_TRIGGER_NOT_EXIST;
  OX(id = trigger->get_trigger_id());
  OZ(schema_checker_->get_table_schema(trigger->get_base_object_id(), table));
  CK(OB_NOT_NULL(table));
  OX(table_name = table->get_table_name());
  return ret;
}

int ObShowResolver::parse_and_resolve_select_sql(const ObString &sql)
{
  int ret = OB_SUCCESS;
  ParsedSql query;
  ObParser parser(*allocator_, session_info_->get_sql_mode());
  OZ(parser.parse(sql, query));
  if (OB_SUCC(ret)) {
    const ParsedSql *previous_syntax = params_.syntax_;
    params_.syntax_ = &query;
    ret = ObSelectResolver::resolve(*query.statement());
    params_.syntax_ = previous_syntax;
  }
  return ret;
}

int ObShowResolver::resolve_like_or_where_clause(ObShowResolverContext &context)
{
  int ret = OB_SUCCESS;
  const peg::Node &statement = *context.parse_tree_;
  const auto filter = peg::show_filter(statement);
  if (filter == peg::ShowFilter::Where) {
    const ObString previous_alias = show_table_alias_;
    show_table_alias_ = context.column_name_;
    ret = ObDMLResolver::resolve_where_clause(&peg::show_filter_expression(statement));
    show_table_alias_ = previous_alias;
  } else if (filter == peg::ShowFilter::Like) {
    const auto pattern = peg::expression_literal(peg::show_like_pattern(statement)).text;
    const auto escape = peg::expression_literal(peg::show_like_escape(statement)).text;
    if (!pattern.empty()) {
      ObString owned_pattern, owned_escape;
      OZ(ob_write_string(*allocator_, ObString(pattern.size(), pattern.data()), owned_pattern));
      OZ(ob_write_string(*allocator_, ObString(escape.size(), escape.data()), owned_escape));
      ObQualifiedName name;
      name.tbl_name_ = ObString::make_string(ObShowSqlSet::SUBQERY_ALIAS);
      name.col_name_ = context.like_column_;
      ObRawExpr *column = nullptr;
      ObConstRawExpr *pattern_expr = nullptr, *escape_expr = nullptr;
      ObOpRawExpr *like = nullptr;
      OZ(resolve_column_ref_expr(name, column));
      CK(OB_NOT_NULL(column));
      const auto collation = ObCharset::get_default_collation(ObCharset::get_default_charset());
      OZ(ObRawExprUtils::build_const_string_expr(*params_.expr_factory_, ObVarcharType, owned_pattern, collation, pattern_expr));
      OZ(ObRawExprUtils::build_const_string_expr(*params_.expr_factory_, ObVarcharType, owned_escape, collation, escape_expr));
      OZ(params_.expr_factory_->create_raw_expr(T_OP_LIKE, like));
      OZ(like->set_param_exprs(column, pattern_expr, escape_expr));
      OZ(like->formalize(session_info_));
      OZ(get_stmt()->add_condition_expr(like));
    }
  }
  return ret;
}

int ObShowResolver::resolve_column_ref_expr(const ObQualifiedName &name, ObRawExpr *&expression)
{
  int ret = OB_SUCCESS;
  if (!show_table_alias_.empty() && ObCharset::case_insensitive_equal(name.col_name_, show_table_alias_)) {
    ObQualifiedName resolved;
    OZ(resolved.assign(name));
    OX(resolved.col_name_ = ObString::make_string("table_name"));
    OZ(resolve_table_column_ref(resolved, expression));
  } else OZ(resolve_table_column_ref(name, expression));
  return ret;
}

int ObShowResolver::ObSqlStrGenerator::init(common::ObIAllocator *allocator)
{
  int ret = OB_SUCCESS;
  if (OB_UNLIKELY(NULL == (sql_buf_ = static_cast<char *>(allocator->alloc(OB_MAX_SQL_LENGTH))))) {
    ret = OB_ALLOCATE_MEMORY_FAILED;
    LOG_ERROR("failed to alloc sql buf", K(ret), K(OB_MAX_SQL_LENGTH));
  }
  return ret;
}

int ObShowResolver::ObSqlStrGenerator::gen_select_str(const char *select_str, ...)
{
  int ret = OB_SUCCESS;
  if (OB_ISNULL(sql_buf_)) {
    ret = OB_NOT_INIT;
  } else {
    if (NULL == select_str) {
      if (OB_FAIL(databuff_printf(sql_buf_, OB_MAX_SQL_LENGTH, sql_buf_pos_, "SELECT * "))) {
      }
    } else {
      va_list select_args;
      va_start(select_args, select_str);
      if (OB_FAIL(databuff_vprintf(sql_buf_, OB_MAX_SQL_LENGTH, sql_buf_pos_,
                                    select_str, select_args))) {
      }
      va_end(select_args);
    }
  }
  return ret;
}

int ObShowResolver::ObSqlStrGenerator::gen_from_str(const char *subquery_str, ...)
{
  int ret = OB_SUCCESS;
  HEAP_VAR(char[OB_MAX_SQL_LENGTH], tmp_buf) {
    int64_t pos = 0;
    if (OB_ISNULL(sql_buf_) || OB_ISNULL(subquery_str)) {
      ret = OB_NOT_INIT;
    } else if (OB_FAIL(databuff_printf(tmp_buf,
                                OB_MAX_SQL_LENGTH,
                                pos,
                                " FROM (%s) %s",
                                subquery_str,
                                ObShowSqlSet::SUBQERY_ALIAS))) {
    } else {
      va_list subquery_args;
      va_start(subquery_args, subquery_str);
      if (OB_FAIL(databuff_vprintf(sql_buf_,
                                   OB_MAX_SQL_LENGTH,
                                   sql_buf_pos_,
                                   tmp_buf,
                                   subquery_args))) {
      }
      va_end(subquery_args);
    }
  }
  return ret;
}

int ObShowResolver::ObSqlStrGenerator::gen_limit_str(int64_t offset, int64_t row_cnt)
{
  int ret = OB_SUCCESS;
  if (OB_FAIL(databuff_printf(sql_buf_,
                              OB_MAX_SQL_LENGTH,
                              sql_buf_pos_,
                              " LIMIT %ld, %ld ",
                              offset,
                              row_cnt))) {
  }
  return ret;
}
void ObShowResolver::ObSqlStrGenerator::assign_sql_str(ObString &sql_str)
{
  sql_str.assign_ptr(sql_buf_, static_cast<uint32_t>(sql_buf_pos_));
}

}/* ns sql*/
}/* ns oceanbase */


namespace oceanbase
{
namespace sql
{
#define DEFINE_SHOW_CLAUSE(clause_type, string)                         \
  const char *ObShowResolver::ObShowSqlSet::clause_type = string
// @SHOW_STMT_TYPE : Identify the type of show statement, if the same show statement does not use subquery or select, then a different SHOW_STMT_TYPE needs to be defined
// @select_str : NULL to use default value (default is "select *"), other cases require writing your own select statement
// @subquery : This item cannot be NULL
// @like_str : like statement used column, if this show statement does not support like syntax, set to NULL
#define DEFINE_SHOW_CLAUSE_SET(SHOW_STMT_TYPE, select_str, subquery_str, ora_subquery_str, like_str) \
  DEFINE_SHOW_CLAUSE(SHOW_STMT_TYPE##_SELECT, select_str);              \
  DEFINE_SHOW_CLAUSE(SHOW_STMT_TYPE##_SUBQUERY, subquery_str);          \
  DEFINE_SHOW_CLAUSE(SHOW_STMT_TYPE##_ORA_SUBQUERY, ora_subquery_str);  \
  DEFINE_SHOW_CLAUSE(SHOW_STMT_TYPE##_LIKE, like_str)

DEFINE_SHOW_CLAUSE(SUBQERY_ALIAS, "subquery_alias");
DEFINE_SHOW_CLAUSE_SET(SHOW_TABLES,
                       "SELECT table_name AS `Tables_in_%.*s` ",
                       "SELECT table_name FROM %s.%s WHERE database_id = %ld ORDER BY table_name COLLATE utf8mb4_bin ASC",
                       NULL,
                       "table_name");
DEFINE_SHOW_CLAUSE_SET(SHOW_TABLES_LIKE,
                       "SELECT table_name AS `Tables_in_%.*s (%.*s)` ",
                       "SELECT table_name FROM %s.%s WHERE database_id = %ld ORDER BY table_name COLLATE utf8mb4_bin ASC",
                       NULL,
                       "table_name");
DEFINE_SHOW_CLAUSE_SET(SHOW_FULL_TABLES,
                       "SELECT table_name AS `Tables_in_%.*s`, table_type AS `Table_type` ",
                       "SELECT table_name, table_type FROM %s.%s WHERE database_id = %ld ORDER BY table_name COLLATE utf8mb4_bin ASC",
                       NULL,
                       "table_name");
DEFINE_SHOW_CLAUSE_SET(SHOW_FULL_TABLES_LIKE,
                       "SELECT table_name AS `Tables_in_%.*s (%.*s)`, table_type AS `Table_type` ",
                       "SELECT table_name, table_type FROM %s.%s WHERE database_id = %ld ORDER BY table_name COLLATE utf8mb4_bin ASC",
                       NULL,
                       "table_name");
DEFINE_SHOW_CLAUSE_SET(SHOW_CHARSET,
                       NULL,
                       "SELECT charset AS Charset, description AS `Description`, default_collation AS `Default collation`, max_length AS `Maxlen` FROM %s.%s",
                       R"(SELECT "CHARSET" AS "CHARSET", "DESCRIPTION" AS "DESCRIPTION", "DEFAULT_COLLATION" AS "DEFAULT COLLATION", "MAX_LENGTH" AS "MAXLEN" FROM %s.%s)",
                       "Charset");
DEFINE_SHOW_CLAUSE_SET(SHOW_VARIABLES,
                       NULL,
                       "SELECT /*+parallel(1)*/ variable_name AS `Variable_name`, value AS `Value` FROM %s.%s ORDER BY variable_name ASC",
                       R"(SELECT /*+parallel(1)*/ "VARIABLE_NAME" AS "VARIABLE_NAME", "VALUE" AS "VALUE" FROM %s.%s ORDER BY VARIABLE_NAME ASC)",
                       "Variable_name");
DEFINE_SHOW_CLAUSE_SET(SHOW_GLOBAL_VARIABLES,
                       NULL,
                       "SELECT /*+parallel(1)*/ variable_name AS `Variable_name`, value AS `Value` FROM %s.%s ORDER BY variable_name ASC",
                       R"(SELECT /*+parallel(1)*/ "VARIABLE_NAME" AS "VARIABLE_NAME", "VALUE" AS "VALUE" FROM %s.%s ORDER BY VARIABLE_NAME ASC)",
                       "Variable_name");
DEFINE_SHOW_CLAUSE_SET(SHOW_COLUMNS,
                       NULL,
                       "SELECT field AS `Field`, type AS `Type`, `NULL` AS `Null`, `KEY` AS `Key`, `DEFAULT` AS `Default`, extra AS `Extra` \
                        FROM %s.%s  where table_id = %ld AND is_hidden = False",
                        R"(SELECT "FIELD" AS "FIELD", "TYPE" AS "TYPE", "NULL" AS "NULL", "KEY" AS "KEY", "DEFAULT" AS "DEFAULT", "EXTRA" AS "EXTRA" )"
                        R"(FROM %s.%s  WHERE TABLE_ID = %ld AND IS_HIDDEN = 0)",
                       "Field");
DEFINE_SHOW_CLAUSE_SET(SHOW_FULL_COLUMNS,
                       NULL,
                       "SELECT field AS `Field`, type AS `Type`, collation AS `Collation`, `NULL` AS `Null`, `KEY` AS `Key`, `DEFAULT` AS `Default`, extra AS `Extra`, `PRIVILEGES` AS `Privileges`, `COMMENT` AS `Comment` \
                       FROM %s.%s  where table_id = %ld AND is_hidden = False",
                       R"(SELECT "FIELD" AS "FIELD", "TYPE" AS "TYPE", "COLLATION" AS "COLLATION", "NULL" AS "NULL", KEY AS "KEY", "DEFAULT" AS "DEFAULT", EXTRA AS "EXTRA", "PRIVILEGES" AS "PRIVILEGES", "COMMENT" AS "COMMENT" )"
                       R"(FROM %s.%s  WHERE TABLE_ID = %ld AND IS_HIDDEN = 0)",
                       "Field");
DEFINE_SHOW_CLAUSE_SET(SHOW_EXTENDED_COLUMNS,
                       NULL,
                       "SELECT field AS `Field`, type AS `Type`, `NULL` AS `Null`, `KEY` AS `Key`, `DEFAULT` AS `Default`, extra AS `Extra` \
                        FROM %s.%s  where table_id = %ld",
                        R"(SELECT "FIELD" AS "FIELD", "TYPE" AS "TYPE", "NULL" AS "NULL", "KEY" AS "KEY", "DEFAULT" AS "DEFAULT", "EXTRA" AS "EXTRA" )"
                        R"(FROM %s.%s  WHERE TABLE_ID = %ld)",
                       "Field");
DEFINE_SHOW_CLAUSE_SET(SHOW_EXTENDED_FULL_COLUMNS,
                       NULL,
                       "SELECT field AS `Field`, type AS `Type`, collation AS `Collation`, `NULL` AS `Null`, `KEY` AS `Key`, `DEFAULT` AS `Default`, extra AS `Extra`, `PRIVILEGES` AS `Privileges`, `COMMENT` AS `Comment` \
                       FROM %s.%s  where table_id = %ld",
                       R"(SELECT "FIELD" AS "FIELD", "TYPE" AS "TYPE", "COLLATION" AS "COLLATION", "NULL" AS "NULL", KEY AS "KEY", "DEFAULT" AS "DEFAULT", EXTRA AS "EXTRA", "PRIVILEGES" AS "PRIVILEGES", "COMMENT" AS "COMMENT" )"
                       R"(FROM %s.%s  WHERE TABLE_ID = %ld)",
                       "Field");
DEFINE_SHOW_CLAUSE_SET(SHOW_CREATE_DATABASE,
                       NULL,
                       "SELECT `database_name` AS `Database`, create_database AS `Create Database` FROM %s.%s  WHERE database_id = %ld",
                       NULL,
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_CREATE_DATABASE_EXISTS,
                       NULL,
                       "SELECT `database_name` AS `Database`, create_database_with_if_not_exists AS `Create Database` FROM %s.%s WHERE database_id = %ld",
                       NULL,
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_INDEXES,
                       NULL,
                       "SELECT `TABLE` AS `Table`, NON_UNIQUE AS Non_unique, KEY_NAME AS Key_name, SEQ_IN_INDEX AS Seq_in_index, COLUMN_NAME AS Column_name, COLLATION AS Collation, CARDINALITY AS Cardinality, SUB_PART AS Sub_part, PACKED AS Packed, `NULL` AS `Null`, INDEX_TYPE AS Index_type, `COMMENT` AS `Comment`, INDEX_COMMENT AS Index_comment, IS_VISIBLE AS Visible, EXPRESSION AS Expression FROM %s.%s  where table_id = %ld AND is_column_visible = true",
                       R"(SELECT "TABLE" AS "TABLE", "NON_UNIQUE" AS "NON_UNIQUE", "KEY_NAME" AS "KEY_NAME", "SEQ_IN_INDEX" AS "SEQ_IN_INDEX", "COLUMN_NAME" AS "COLUMN_NAME", "COLLATION" AS "COLLATION", "CARDINALITY" AS "CARDINALITY", "SUB_PART" AS "SUB_PART", "PACKED" AS "PACKED", "NULL" AS "NULL", "INDEX_TYPE" AS "INDEX_TYPE", "COMMENT" AS "COMMENT", )"
                       R"(INDEX_COMMENT" AS "INDEX_COMMENT", "IS_VISIBLE" AS "VISIBLE", "EXPRESSION" AS "EXPRESSION" FROM %s.%s  WHERE TABLE_ID = %ld AND IS_COLUMN_VISIBLE = 1")",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_EXTENDED_INDEXES,
                       NULL,
                       "SELECT `TABLE` AS `Table`, NON_UNIQUE AS Non_unique, KEY_NAME AS Key_name, SEQ_IN_INDEX AS Seq_in_index, COLUMN_NAME AS Column_name, COLLATION AS Collation, CARDINALITY AS Cardinality, SUB_PART AS Sub_part, PACKED AS Packed, `NULL` AS `Null`, INDEX_TYPE AS Index_type, `COMMENT` AS `Comment`, INDEX_COMMENT AS Index_comment, IS_VISIBLE AS Visible, EXPRESSION AS Expression FROM %s.%s  where table_id = %ld",
                       R"(SELECT "TABLE" AS "TABLE", "NON_UNIQUE" AS "NON_UNIQUE", "KEY_NAME" AS "KEY_NAME", "SEQ_IN_INDEX" AS "SEQ_IN_INDEX", "COLUMN_NAME" AS "COLUMN_NAME", "COLLATION" AS "COLLATION", "CARDINALITY" AS "CARDINALITY", "SUB_PART" AS "SUB_PART", "PACKED" AS "PACKED", "NULL" AS "NULL", "INDEX_TYPE" AS "INDEX_TYPE", "COMMENT" AS "COMMENT", )"
                       R"(INDEX_COMMENT" AS "INDEX_COMMENT", "IS_VISIBLE" AS "VISIBLE", "EXPRESSION" AS "EXPRESSION" FROM %s.%s  WHERE TABLE_ID = %ld")",
                       NULL);

DEFINE_SHOW_CLAUSE_SET(SHOW_TRACE,
                       NULL,
                       "SELECT span_name as `Operation`, start_ts as `StartTime`, concat(cast(elapse/1000 as number(20, 3)), ' ms')  as `ElapseTime` from %s.%s",
                       R"(SELECT span_name as "OPERATION", to_char(start_ts,'yyyy-mm-dd hh24:mi:ss') as "START_TIME", concat(to_char(elapse/1000, 'FM99999999999999990.000'), ' ms') as "ELAPSE_TIME" FROM %s.%s)",
                       NULL);

DEFINE_SHOW_CLAUSE_SET(SHOW_TRACE_JSON,
                       NULL,
                       "select json_arrayagg(json_object('trace_id', trace_id, 'parent', parent_span_id, 'span_id', span_id, 'span_name', span_name, 'start_ts', start_ts, 'end_ts', end_ts, 'elapse', elapse, 'tags', cast(case when tags='' then NULL else tags end as json), 'logs', cast(case when logs='' then NULL else logs end as json))) as ShowTraceJSON from %s.%s",
                       R"(select json_arrayagg(json_object('trace_id' : trace_id, 'parent' : parent_span_id, 'span_id' : span_id, 'span_name' : span_name, 'start_ts' : cast(start_ts as varchar(100)), 'end_ts' : cast(end_ts as varchar(100)), 'elapse' : elapse, 'tags' : cast(tags as json), 'logs' : cast(logs as json) returning json) returning json)  as SHOW_TRACE_JSON from %s.%s)",
                       NULL);

DEFINE_SHOW_CLAUSE_SET(SHOW_ENGINES,
                       NULL,
                       "SELECT * FROM %s.%s ",
                       NULL,
                       NULL);

DEFINE_SHOW_CLAUSE_SET(SHOW_PROFILE,
                       NULL,
                       "SELECT * FROM %s.%s ",
                       NULL,
                       NULL);

DEFINE_SHOW_CLAUSE_SET(SHOW_ENGINE,
                       NULL,
                       "SELECT 1 as `Type`, 1 as `Name`, 1 as `Status` FROM dual where 0 = 1 ",
                       NULL,
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_OPEN_TABLES,
                       NULL,
                       "SELECT 1 as `Database`, 1 as `Table`, 1 as In_use, 1 as Name_locked FROM dual where 0 = 1 ",
                       NULL,
                       "Table");

DEFINE_SHOW_CLAUSE_SET(SHOW_PRIVILEGES,
                       NULL,
                       "SELECT * FROM %s.%s ",
                       "SELECT * FROM %s.%s ",
                       NULL);
        
DEFINE_SHOW_CLAUSE_SET(SHOW_COLLATION,
                       NULL,
                       "SELECT collation AS `Collation`, charset AS `Charset`, id AS `Id`, is_default AS `Default`, is_compiled AS `Compiled`, sortlen AS `Sortlen` FROM %s.%s ",
                       R"(SELECT "COLLATION" AS "COLLATION", "CHARSET" AS "CHARSET", "ID" AS "ID", "IS_DEFAULT" AS "DEFAULT", "IS_COMPILED" AS "COMPILED", "SORTLEN" AS "SORTLEN" FROM %s.%s )",
                       "Collation");
DEFINE_SHOW_CLAUSE_SET(SHOW_GRANTS,
                       "SELECT grants AS `Grants for %.*s@%.*s` ",
                       "SELECT grants FROM %s.%s WHERE user_id = %ld",
                       R"(SELECT "GRANTS" AS "GRANTS" FROM %s.%s WHERE USER_ID = %ld)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_PROCESSLIST,
                       NULL,
                       "SELECT id AS `Id`, user AS `User`, host AS `Host`, db AS `db`, command AS `Command`, cast(time as SIGNED) AS `Time`, state AS `State`, info AS `Info` FROM %s.%s WHERE %ld>=0",
                       R"(SELECT "ID" AS "ID", "USER" AS "USER", "HOST" AS "HOST", "DB" AS "DB", "COMMAND" AS "COMMAND", CAST("TIME" AS INT) AS "TIME", "STATE" AS "STATE", "INFO" AS "INFO" FROM %s.%s WHERE %ld>=0)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_FULL_PROCESSLIST,
                       NULL,
                       "SELECT id AS `Id`, user AS `User`, host AS `Host`, db AS `db`, command AS `Command`, cast(time as SIGNED) AS `Time`, state AS `State`, info AS `Info` FROM %s.%s WHERE %ld>=0",
                       R"(SELECT "ID" AS "ID", "USER" AS "USER", "HOST" AS "HOST", "DB" AS "DB", "COMMAND" AS "COMMAND", CAST("TIME" AS INT) AS "TIME", "STATE" AS "STATE", "INFO" AS "INFO" FROM %s.%s WHERE %ld>=0)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_SYS_PROCESSLIST,
                       NULL,
                       "SELECT id AS `Id`, user AS `User`, host AS `Host`, db AS `db`, command AS `Command`, cast(time as SIGNED) AS `Time`, state AS `State`, info AS `Info` FROM %s.%s",
                       R"(SELECT "ID" AS "ID", "USER" AS "USER", "HOST" AS "HOST", "DB" AS "DB", "COMMAND" AS "COMMAND", CAST("TIME" AS INT) AS "TIME", "STATE" AS "STATE", "INFO" AS "INFO" FROM %s.%s)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_SYS_FULL_PROCESSLIST,
                       NULL,
                       "SELECT id AS `Id`, user AS `User`, host AS `Host`, db AS `db`, command AS `Command`, cast(time as SIGNED) AS `Time`, state AS `State`, info AS `Info`, sql_port AS `Port` FROM oceanbase.__all_virtual_server_stat, %s.%s",
                       R"(SELECT "ID" AS "ID", "USER" AS "USER", "HOST" AS "HOST", "DB" AS "DB", "COMMAND" AS "COMMAND", CAST("TIME" AS INT) AS "TIME", "STATE" AS "STATE", "INFO" AS "INFO", "SQL_PORT" AS "PORT" FROM %s.%s)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_TABLE_STATUS,
                       NULL,
                        "SELECT table_name AS `Name`, engine as `Engine`, version as `Version`, row_format as `Row_format`, `ROWS` as `Rows`, avg_row_length as `Avg_row_length`, data_length as `Data_length`, max_data_length as `Max_data_length`, index_length as `Index_length`, data_free as `Data_free`, auto_increment as `Auto_increment`, create_time as `Create_time`, update_time as `Update_time`, check_time as `Check_time`, collation as `Collation`, checksum as `Checksum`, create_options as `Create_options`, `COMMENT` as `Comment` FROM (%s) WHERE database_id = %ld ORDER BY name COLLATE utf8mb4_bin ASC",
                       R"(SELECT "TABLE_NAME" AS "NAME", "ENGINE", "VERSION", "ROW_FORMAT", "ROWS" AS "ROWS", "AVG_ROW_LENGTH", "DATA_LENGTH", "MAX_DATA_LENGTH", "INDEX_LENGTH", "DATA_FREE", "AUTO_INCREMENT", "CREATE_TIME", "UPDATE_TIME", "CHECK_TIME", "COLLATION", "CHECKSUM", "CREATE_OPTIONS", "COMMENT" AS "COMMENT" FROM (%s) WHERE DATABASE_ID = %ld ORDER BY NAME COLLATE UTF8MB4_BIN ASC)",
                       "name");
DEFINE_SHOW_CLAUSE_SET(SHOW_PROCEDURE_STATUS,
                       NULL,
                       "select database_name AS `Db`, routine_name AS `Name`, c.type AS `Type`, c.definer AS `Definer`, p.gmt_modified AS `Modified`, p.gmt_create AS `Created`, c.security_type AS `Security_type`, p.comment AS `Comment`, character_set_client, collation_connection, db_collation AS `Database Collation`from %s.%s p, %s.%s d, %s.%s c where p.database_id = d.database_id and d.database_name = c.db and p.routine_name = c.name and (case c.type when 'PROCEDURE' then 1 when 'FUNCTION' then 2 else 0 end) = p.routine_type and d.database_id = %ld and p.routine_type = %ld and (0 = sys_privilege_check('routine_acc') or 0 = sys_privilege_check('routine_acc', d.database_name, p.routine_name, p.routine_type)) ORDER BY name COLLATE utf8mb4_bin ASC",
                       NULL,
                       "name");
DEFINE_SHOW_CLAUSE_SET(SHOW_TRIGGERS,
                       NULL,
                       "select t.trigger_name as `Trigger`, t.event_manipulation as `Event`, t.event_object_table as `Table`, t.action_statement as `Statement`, t.action_timing as `Timing`, t.created as `Created`, t.sql_mode as `sql_mode`, t.definer as `Definer`, t.character_set_client as `character_set_client`, t.collation_connection as `collation_connection`, t.database_collation as `Database Collation` from %s.%s t, %s.%s d where t.event_object_schema = d.database_name and d.database_id = %ld ",
                       NULL,
                       "Trigger");
DEFINE_SHOW_CLAUSE_SET(SHOW_TRIGGERS_LIKE,
                       NULL,
                       "select t.trigger_name as `Trigger`, t.event_manipulation as `Event`, t.event_object_table as `Table`, t.action_statement as `Statement`, t.action_timing as `Timing`, t.created as `Created`, t.sql_mode as `sql_mode`, t.definer as `Definer`, t.character_set_client as `character_set_client`, t.collation_connection as `collation_connection`, t.database_collation as `Database Collation` from %s.%s t, %s.%s d where t.event_object_schema = d.database_name and d.database_id = %ld ",
                       NULL,
                       "Table");
DEFINE_SHOW_CLAUSE_SET(SHOW_WARNINGS,
                       NULL,
                       "SELECT `level` AS `Level`, `code` AS `Code`, `message` AS `Message` FROM %s.%s ",
                       NULL,
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_COUNT_WARNINGS,
                       NULL,
                       "SELECT count(*) AS `@@session.warning_count` FROM %s.%s ",
                       NULL,
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_ERRORS,
                       NULL,
                       "SELECT `level` AS `Level`, `code` AS `Code`, `message` AS `Message` FROM %s.%s  WHERE `level` = \'Error\'",
                       NULL,
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_COUNT_ERRORS,
                       NULL,
                       "SELECT count(*) AS `@@session.error_count` FROM %s.%s  WHERE `level` = \'Error\'",
                       NULL,
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_PARAMETERS,
                       NULL,
                       "SELECT svr_type, name, data_type, value, info, section, scope, source, edit_level from %s.%s where name not like '\\_%%'",
                       R"(SELECT "SVR_TYPE", "NAME", "DATA_TYPE", "VALUE", "INFO", "SECTION", "SCOPE", "SOURCE", "EDIT_LEVEL" FROM %s.%s WHERE NAME NOT LIKE '\_%%' ESCAPE '\')",
                       "name");
DEFINE_SHOW_CLAUSE_SET(SHOW_PARAMETERS_WITH_DEFAULT_VALUE,
                       NULL,
                      "SELECT svr_type, name, data_type, value, info, section, scope, source, edit_level, default_value, isdefault from %s.%s where (name not like '\\_%%' or isdefault=0)",
                      R"(SELECT "SVR_TYPE", "NAME", "DATA_TYPE", "VALUE", "INFO", "SECTION", "SCOPE", "SOURCE", "EDIT_LEVEL", "DEFAULT_VALUE", "ISDEFAULT" FROM %s.%s WHERE (NAME NOT LIKE '\_%%' ESCAPE '\' or ISDEFAULT=0))",
                      "name");
DEFINE_SHOW_CLAUSE_SET(SHOW_PARAMETERS_UNSYS,
                       NULL,
                       "SELECT 1 `gmt_create`, 1 `gmt_modified`, 1 `svr_type`, 1 `name`, 1 `data_type`, 1 `value`, 1 `info`, 1 `section`, 1 `scope`, 1 `source`, 1 `edit_level` FROM (SELECT 1 FROM DUAL) tmp_table WHERE 1 != 1",
                       R"(SELECT 1 "GMT_CREATE", 1 "GMT_MODIFIED", 1 "SVR_TYPE", 1 "NAME", 1 "DATA_TYPE", 1 "VALUE", 1 "INFO", 1 "SECTION", 1 "SCOPE" , 1 "SOURCE", 1 "EDIT_LEVEL" FROM (SELECT 1 FROM DUAL) TMP_TABLE WHERE 1 != 1)",
                       "name");
DEFINE_SHOW_CLAUSE_SET(SHOW_PARAMETERS_COMPAT,
                       NULL,
                       "SELECT svr_type, name, data_type, value, info, section from %s.%s where name not like '\\_%%'",
                       R"(SELECT "SVR_TYPE", "NAME", "DATA_TYPE", "VALUE", "INFO", "SECTION" FROM %s.%s WHERE NAME NOT LIKE '\\_%%')",
                       "name");
DEFINE_SHOW_CLAUSE_SET(SHOW_SESSION_STATUS,
                       NULL,
                       "select variable_name as Variable_name, variable_value as Value from %s.%s",
                       NULL,
                       "Variable_name");
DEFINE_SHOW_CLAUSE_SET(SHOW_GLOBAL_STATUS,
                       NULL,
                       "select variable_name as Variable_name, variable_value as Value from %s.%s",
                       NULL,
                       "Variable_name");
DEFINE_SHOW_CLAUSE_SET(SHOW_DATABASES,
                       NULL,
                       "SELECT `database_name` AS `Database` FROM %s.%s  WHERE in_recyclebin = 0 and database_name not in('%s', '%s', '%s') and 0 = sys_privilege_check(\'db_acc\', `database_name`, \'\') order by database_name asc",
                       NULL,
                       "Database");
DEFINE_SHOW_CLAUSE_SET(SHOW_DATABASES_LIKE,
                       "SELECT `Database` AS `Database (%.*s)` ",
                       "SELECT `database_name` AS `Database` FROM %s.%s  WHERE in_recyclebin = 0 and database_name not in ('%s', '%s', '%s') and 0 = sys_privilege_check(\'db_acc\', `database_name`, \'\') order by database_name asc",
                       NULL,
                       "Database");
DEFINE_SHOW_CLAUSE_SET(SHOW_DATABASES_STATUS,
                       NULL,
                       "select db as `Database`, case when sum(read_only) = 0 then \'read write\' when sum(read_only) < count(read_only) then \'partially read only\' else \'read only\' end as `Status` from %s.%s where (0 = sys_privilege_check('db_acc') or 0 = sys_privilege_check('db_acc', `db`, '')) group by db",
                       NULL,
                       "Database");
DEFINE_SHOW_CLAUSE_SET(SHOW_DATABASES_STATUS_LIKE,
                       "SELECT `Database` AS `Database (%.*s)`, `Status` ",
                       "select db as `Database`, case when sum(read_only) = 0 then \'read write\' when sum(read_only) < count(read_only) then \'partially read only\' else \'read only\' end as `Status` from %s.%s where (0 = sys_privilege_check('db_acc') or 0 = sys_privilege_check('db_acc', `db`, '')) group by db",
                       NULL,
                       "Database");
DEFINE_SHOW_CLAUSE_SET(SHOW_CREATE_TABLE,
                       NULL,
                       "SELECT table_name AS `Table`, create_table AS `Create Table` FROM %s.%s  WHERE table_id = %ld",
                       R"(SELECT "TABLE_NAME" AS "TABLE", "CREATE_TABLE" AS "CREATE TABLE" FROM %s.%s  WHERE TABLE_ID = %ld)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_CREATE_VIEW,
                       NULL,
                       "SELECT table_name AS `View`, create_table AS `Create View` , character_set_client, collation_connection FROM %s.%s WHERE table_id = %ld",
                       R"(SELECT "TABLE_NAME" AS "VIEW", "CREATE_TABLE" AS "CREATE VIEW" , "CHARACTER_SET_CLIENT", "COLLATION_CONNECTION" FROM %s.%s WHERE TABLE_ID = %ld)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_CREATE_PROCEDURE,
                       NULL,
                       "SELECT routine_name AS `Procedure`, sql_mode, create_routine AS `Create Procedure`, character_set_client, collation_connection, collation_database AS `Database Collation` FROM %s.%s  WHERE routine_id = %ld and proc_type = %ld",
                       R"(SELECT "ROUTINE_NAME" AS "PROCEDURE", "SQL_MODE", "CREATE_ROUTINE" AS "CREATE PROCEDURE", "CHARACTER_SET_CLIENT", "COLLATION_CONNECTION", "COLLATION_DATABASE" AS "DATABASE COLLATION" FROM %s.%s  WHERE ROUTINE_ID = %ld AND PROC_TYPE = %ld)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_CREATE_FUNCTION,
                       NULL,
                       "SELECT routine_name AS `Function`, sql_mode, create_routine AS `Create Function`, character_set_client, collation_connection, collation_database AS `Database Collation` FROM %s.%s  WHERE routine_id = %ld and proc_type = %ld",
                       R"(SELECT "ROUTINE_NAME" AS "FUNCTION", "SQL_MODE", "CREATE_ROUTINE" AS "CREATE FUNCTION", "CHARACTER_SET_CLIENT", "COLLATION_CONNECTION", "COLLATION_DATABASE" AS "DATABASE COLLATION" FROM %s.%s  WHERE ROUTINE_ID = %ld AND PROC_TYPE = %ld)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_CREATE_TRIGGER,
                       NULL,
                       "SELECT trigger_name AS `Trigger`, sql_mode, create_trigger AS `SQL Original Statement`, character_set_client, collation_connection, collation_database AS `Database Collation` FROM %s.%s  WHERE trigger_id = %ld",
                       R"(SELECT trigger_name AS "TRIGGER", "SQL_MODE", "CREATE_TRIGGER", "CHARACTER_SET_CLIENT", "COLLATION_CONNECTION", "COLLATION_DATABASE" FROM %s.%s  WHERE trigger_id = %ld)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_RECYCLEBIN,
                       "SELECT OBJECT_NAME, ORIGINAL_NAME, TYPE, CREATETIME",
                       "SELECT OBJECT_NAME, ORIGINAL_NAME, case TYPE when 1 then 'TABLE' when 2 then 'INDEX' when 3 then 'VIEW' when 4 then 'DATABASE' when 5 then 'AUX_VP' when 6 then 'TRIGGER' else 'INVALID' end as TYPE, gmt_create as CREATETIME FROM %s.%s WHERE TYPE != 7 AND TYPE != 8 AND TYPE != 9",
                       R"(SELECT "OBJECT_NAME", "ORIGINAL_NAME", CASE "TYPE" WHEN 1 THEN 'TABLE' WHEN 2 THEN 'INDEX' WHEN 3 THEN 'VIEW' WHEN 4 THEN 'DATABASE' when 5 then 'AUX_VP' when 6 then 'TRIGGER' ELSE 'INVALID' END AS "TYPE", "GMT_CREATE" AS "CREATETIME" FROM %s.%s WHERE TYPE != 7 AND TYPE != 8 AND TYPE != 9)",
                       NULL);
DEFINE_SHOW_CLAUSE_SET(SHOW_CREATE_USER,
                       NULL,
                       "SELECT \"%.*s\" AS `CREATE USER for %.*s@%.*s` FROM DUAL",
                       NULL,
                       NULL);
}/* ns sql*/
}/* ns oceanbase */
