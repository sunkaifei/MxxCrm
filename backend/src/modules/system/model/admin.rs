//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::Serialize;
use crate::modules::system::entity::{admin, admin::Entity as Admin, admin_dept_merge, dept, dept::Entity as Dept};
use crate::utils::string_utils::{deserialize_string_to_i32, deserialize_string_to_u64, deserialize_string_vec_to_u64_vec, serialize_option_u64_to_string, u64_to_string};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTime;
use sea_orm::*;
use sea_query::{CommonTableExpression, Expr, IntoTableRef, Query, UnionType};
use serde::Deserialize;
use validator::Validate;
use crate::modules::website::model::website::WebSiteVO;

#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminSaveRequest {
    ///用户账号
    pub user_name: Option<String>,
    ///用户昵称
    pub nick_name: Option<String>,
    ///用户类型：0普通用户，1超级管理员
    pub user_type: Option<i32>,
    ///岗位
    #[serde(deserialize_with = "deserialize_string_vec_to_u64_vec", default)]
    pub post_ids: Option<Vec<i64>>,
    ///角色
    #[serde(deserialize_with = "deserialize_string_vec_to_u64_vec", default)]
    pub role_ids: Option<Vec<i64>>,
    ///部门
    #[serde(deserialize_with = "deserialize_string_vec_to_u64_vec", default)]
    pub dept_ids: Option<Vec<i64>>,
    ///用户邮箱
    pub email: Option<String>,
    ///手机号码
    pub mobile: Option<String>,
    ///用户性别（0男 1女 2未知）
    #[validate(range(min = 0, max = 2))]
    #[serde(default, deserialize_with = "deserialize_string_to_i32")]
    pub gender: Option<i32>,
    ///头像地址
    pub avatar: Option<String>,
    ///密码
    pub password: Option<String>,
    ///帐号状态（0停用 1正常）
    #[serde(default)]
    pub status: Option<i32>,
    ///备注
    pub remark: Option<String>,
    ///用户排序
    #[serde(default)]
    pub sort: Option<i32>,
    ///直属上级用户ID（用于审批流向上查找领导，NULL 表示无上级/顶层决策人）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub direct_manager_id: Option<i64>,
    ///入职时间
    #[serde(default)]
    pub hire_date: Option<NaiveDate>,
    ///是否参与工资核算：0不参与 1参与（默认参与）
    #[serde(default)]
    pub salary_enabled: Option<i32>,
    ///试用期月数（0或空=无试用期）
    #[serde(default)]
    pub probation_months: Option<i32>,
    ///试用期工资比例（如0.60=60%，空=不打折）
    #[serde(default)]
    pub probation_ratio: Option<Decimal>,
}


impl From<AdminSaveRequest> for AdminSaveDTO {
    fn from(req: AdminSaveRequest) -> Self {
        // 用户名统一转小写、去空格
        let normalized_username = req.user_name.map(|n| n.trim().replace(' ', "").to_lowercase());
        AdminSaveDTO {
            id: None,
            user_name: normalized_username,
            nick_name: req.nick_name,
            user_type: req.user_type,
            email: req.email,
            mobile: req.mobile,
            gender: req.gender,
            avatar: req.avatar,
            password: req.password,
            status: req.status,
            deleted: None,
            login_ip: None,
            login_date: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            remark: req.remark,
            sort: req.sort,
            direct_manager_id: req.direct_manager_id,
            hire_date: req.hire_date,
            salary_enabled: req.salary_enabled,
            probation_months: req.probation_months,
            probation_ratio: req.probation_ratio,
        }
    }
}

