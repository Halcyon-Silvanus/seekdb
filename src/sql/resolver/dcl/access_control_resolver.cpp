/*
 * Copyright (c) 2026 OceanBase.
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

#include <cctype>

#include "sql/resolver/dcl/access_control_resolver.h"
#include "sql/resolver/dcl/access_control_common.h"
#include "sql/resolver/dcl/ob_create_user_stmt.h"
#include "sql/resolver/dcl/ob_drop_user_stmt.h"
#include "sql/resolver/dcl/ob_set_password_stmt.h"
#include "sql/engine/expr/ob_expr_validate_password_strength.h"
#include "sql/ob_sql_utils.h"

namespace oceanbase::sql
{
using namespace common;
using namespace share;
using namespace share::schema;
namespace access_control
{
int copy_text(rust::Str text, ObResolverParams &params, ObString &copy)
{
  int ret = OB_SUCCESS;
  if (text.empty()) {
    copy = ObString::make_string("");
  } else {
    ret = ob_write_string(*params.allocator_,
                          ObString(static_cast<int32_t>(text.size()), text.data()), copy);
  }
  return ret;
}

int resolve_account(const peg::Node &spec, ObResolverParams &params, AccountNameMode mode,
                    ObString &name, ObString &host)
{
  int ret = OB_SUCCESS;
  const bool current = peg::account_kind(spec) == peg::AccountKind::Current;
  if (current) {
    OZ (ob_write_string(*params.allocator_, params.session_info_->get_user_name(), name));
    OZ (ob_write_string(*params.allocator_, params.session_info_->get_host_name(), host));
  } else {
    OZ (copy_text(peg::account_name(spec), params, name));
    if (OB_SUCC(ret)) {
      if (peg::account_has_host(spec)) {
        OZ (copy_text(peg::account_host(spec), params, host));
      } else {
        host = ObString::make_string(OB_DEFAULT_HOST_NAME);
      }
    }
  }
  if (OB_SUCC(ret) && mode == AccountNameMode::Schema
      && (current || peg::account_name_form(spec) != peg::AccountNameForm::Identifier)) {
    OZ (ObSQLUtils::convert_sql_text_to_schema_for_storing(
        *params.allocator_, params.session_info_->get_dtc_params(), name));
  }
  return ret;
}

int check_plugin(const peg::Node &spec, ObResolverParams &params, bool case_sensitive)
{
  int ret = OB_SUCCESS;
  if (peg::account_has_plugin(spec)) {
    const auto text = peg::account_plugin(spec);
    const ObString plugin(static_cast<int32_t>(text.size()), text.data());
    ObString default_plugin;
    if (OB_FAIL(params.session_info_->get_sys_variable(
            SYS_VAR_DEFAULT_AUTHENTICATION_PLUGIN, default_plugin))) {
    } else if ((case_sensitive ? plugin.compare(default_plugin)
                               : plugin.case_compare(default_plugin)) != 0) {
      ret = OB_ERR_PLUGIN_IS_NOT_LOADED;
      LOG_USER_ERROR(OB_ERR_PLUGIN_IS_NOT_LOADED, plugin.length(), plugin.ptr());
    }
  }
  return ret;
}

int resolve_role_account(const peg::Node &spec, ObResolverParams &params,
                         ObString &name, ObString &host)
{
  int ret = OB_SUCCESS;
  if (OB_FAIL(resolve_account(spec, params, AccountNameMode::Schema, name, host))) {
  } else if (!params.session_info_->is_inner() && name.case_compare(OB_RESTORE_USER_NAME) == 0) {
    ret = OB_ERR_NO_PRIVILEGE;
    LOG_WARN("restore user is reserved", K(ret));
  } else {
    OZ (check_plugin(spec, params, false));
  }
  return ret;
}

int check_reserved_account(ObResolverParams &params, uint64_t user_id)
{
  int ret = OB_SUCCESS;
  const uint64_t session_user_id = params.session_info_->get_priv_user_id();
  if (user_id == OB_INVALID_ID || session_user_id == OB_INVALID_ID) {
    ret = OB_ERR_UNEXPECTED;
  } else if (user_id == OB_SYS_USER_ID && session_user_id != OB_SYS_USER_ID) {
    ret = OB_NOT_SUPPORTED;
    LOG_USER_ERROR(OB_NOT_SUPPORTED, "modify on reserved user");
  }
  return ret;
}

int check_reserved_account(ObResolverParams &params, const ObString &name, const ObString &host)
{
  int ret = OB_SUCCESS;
  uint64_t user_id = OB_INVALID_ID;
  if (name.empty() || params.session_info_->get_priv_user_id() == OB_INVALID_ID) {
    ret = OB_ERR_UNEXPECTED;
  } else if (OB_FAIL(params.schema_checker_->get_user_id(name, host, user_id))) {
    if (ret == OB_USER_NOT_EXIST) {
      ret = OB_SUCCESS;
    }
  } else {
    OZ (check_reserved_account(params, user_id));
  }
  return ret;
}

int resolve_dropped_account(const peg::Node &spec, ObResolverParams &params, bool protect_root,
                            ObString &name, ObString &host)
{
  int ret = OB_SUCCESS;
  const ObUserInfo *user = nullptr;
  if (OB_FAIL(resolve_account(spec, params, AccountNameMode::Schema, name, host))) {
  } else if (OB_FAIL(params.schema_checker_->get_user_info(name, host, user))) {
    if (ret == OB_USER_NOT_EXIST) {
      ret = OB_SUCCESS;
    }
  } else if (OB_ISNULL(user)) {
    ret = OB_ERR_UNEXPECTED;
  } else if (is_inner_user_or_role(user->get_user_id())) {
    ret = OB_ERR_NO_PRIVILEGE;
  } else if (protect_root) {
    OZ (check_reserved_account(params, user->get_user_id()));
  }
  return ret;
}

int check_password_strength(ObString &password, ObResolverParams &params)
{
  int ret = OB_SUCCESS;
  int64_t policy = 0;
  int64_t check_user_name = 0;
  const size_t char_len = password.empty() ? 0 : ObCharset::strlen_char(ObCharset::get_system_collation(),
                                                password.ptr(), password.length());
  bool passed = true;
  if (OB_FAIL(params.session_info_->get_sys_variable(SYS_VAR_VALIDATE_PASSWORD_POLICY, policy))) {
  } else if (OB_FAIL(params.session_info_->get_sys_variable(
          SYS_VAR_VALIDATE_PASSWORD_CHECK_USER_NAME, check_user_name))) {
  } else if (!check_user_name
             && ObCharset::case_insensitive_equal(password, params.session_info_->get_user_name())) {
    ret = OB_ERR_NOT_VALID_PASSWORD;
  } else if (OB_FAIL(ObExprValidatePasswordStrength::validate_password_low(
          password, char_len, *params.session_info_, passed))) {
  } else if (passed && policy == 1) {
    OZ (ObExprValidatePasswordStrength::validate_password_medium(
        password, char_len, *params.session_info_, passed));
  } else if (passed && policy != 0) {
    ret = OB_ERR_UNEXPECTED;
  }
  if (OB_SUCC(ret) && !passed) {
    ret = OB_ERR_NOT_VALID_PASSWORD;
  }
  return ret;
}

bool valid_encoded_password(const ObString &password)
{
  bool valid = password.length() == 41 && password[0] == '*';
  for (int32_t i = 1; valid && i < password.length(); ++i) {
    valid = std::isxdigit(static_cast<unsigned char>(password[i]));
  }
  return valid;
}

namespace
{
int resolve_create_user(const peg::Node &node, ObResolverParams &params, ObStmt *&stmt)
{
  int ret = OB_SUCCESS;
  ObCreateUserStmt *created = nullptr;
  OZ (create_resolved_statement(params, created));
  if (OB_SUCC(ret)) {
    created->set_if_not_exists(peg::access_options(node).if_not_exists);
    for (const auto &account : peg::access_accounts(node)) {
      if (OB_FAIL(ret)) { break; }
      ObString name;
      ObString host;
      ObString password;
      ObString need_enc = ObString::make_string("NO");
      OZ (resolve_role_account(account, params, name, host));
      if (OB_SUCC(ret) && name.empty()) {
        ret = OB_CANNOT_USER;
        LOG_USER_ERROR(OB_CANNOT_USER, 11, "CREATE USER", host.length(), host.ptr());
      }
      const auto authentication = peg::account_authentication(account);
      if (OB_SUCC(ret) && authentication != peg::AuthenticationKind::Absent) {
        OZ (copy_text(peg::account_password(account), params, password));
        if (OB_SUCC(ret) && authentication == peg::AuthenticationKind::Encoded) {
          if (!valid_encoded_password(password)) {
            ret = OB_ERR_PASSWORD_FORMAT;
            LOG_WARN("invalid password format", K(ret));
          }
        } else if (OB_SUCC(ret)) {
          need_enc = ObString::make_string("YES");
        }
      }
      if (OB_SUCC(ret) && authentication != peg::AuthenticationKind::Encoded) {
        OZ (check_password_strength(password, params));
      }
      if (OB_SUCC(ret)) {
        if (name.length() > OB_MAX_USER_NAME_LENGTH) {
          ret = OB_WRONG_USER_NAME_LENGTH;
          LOG_USER_ERROR(OB_WRONG_USER_NAME_LENGTH, name.length(), name.ptr());
        } else {
          OZ (created->add_user(name, host, password, need_enc));
        }
      }
    }
    OZ (created->add_ssl_info(get_ssl_type_string(ObSSLType::SSL_TYPE_NOT_SPECIFIED), {}, {}, {}));
  }
  if (OB_SUCC(ret)) { stmt = created; }
  return ret;
}

int resolve_drop_user(const peg::Node &node, ObResolverParams &params, ObStmt *&stmt)
{
  int ret = OB_SUCCESS;
  ObDropUserStmt *dropped = nullptr;
  OZ (create_resolved_statement(params, dropped));
  if (OB_SUCC(ret)) {
    dropped->set_if_exists(peg::access_options(node).if_exists);
    for (const auto &account : peg::access_accounts(node)) {
      if (OB_FAIL(ret)) { break; }
      ObString name;
      ObString host;
      OZ (resolve_dropped_account(account, params, true, name, host));
      OZ (dropped->add_user(name, host));
    }
  }
  if (OB_SUCC(ret)) { stmt = dropped; }
  return ret;
}

int resolve_password(const peg::Node &node, ObResolverParams &params, ObStmt *&stmt)
{
  int ret = OB_SUCCESS;
  ObSetPasswordStmt *changed = nullptr;
  OZ (create_resolved_statement(params, changed));
  if (OB_SUCC(ret)) {
    const auto &account = peg::access_accounts(node)[0];
    ObString name;
    ObString host;
    ObString password;
    OZ (check_plugin(account, params, false));
    OZ (resolve_account(account, params, AccountNameMode::Verbatim, name, host));
    if (OB_SUCC(ret) && name.empty() && peg::account_kind(account) == peg::AccountKind::Named) {
      ret = OB_USER_NOT_EXIST;
      LOG_WARN("empty user cannot be used by SET PASSWORD", K(ret));
    }
    if (OB_SUCC(ret)) {
      changed->set_for_current_user(name == params.session_info_->get_user_name()
                                    && host == params.session_info_->get_host_name());
    }
    OZ (check_reserved_account(params, name, host));
    OZ (copy_text(peg::account_password(account), params, password));
    OZ (check_password_strength(password, params));
    if (OB_SUCC(ret)) {
      changed->set_need_enc(!password.empty());
      OZ (changed->set_user_password(name, host, password));
      OZ (changed->add_ssl_info(get_ssl_type_string(ObSSLType::SSL_TYPE_NOT_SPECIFIED), {}, {}, {}));
    }
  }
  if (OB_SUCC(ret)) { stmt = changed; }
  return ret;
}
}
}

int resolve_access_control(const peg::Node &node, ObResolverParams &params, ObStmt *&stmt)
{
  int ret = OB_SUCCESS;
  if (OB_ISNULL(params.session_info_) || OB_ISNULL(params.schema_checker_)
      || OB_ISNULL(params.allocator_) || OB_ISNULL(params.stmt_factory_)
      || OB_ISNULL(params.query_ctx_)) {
    ret = OB_NOT_INIT;
  } else {
    switch (node.kind()) {
      case peg::NodeKind::CreateUser:
        ret = access_control::resolve_create_user(node, params, stmt);
        break;
      case peg::NodeKind::DropUser:
        ret = access_control::resolve_drop_user(node, params, stmt);
        break;
      case peg::NodeKind::SetPassword:
        ret = access_control::resolve_password(node, params, stmt);
        break;
      case peg::NodeKind::Grant:
      case peg::NodeKind::Revoke:
      case peg::NodeKind::RevokeAll:
        ret = access_control::resolve_privileges(node, params, stmt);
        break;
      case peg::NodeKind::CreateRole:
      case peg::NodeKind::DropRole:
      case peg::NodeKind::GrantRole:
      case peg::NodeKind::RevokeRole:
      case peg::NodeKind::SetRole:
      case peg::NodeKind::SetDefaultRole:
      case peg::NodeKind::AlterUserDefaultRole:
        ret = access_control::resolve_roles(node, params, stmt);
        break;
      default:
        ret = OB_INVALID_ARGUMENT;
    }
  }
  return ret;
}
}