#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminUpdateRequest {
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    #[validate(length(min = 1, max = 38))]
    pub user_name: Option<String>,
    #[validate(range(min = 0, max = 1))]
    pub user_type: Option<i32>,
    #[validate(length(min = 1, max = 38))]
    pub nick_name: Option<String>,
    ///手机号码
    pub mobile: Option<String>,
    ///岗位
    #[serde(deserialize_with = "deserialize_string_vec_to_u64_vec", default)]
    pub post_ids: Option<Vec<i64>>,
    ///角色
    #[serde(deserialize_with = "deserialize_string_vec_to_u64_vec", default)]
    pub role_ids: Option<Vec<i64>>,
    ///部门
    #[serde(deserialize_with = "deserialize_string_vec_to_u64_vec", default)]
    pub dept_ids: Option<Vec<i64>>,
    ///头像地址
    pub avatar: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    ///用户性别（0男 1女 2未知）
    #[validate(range(min = 0, max = 2))]
    #[serde(default, deserialize_with = "deserialize_string_to_i32")]
    pub gender: Option<i32>,
    #[serde(default)]
    pub sort: Option<i32>,
    #[serde(default)]
    pub status: Option<i32>,
    pub remark: Option<String>,
    ///直属上级用户ID（用于审批流向上查找领导，NULL 表示无上级/顶层决策人）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub direct_manager_id: Option<i64>,
    ///入职时间
    #[serde(default)]
    pub hire_date: Option<NaiveDate>,
    ///是否参与工资核算：0不参与 1参与（默认参与）
    #[serde(default)]
    pub salary_enabled: Option<i32>,
    ///试用期月数（0或空=无试用期）
    #[serde(default)]
    pub probation_months: Option<i32>,
    ///试用期工资比例（如0.60=60%，空=不打折）
    #[serde(default)]
    pub probation_ratio: Option<Decimal>,
}

impl From<AdminUpdateRequest> for AdminSaveDTO {
    fn from(req: AdminUpdateRequest) -> Self {
        // 用户名统一转小写、去空格
        let normalized_username = req.user_name.map(|n| n.trim().replace(' ', "").to_lowercase());
        AdminSaveDTO {
            id: req.id,
            user_name: normalized_username,
            nick_name: req.nick_name,
            user_type: req.user_type,
            email: req.email,
            mobile: req.mobile,
            gender: req.gender,
            avatar: req.avatar,
            password: None,
            status: req.status,
            deleted: None,
            login_ip: None,
            login_date: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            remark: req.remark,
            sort: req.sort,
            direct_manager_id: req.direct_manager_id,
            hire_date: req.hire_date,
            salary_enabled: req.salary_enabled,
            probation_months: req.probation_months,
            probation_ratio: req.probation_ratio,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdminSaveDTO {
    ///用户ID
    pub id: Option<i64>,
    ///用户账号
    pub user_name: Option<String>,
    ///用户昵称
    pub nick_name: Option<String>,
    ///用户类型：0普通用户，1超级管理员
    pub user_type: Option<i32>,
    ///用户邮箱
    pub email: Option<String>,
    ///手机号码
    pub mobile: Option<String>,
    ///用户性别（0男 1女 2未知）
    pub gender: Option<i32>,
    ///头像地址
    pub avatar: Option<String>,
    ///密码
    pub password: Option<String>,
    ///帐号状态（0停用 1正常）
    pub status: Option<i32>,
    ///删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
    ///最后登陆IP
    pub login_ip: Option<String>,
    ///最后登陆时间
    pub login_date: Option<DateTime>,
    ///创建者
    pub create_by: Option<String>,
    ///创建时间
    pub create_time: Option<DateTime>,
    ///更新者
    pub update_by: Option<String>,
    ///更新时间
    pub update_time: Option<DateTime>,
    ///备注
    pub remark: Option<String>,
    ///用户排序
    pub sort: Option<i32>,
    ///直属上级用户ID（用于审批流向上查找领导，NULL 表示无上级/顶层决策人）
    pub direct_manager_id: Option<i64>,
    ///入职时间
    pub hire_date: Option<NaiveDate>,
    ///是否参与工资核算：0不参与 1参与
    pub salary_enabled: Option<i32>,
    ///试用期月数（0或空=无试用期）
    pub probation_months: Option<i32>,
    ///试用期工资比例（如0.60=60%，空=不打折）
    pub probation_ratio: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLoginRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    //验证码
    pub captcha_code: Option<String>,
    //验证码凭证，用于验证码校验
    pub captcha_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRegisterRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    ///姓名（写入 nick_name）
    pub nick_name: Option<String>,
    ///申请部门（文本，写入 remark）
    pub dept_name: Option<String>,
    ///申请岗位（文本，写入 remark）
    pub post_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckUsernameResult {
    pub exists: bool,
    pub message: String,
}

impl From<UserRegisterRequest> for AdminSaveRequest {
    fn from(req: UserRegisterRequest) -> Self {
        let username = req.username.clone().unwrap_or_default();
        // 将申请部门/岗位拼入 remark，审核时管理员可见
        let dept = req.dept_name.unwrap_or_default();
        let post = req.post_name.unwrap_or_default();
        let remark = format!("申请部门：{}；申请岗位：{}", dept, post);
        AdminSaveRequest {
            user_name: req.username,
            nick_name: req.nick_name.or(Some(username)),
            user_type: Some(0),
            post_ids: None,
            role_ids: None,
            dept_ids: None,
            email: req.email,
            mobile: req.mobile,
            gender: None,
            avatar: None,
            password: req.password,
            status: Some(0), // 待审核，审核通过后改为1
            remark: Some(remark),
            sort: None,
            direct_manager_id: None,
            hire_date: None,
            salary_enabled: Some(1),
            probation_months: None,
            probation_ratio: None,
        }
    }
}

///重置用户密码结构体
#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateResetPasswordRequest {
    /// 用户id
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub user_id: Option<i64>,
    #[validate(length(min = 3, max = 38))]
    pub password: Option<String>,
}

///用户更新密码结构体
#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAdminPasswordRequest {
    /// 用户id
    #[validate(length(min = 3, max = 38))]
    pub old_password: Option<String>,
    #[validate(length(min = 3, max = 38))]
    pub new_password: Option<String>,
    #[validate(length(min = 3, max = 38))]
    pub confirm_password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLoginRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub login_ip: Option<String>,
    pub login_date: Option<DateTime>,
}

impl From<UpdateLoginRequest> for AdminSaveDTO {
    fn from(req: UpdateLoginRequest) -> Self {
        AdminSaveDTO {
            id: req.id,
            user_name: None,
            nick_name: None,
            user_type: None,
            email: None,
            mobile: None,
            gender: None,
            avatar: None,
            password: None,
            status: None,
            deleted: None,
            login_ip: req.login_ip,
            login_date: None,
            create_by: None,
            create_time: None,
            update_by: None,
            update_time: None,
            remark: None,
            sort: None,
            direct_manager_id: None,
            hire_date: None,
            salary_enabled: None,
            probation_months: None,
            probation_ratio: None,
        }
    }
}



#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAdminStatusRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub status: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAdminRoleRequest {
    pub admin_id: Option<i64>,
    pub role_ids: Vec<i64>,
    pub create_time: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenVO {
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub role: Vec<String>,
}

/// 用户角色名称DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoleNameDTO {
    ///角色名称
    pub role_name: Option<String>,
}

/// 用户组织名称DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeptNameDTO {
    ///组织名称
    pub dept_name: Option<String>,
}

/// 用户岗位名称DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostNameDTO {
    ///岗位名称
    pub post_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminOptionVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub value: Option<i64>,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserLoginVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    /// 数据权限范围（1全部 2自定义 3本部门 4本部门及以下 5仅本人）
    pub data_scope: Option<i32>,
    /// 用户岗位名称列表（用于工作台等按职能个性化）
    pub post_names: Vec<String>,
    /// 用户部门名称列表
    pub dept_names: Vec<String>,
}

/// 管理员列表展示数据
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminListVO {
    #[serde(serialize_with = "u64_to_string")]
    pub id:  i64,
    pub user_name: Option<String>,
    ///用户姓名
    pub nick_name: Option<String>,
    ///用户类型：0普通用户，1超级管理员
    pub user_type: Option<i32>,
    pub mobile: Option<String>,
    pub email: Option<String>,
    ///角色名称（逗号分隔）
    pub role_name: Option<String>,
    ///所有拥有的权限组名称
    pub roles: Option<Vec<RoleNameDTO>>,
    ///所有的所在部门名称
    pub depts: Option<Vec<DeptNameDTO>>,
    ///部门名称（逗号分隔）
    pub dept_name: Option<String>,
    ///所有的岗位名称
    pub posts: Option<Vec<PostNameDTO>>,
    ///岗位名称（逗号分隔）
    pub post_name: Option<String>,
    pub remark: Option<String>,
    pub sort:  Option<i32>,
    pub status:  Option<i32>,
    #[serde(rename = "lastLoginIp")]
    pub login_ip: Option<String>,
    #[serde(rename = "lastLoginTime")]
    pub login_date: Option<String>,
    pub create_time: Option<String>,
    ///直属上级用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub direct_manager_id: Option<i64>,
    ///直属上级姓名
    pub direct_manager_name: Option<String>,
    ///是否在线（基于登录会话缓存判断）
    #[serde(default)]
    pub online: bool,
    ///审核状态：0待审核 1已通过
    #[serde(default)]
    pub audit_status: i32,
    ///入职时间
    pub hire_date: Option<NaiveDate>,
    ///是否参与工资核算：0不参与 1参与
    #[serde(default)]
    pub salary_enabled: i32,
    ///试用期月数（0或空=无试用期）
    pub probation_months: Option<i32>,
    ///试用期工资比例（如0.60=60%，空=不打折）
    pub probation_ratio: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AdminDetailVO {
    ///用户ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    ///用户账号
    pub user_name: Option<String>,
    ///用户昵称
    pub nick_name: Option<String>,
    ///用户类型：0普通用户，1超级管理员
    pub user_type: Option<i32>,
    ///部门ID
    pub dept_ids: Option<Vec<Option<String>>>,
    ///部门名称列表（与deptIds顺序对应）
    pub dept_names: Option<Vec<Option<String>>>,
    ///岗位ID
    pub post_ids: Option<Vec<Option<String>>>,
    ///岗位名称列表（与postIds顺序对应）
    pub post_names: Option<Vec<Option<String>>>,
    ///角色ID
    pub role_ids: Option<Vec<Option<String>>>,
    ///角色名称列表（与roleIds顺序对应）
    pub role_names: Option<Vec<Option<String>>>,
    ///用户邮箱
    pub email: Option<String>,
    ///手机号码
    pub mobile: Option<String>,
    ///用户性别（0男 1女 2未知）
    pub gender: Option<i32>,
    ///头像地址
    pub avatar: Option<String>,
    ///帐号状态（0停用 1正常）
    pub status: Option<i32>,
    ///删除标志（0未删除 1已删除）
    pub deleted: Option<i32>,
    ///最后登陆IP
    pub login_ip: Option<String>,
    ///最后登陆时间
    pub login_date: Option<DateTime>,
    ///创建者
    pub create_by: Option<String>,
    ///创建时间
    pub create_time: Option<String>,
    ///更新者
    pub update_by: Option<String>,
    ///更新时间
    pub update_time: Option<DateTime>,
    ///备注
    pub remark: Option<String>,
    ///用户排序
    pub sort: Option<i32>,
    ///直属上级用户ID（用于审批流向上查找领导，NULL 表示无上级/顶层决策人）
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub direct_manager_id: Option<i64>,
    ///直属上级姓名（用于前端展示）
    pub direct_manager_name: Option<String>,
    ///入职时间
    pub hire_date: Option<NaiveDate>,
    ///是否参与工资核算：0不参与 1参与
    pub salary_enabled: Option<i32>,
    ///试用期月数（0或空=无试用期）
    pub probation_months: Option<i32>,
    ///试用期工资比例（如0.60=60%，空=不打折）
    pub probation_ratio: Option<Decimal>,
}

impl From<admin::Model> for AdminDetailVO {
    fn from(model: admin::Model) -> Self {
        Self {
            id: Option::from(model.id),
            user_name: model.user_name,
            nick_name: model.nick_name,
            user_type: model.user_type,
            dept_ids: None,
            dept_names: None,
            post_ids: None,
            post_names: None,
            role_ids: None,
            role_names: None,
            email: model.email,
            mobile: model.mobile,
            gender: model.gender,
            avatar: model.avatar,
            status: model.status,
            deleted: model.deleted,
            login_ip: model.login_ip,
            login_date: model.login_date,
            create_by: model.create_by,
            create_time: model.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_by: model.update_by,
            update_time: model.update_time,
            remark: model.remark,
            sort: model.sort,
            direct_manager_id: model.direct_manager_id,
            direct_manager_name: None,
            hire_date: model.hire_date,
            salary_enabled: model.salary_enabled,
            probation_months: model.probation_months,
            probation_ratio: model.probation_ratio,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub user_name: Option<String>,
    pub nick_name: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub user_type: Option<i32>,
    pub status: Option<i32>,
    pub dept_id: Option<i64>,
}

/// 条件
#[derive(Clone)]
pub struct PageWhere {
    pub user_name: Option<String>,
    pub nick_name: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub user_type: Option<i32>,
    pub status: Option<i32>,
    pub dept_id: Option<i64>,
}

impl PageWhere {
    /// 格式化
    pub fn format(&self) -> Self {
        let mut user_name = None;
        if self.user_name != Some("".to_string()) {
            user_name = self.user_name.clone();
        }

        let mut nick_name = None;
        if self.nick_name != Some("".to_string()) {
            nick_name = self.nick_name.clone();
        }

        let mut email = None;
        if self.email != Some("".to_string()) {
            email = self.email.clone();
        }

        let mut mobile = None;
        if self.mobile != Some("".to_string()) {
            mobile = self.mobile.clone();
        }

        let mut user_type = None;
        if self.user_type == Some(1) || self.user_type == Some(0) {
            user_type = self.user_type;
        }

        let mut status = None;
        if self.status == Some(1) || self.status == Some(0) {
            status = self.status;
        }

        let mut dept_id = None;
        if self.dept_id != Some(0) {
            dept_id = self.dept_id;
        }

        Self {
            user_name,
            nick_name,
            email,
            mobile,
            user_type,
            status,
            dept_id,
        }
    }
}

pub struct AdminModel;

impl AdminModel {
    pub async fn insert<C: ConnectionTrait>(db: &C, form_data: &AdminSaveDTO) -> Result<i64, DbErr> {
        let payload = admin::ActiveModel{
            user_name:       Set(form_data.user_name.to_owned()),
            nick_name:       Set(form_data.nick_name.to_owned()),
            user_type:       Set(form_data.user_type.to_owned()),
            email:           Set(form_data.email.to_owned()),
            mobile:          Set(form_data.mobile.to_owned()),
            gender:             Set(form_data.gender.to_owned()),
            avatar:          Set(form_data.avatar.to_owned()),
            password:        Set(form_data.password.to_owned()),
            status:          Set(form_data.status.to_owned()),
            direct_manager_id: Set(form_data.direct_manager_id.to_owned()),
            hire_date:       Set(form_data.hire_date.to_owned()),
            salary_enabled:  Set(form_data.salary_enabled.to_owned()),
            probation_months: Set(form_data.probation_months.to_owned()),
            probation_ratio: Set(form_data.probation_ratio.to_owned()),
            create_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        Admin::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// ### 批量删除（硬删除）
    /// * db 数据库连接
    /// * ids: 批量删除的id
    pub async fn batch_delete_by_ids<C: ConnectionTrait>(db: &C, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Ok(Admin::delete_many()
            .filter(admin::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await?.rows_affected as i64)
    }

    /// ### 软删除
    /// * db 数据库连接
    /// * id: 需要删除的id
    pub async fn soft_delete<C: ConnectionTrait>(db: &C, id: i64) -> Result<i64, DbErr> {
        let payload = admin::ActiveModel {
            deleted: Set(Some(1)),
            ..Default::default()
        };
        let update_result = Admin::update_many()
            .set(payload)
            .filter(admin::Column::Id.eq(id))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    /// ### 更新用户信息
    /// * db 数据库连接
    /// * user_id: 用户id  为了防止忘记传递id，所以单独取出来
    /// * form_data: 更新数据
    pub async fn update_admin<C: ConnectionTrait>(
        db: &C,
        user_id: i64,
        form_data: &AdminSaveDTO,
    ) -> Result<i64, DbErr> {
        let mut payload = admin::ActiveModel{
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        if let Some(v) = form_data.user_name.clone() { payload.user_name = Set(Some(v)); }
        if let Some(v) = form_data.nick_name.clone() { payload.nick_name = Set(Some(v)); }
        if let Some(v) = form_data.user_type { payload.user_type = Set(Some(v)); }
        if let Some(v) = form_data.email.clone() { payload.email = Set(Some(v)); }
        if let Some(v) = form_data.mobile.clone() { payload.mobile = Set(Some(v)); }
        if let Some(v) = form_data.gender { payload.gender = Set(Some(v)); }
        if let Some(v) = form_data.avatar.clone() { payload.avatar = Set(Some(v)); }
        if let Some(v) = form_data.status { payload.status = Set(Some(v)); }
        if let Some(v) = form_data.remark.clone() { payload.remark = Set(Some(v)); }
        if let Some(v) = form_data.sort { payload.sort = Set(Some(v)); }
        if let Some(v) = form_data.password.clone() { payload.password = Set(Some(v)); }
        if let Some(v) = form_data.deleted { payload.deleted = Set(Some(v)); }
        if let Some(v) = form_data.hire_date { payload.hire_date = Set(Some(v)); }
        if let Some(v) = form_data.salary_enabled { payload.salary_enabled = Set(Some(v)); }
        if let Some(v) = form_data.probation_months { payload.probation_months = Set(Some(v)); }
        if let Some(v) = form_data.probation_ratio { payload.probation_ratio = Set(Some(v)); }
        // 直属上级：前端传正数表示设置上级，传 0/null 表示清除上级，字段缺失表示不更新
        if let Some(v) = form_data.direct_manager_id {
            if v > 0 {
                payload.direct_manager_id = Set(Some(v));
            } else {
                payload.direct_manager_id = Set(None);
            }
        }

        let update_result: UpdateResult = Admin::update_many()
            .set(payload)
            .filter(admin::Column::Id.eq(user_id))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    /// ### 更新当前登录用户头像
    ///
    /// 仅更新 avatar 字段与 update_time，供“个人中心-更换头像”使用。
    /// 不经过 AdminSaveDTO，避免误改其他字段（如状态、角色、密码）。
    ///
    /// * db 数据库连接
    /// * user_id 当前用户id
    /// * avatar 头像访问地址（含缓存破坏版本号）
    pub async fn update_avatar<C: ConnectionTrait>(
        db: &C,
        user_id: i64,
        avatar: &str,
    ) -> Result<i64, DbErr> {
        let update_result: UpdateResult = Admin::update_many()
            .set(admin::ActiveModel {
                avatar: Set(Some(avatar.to_string())),
                update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
                ..Default::default()
            })
            .filter(admin::Column::Id.eq(user_id))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }

    /// ### 更改密码
    /// * db 数据库连接
    /// * user_id: 用户id
    /// * password: 需要更新密码
    /// 
    /// 返回更新成功数量
    pub async fn update_user_password(
        db: &DbConn,
        user_id: &Option<i64>,
        password: &Option<String>,
    ) -> Result<i64, DbErr> {
        let payload = admin::ActiveModel{
            password:        Set(password.to_owned()),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = Admin::update_many()
            .set(payload)
            .filter(admin::Column::Id.eq(user_id.clone().unwrap_or_default()))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 更新用户状态
    pub async fn update_by_status(
        db: &DbConn,
        user_id: &i64,
        status: &Option<i32>,
    ) -> Result<i64, DbErr> {
        let payload = admin::ActiveModel{
            status:          Set(status.to_owned()),
            update_time:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = Admin::update_many()
            .set(payload)
            .filter(admin::Column::Id.eq(user_id.clone()))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 审核注册用户（通过时 status=1, audit_status=1；拒绝时保持 status=0, audit_status=0）
    pub async fn update_audit_status(
        db: &DbConn,
        user_id: i64,
        audit_status: i32,
    ) -> Result<i64, DbErr> {
        let (audit_val, status_val) = if audit_status == 1 {
            (Some(1), Some(1))
        } else {
            (Some(0), Some(0))
        };
        let payload = admin::ActiveModel {
            audit_status: Set(audit_val),
            status: Set(status_val),
            update_time: Set(Option::from(chrono::Local::now().naive_local())),
            ..Default::default()
        };
        let update_result: UpdateResult = Admin::update_many()
            .set(payload)
            .filter(admin::Column::Id.eq(user_id))
            .exec(db)
            .await?;
        Ok(update_result.rows_affected as i64)
    }
    
    /// 更新登录IP和时间信息
    pub async fn update_login_info(db: &DbConn,
                                   user_id: i64,
                                   form_data: &AdminSaveDTO,
    ) -> Result<i64, DbErr> {
        let payload = admin::ActiveModel{
            login_ip:       Set(form_data.login_ip.to_owned()),
            login_date:     Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };
        let update_result: UpdateResult = Admin::update_many()
            .set(payload)
            .filter(admin::Column::Id.eq(user_id))
            .exec(db).await?;
        Ok(update_result.rows_affected as i64)
    }

    /// 查询用户名是否唯一（不区分大小写）
    pub async fn find_by_name_unique(db: &DbConn, name: &Option<String>, id: &Option<i64>) -> Result<i64, DbErr> {
        let result = Admin::find()
            .filter(admin::Column::UserName.eq(name.clone().unwrap_or_default().to_lowercase()))
            .apply_if(id.clone(), |query, v| {
                query.filter(admin::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(result)
    }

    /// 查询手机号是否唯一
    pub async fn find_by_mobile_unique(db: &DbConn, mobile: &Option<String>, id: Option<i64>)-> Result<i64, DbErr> {
        let result = Admin::find()
            .filter(admin::Column::Mobile.eq(mobile.clone().unwrap_or_default()))
            .apply_if(id, |query, v| {
                query.filter(admin::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(result)
    }

    /// 查询邮箱是否唯一
    pub async fn find_by_email_unique(db: &DbConn, email: &Option<String>, id: Option<i64>)-> Result<i64, DbErr> {
        let result = Admin::find()
            .filter(admin::Column::Email.eq(email.clone().unwrap_or_default()))
            .apply_if(id, |query, v| {
                query.filter(admin::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(result)
    }

    /// 查询昵称是否唯一
    pub async fn find_by_nick_name_unique(db: &DbConn, nick_name: &Option<String>, id: Option<i64>)-> Result<i64, DbErr> {
        let result = Admin::find()
            .filter(admin::Column::NickName.eq(nick_name.clone().unwrap_or_default()))
            .apply_if(id, |query, v| {
                query.filter(admin::Column::Id.ne(v))
            })
            .count(db).await? as i64;
        Ok(result)
    }
    
    /// 根据id查询用户信息
    pub async fn find_by_id(db: &DbConn, id: &Option<i64>) -> Result<Option<admin::Model>, DbErr> {
        Admin::find_by_id(id.clone().unwrap_or_default())
            .one(db)
            .await
    }

    /// 根据用户名查询用户信息（不区分大小写）
    pub async fn find_by_username(db: &DbConn, username: &Option<String>) -> Result<Option<admin::Model>, DbErr> {
        Admin::find()
            .filter(admin::Column::UserName.eq(username.clone().unwrap_or_default().to_lowercase()))
            .one(db)
            .await
    }

    /// 根据手机号查询用户信息
    pub async fn find_by_mobile(db: &DbConn, mobile: &Option<String>) -> Result<Option<admin::Model>, DbErr> {
        Admin::find()
            .filter(admin::Column::Mobile.eq(mobile.clone().unwrap_or_default()))
            .one(db)
            .await
    }

    /// 根据邮箱查询用户信息
    pub async fn find_by_email(db: &DbConn, email: &String) -> Result<Option<admin::Model>, DbErr> {
        Admin::find()
            .filter(admin::Column::Email.eq(email))
            .one(db)
            .await
    }


    pub async fn find_by_nick_name(db: &DbConn, nick_name: &String) -> Result<Option<admin::Model>, DbErr> {
        Admin::find()
            .filter(admin::Column::NickName.eq(nick_name))
            .one(db)
            .await
    }

    pub async fn find_by_id_in(db: &DbConn, ids: Vec<i64>) -> Result<Vec<admin::Model>, DbErr> {
        Admin::find()
            .filter(admin::Column::Id.is_in(ids))
            .all(db)
            .await
    }

    pub async fn find_all_options<C: ConnectionTrait>(db: &C) -> Result<Vec<admin::Model>, DbErr> {
        Admin::find()
            .filter(admin::Column::Deleted.eq(0))
            .filter(admin::Column::Status.eq(1))
            .order_by_asc(admin::Column::Sort)
            .all(db)
            .await
    }

    pub async fn select_count(
        db: &DbConn,
        wheres: PageWhere,
    ) -> Result<i64, DbErr> {
        Admin::find()
            .distinct()
            .find_also_related(Dept)
            .apply_if(wheres.user_name, |query, v| {
                query.filter(admin::Column::UserName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.nick_name, |query, v| {
                query.filter(admin::Column::NickName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.email, |query, v| {
                query.filter(admin::Column::Email.eq(v))
            })
            .apply_if(wheres.mobile, |query, v| {
                query.filter(admin::Column::Mobile.eq(v))
            })
            .apply_if(wheres.user_type, |query, v| {
                query.filter(admin::Column::UserType.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(admin::Column::Status.eq(v))
            })
            .apply_if(wheres.dept_id, |query, v| {
                query.filter(dept::Column::Id.eq(v))
            })
            .count(db)
            .await
            .map(|c| c as i64)
            .map(|c| c as i64)
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        wheres: PageWhere,
    ) -> Result<(Vec<admin::Model>, i64), DbErr> {
        let query = Admin::find()
            .distinct()
            .select_only()
            .column(admin::Column::Id)
            .column(admin::Column::UserName)
            .column(admin::Column::NickName)
            .column(admin::Column::UserType)
            .column(admin::Column::Email)
            .column(admin::Column::Mobile)
            .column(admin::Column::Gender)
            .column(admin::Column::Avatar)
            .column(admin::Column::Password)
            .column(admin::Column::Status)
            .column(admin::Column::Deleted)
            .column(admin::Column::LoginIp)
            .column(admin::Column::LoginDate)
            .column(admin::Column::CreateBy)
            .column(admin::Column::CreateTime)
            .column(admin::Column::UpdateBy)
            .column(admin::Column::UpdateTime)
            .column(admin::Column::Remark)
            .column(admin::Column::Sort)
            .column(admin::Column::DirectManagerId)
            .column(admin::Column::HireDate)
            .column(admin::Column::SalaryEnabled)
            .column(admin::Column::ProbationMonths)
            .column(admin::Column::ProbationRatio)
            .join_rev(
                JoinType::LeftJoin,
                admin_dept_merge::Relation::Admin.def(),
                //Alias::new("tt"),
            )
            .join_rev(
                JoinType::LeftJoin,
                dept::Relation::AdminDeptMerge.def(),
                //Alias::new("tt1"),
            )
            .apply_if(wheres.user_name, |query, v| {
                query.filter(admin::Column::UserName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.nick_name, |query, v| {
                query.filter(admin::Column::NickName.contains(format!("%{}%", v).as_str()))
            })
            .apply_if(wheres.email, |query, v| {
                query.filter(admin::Column::Email.eq(v))
            })
            .apply_if(wheres.mobile, |query, v| {
                query.filter(admin::Column::Mobile.eq(v))
            })
            .apply_if(wheres.user_type, |query, v| {
                query.filter(admin::Column::UserType.eq(v))
            })
            .apply_if(wheres.status, |query, v| {
                query.filter(admin::Column::Status.eq(v))
            })
            .apply_if(wheres.dept_id, |query, v| {
                query.filter(dept::Column::Id.eq(v))
            })
            .filter(admin::Column::Deleted.is_null().or(admin::Column::Deleted.ne(Some(1))))
            .order_by_asc(admin::Column::Id);
            //.paginate(db, per_page as u64);

            let sql = query.build(DbBackend::Postgres);
            log::info!("Generated SQL: {}", sql.to_string());
            
            let paginator = query.paginate(db, per_page as u64);
            let num_pages = paginator.num_pages().await? as i64;
            //let sql = query.build(DbBackend::Postgres);
            //log::info!("===================sql={}", sql.to_string());
        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}