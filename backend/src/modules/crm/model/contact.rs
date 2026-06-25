use sea_orm::*;
use sea_orm::prelude::{DateTime, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::crm::entity::{contact, contact::Entity as Contact, customer_contact_merge, customer_contact_merge::Entity as CustomerContactMerge, customer};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

// ==================== 联系人请求 DTO ====================

/// 联系人新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContactSaveRequest {
    /// 联系人姓名
    pub name: Option<String>,
    /// 职位/头衔
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// WhatsApp号码
    pub whatsapp: Option<String>,
    /// 微信号
    pub wechat: Option<String>,
    /// 性别（0-男，1-女，2-未知/未指定）
    pub gender: Option<i32>,
    /// 生日日期
    pub birthday: Option<Date>,
    /// 备注信息
    pub notes: Option<String>,
    /// 绑定客户ID（可选，创建时同时绑定）
    pub customer_id: Option<i64>,
    /// 在客户公司的职位（0-决策人 1-影响者 2-使用者 3-其他）
    pub role_type: Option<i32>,
    /// 是否首要联系人
    pub is_primary: Option<bool>,
    /// 是否账单联系人
    pub is_billing: Option<bool>,
    /// 是否收货联系人
    pub is_shipping: Option<bool>,
    /// 绑定时间
    pub bound_at: Option<DateTime>,
}

impl From<ContactSaveRequest> for ContactSaveDTO {
    fn from(item: ContactSaveRequest) -> Self {
        ContactSaveDTO {
            id: None,
            name: item.name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            whatsapp: item.whatsapp,
            wechat: item.wechat,
            gender: item.gender,
            birthday: item.birthday,
            notes: item.notes,
            customer_id: item.customer_id,
            role_type: item.role_type,
            is_primary: item.is_primary,
            is_billing: item.is_billing,
            is_shipping: item.is_shipping,
            bound_at: item.bound_at,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 联系人更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContactUpdateRequest {
    /// 联系人ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 联系人姓名
    pub name: Option<String>,
    /// 职位/头衔
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// WhatsApp号码
    pub whatsapp: Option<String>,
    /// 微信号
    pub wechat: Option<String>,
    /// 性别（0-男，1-女，2-未知/未指定）
    pub gender: Option<i32>,
    /// 生日日期
    pub birthday: Option<Date>,
    /// 备注信息
    pub notes: Option<String>,
    /// 绑定客户ID
    pub customer_id: Option<i64>,
    /// 在客户公司的角色类型（0-决策人 1-影响者 2-使用者 3-其他）
    pub role_type: Option<i32>,
}

impl From<ContactUpdateRequest> for ContactSaveDTO {
    fn from(item: ContactUpdateRequest) -> Self {
        ContactSaveDTO {
            id: item.id,
            name: item.name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            whatsapp: item.whatsapp,
            wechat: item.wechat,
            gender: item.gender,
            birthday: item.birthday,
            notes: item.notes,
            customer_id: item.customer_id,
            role_type: item.role_type,
            is_primary: None,
            is_billing: None,
            is_shipping: None,
            bound_at: None,
            deleted: None,
            created_by: None,
            create_time: None,
            updated_by: None,
            update_time: None,
        }
    }
}

/// 联系人保存DTO（包含新增和更新的所有字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContactSaveDTO {
    /// 联系人ID
    pub id: Option<i64>,
    /// 联系人姓名
    pub name: Option<String>,
    /// 职位/头衔
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// WhatsApp号码
    pub whatsapp: Option<String>,
    /// 微信号
    pub wechat: Option<String>,
    /// 性别（0-男，1-女，2-未知/未指定）
    pub gender: Option<i32>,
    /// 生日日期
    pub birthday: Option<Date>,
    /// 备注信息
    pub notes: Option<String>,
    /// 绑定客户ID（创建时可选绑定）
    pub customer_id: Option<i64>,
    /// 角色类型（0-决策人 1-影响者 2-使用者 3-其他）
    pub role_type: Option<i32>,
    /// 是否首要联系人
    pub is_primary: Option<bool>,
    /// 是否账单联系人
    pub is_billing: Option<bool>,
    /// 是否收货联系人
    pub is_shipping: Option<bool>,
    /// 绑定时间
    pub bound_at: Option<DateTime>,
    /// 软删除标记
    pub deleted: Option<i32>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新人ID
    pub updated_by: Option<i64>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

// ==================== 联系人关联操作请求 ====================

/// 绑定联系人到客户请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContactBindRequest {
    /// 联系人ID
    pub contact_id: i64,
    /// 客户ID
    pub customer_id: i64,
    /// 在该公司的职位
    pub title: Option<String>,
    /// 角色类型（0-决策人 1-影响者 2-使用者 3-其他）
    pub role_type: Option<i32>,
    /// 是否首要联系人
    pub is_primary: Option<bool>,
    /// 是否账单联系人
    pub is_billing: Option<bool>,
    /// 是否收货联系人
    pub is_shipping: Option<bool>,
    /// 绑定时间
    pub bound_at: Option<DateTime>,
}

/// 解绑联系人请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContactUnbindRequest {
    /// 联系人ID
    pub contact_id: i64,
    /// 解绑时间
    pub unbound_at: Option<DateTime>,
    /// 备注（离职原因等）
    pub notes: Option<String>,
}

/// 设置联系人角色请求
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContactSetRoleRequest {
    /// 联系人ID
    pub contact_id: i64,
    /// 是否首要联系人
    pub is_primary: Option<bool>,
    /// 是否账单联系人
    pub is_billing: Option<bool>,
    /// 是否收货联系人
    pub is_shipping: Option<bool>,
    /// 角色类型（0-决策人 1-影响者 2-使用者 3-其他）
    pub role_type: Option<i32>,
}

// ==================== 联系人响应 VO ====================

/// 联系人详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContactDetailVO {
    /// 联系人ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 联系人姓名
    pub name: Option<String>,
    /// 职位/头衔
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// WhatsApp号码
    pub whatsapp: Option<String>,
    /// 微信号
    pub wechat: Option<String>,
    /// 性别（0-男，1-女，2-未知/未指定）
    pub gender: Option<i32>,
    /// 生日日期
    pub birthday: Option<Date>,
    /// 备注信息
    pub notes: Option<String>,
    /// 当前任职公司
    pub current_company: Option<ContactCompanyInfo>,
    /// 职业生涯履历
    pub career_history: Option<Vec<CareerHistoryItem>>,
    /// 创建时间
    pub create_time: Option<DateTime>,
    /// 更新时间
    pub update_time: Option<DateTime>,
}

/// 当前公司信息
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContactCompanyInfo {
    pub customer_id: Option<i64>,
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub title: Option<String>,
    pub role_type: Option<i32>,
    pub is_primary: Option<bool>,
    pub bound_at: Option<DateTime>,
}

/// 职业生涯履历项
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CareerHistoryItem {
    pub id: Option<i64>,
    pub customer_id: Option<i64>,
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub title: Option<String>,
    pub role_type: Option<i32>,
    pub is_current: Option<bool>,
    pub is_primary: Option<bool>,
    pub bound_at: Option<DateTime>,
    pub unbound_at: Option<DateTime>,
    pub notes: Option<String>,
}

/// 联系人列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContactListVO {
    /// 联系人ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 联系人姓名
    pub name: Option<String>,
    /// 职位/头衔
    pub title: Option<String>,
    /// 邮箱地址
    pub email: Option<String>,
    /// 固定电话
    pub phone: Option<String>,
    /// 手机号码
    pub mobile: Option<String>,
    /// 关联客户ID
    pub customer_id: Option<i64>,
    /// 关联客户名称
    pub company_name: Option<String>,
    /// 当前任职角色（0-决策人 1-影响者 2-使用者 3-其他）
    pub role_type: Option<i32>,
    /// 创建时间
    pub create_time: Option<DateTime>,
}

/// 客户联系人VO（客户详情页中使用）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CustomerContactVO {
    /// 联系人ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 联系人姓名
    pub name: Option<String>,
    /// 职位
    pub title: Option<String>,
    /// 角色类型（0-决策人 1-影响者 2-使用者 3-其他）
    pub role_type: Option<i32>,
    /// 邮箱
    pub email: Option<String>,
    /// 手机号
    pub mobile: Option<String>,
    /// 是否首要联系人
    pub is_primary: Option<bool>,
    /// 是否账单联系人
    pub is_billing: Option<bool>,
    /// 是否收货联系人
    pub is_shipping: Option<bool>,
    /// 是否当前在职
    pub is_current: Option<bool>,
    /// 绑定时间
    pub bound_at: Option<DateTime>,
    /// 解绑时间
    pub unbound_at: Option<DateTime>,
    /// 备注
    pub notes: Option<String>,
}

/// 联系人列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContactListQuery {
    /// 页码
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    /// 每页大小
    pub page_size: Option<i64>,
    /// 关键词（搜索联系人姓名、邮箱等）
    pub keywords: Option<String>,
    /// 客户ID（筛选关联特定客户的联系人）
    pub customer_id: Option<i64>,
}

// ==================== ContactModel ====================

/// 联系人数据模型操作类
pub struct ContactModel;

impl ContactModel {
    /// 新增联系人
    pub async fn insert(db: &impl ConnectionTrait, req: &ContactSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = contact::ActiveModel {
            name: Set(req.name.clone()),
            title: Set(req.title.clone()),
            email: Set(req.email.clone()),
            phone: Set(req.phone.clone()),
            mobile: Set(req.mobile.clone()),
            whatsapp: Set(req.whatsapp.clone()),
            wechat: Set(req.wechat.clone()),
            gender: Set(req.gender.clone()),
            birthday: Set(req.birthday.clone()),
            notes: Set(req.notes.clone()),
            create_time: Set(Option::from(now)),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        Contact::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 插入关联记录
    pub async fn insert_merge(db: &impl ConnectionTrait, customer_id: i64, contact_id: i64, req: &ContactSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = customer_contact_merge::ActiveModel {
            customer_id: Set(customer_id),
            contact_id: Set(contact_id),
            title: Set(req.title.clone()),
            role_type: Set(req.role_type.clone()),
            is_current: Set(Some(true)),
            is_primary: Set(req.is_primary.clone()),
            is_billing: Set(req.is_billing.clone()),
            is_shipping: Set(req.is_shipping.clone()),
            bound_at: Set(req.bound_at.clone().or(Some(now))),
            create_time: Set(Option::from(now)),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        CustomerContactMerge::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除联系人（软删除）
    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Contact::update_many()
            .set(contact::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(contact::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    /// 更新联系人基本信息
    pub async fn update_by_id(db: &impl ConnectionTrait, id: &Option<i64>, req: &ContactSaveDTO) -> Result<i64, DbErr> {
        let payload = contact::ActiveModel {
            name: Set(req.name.clone()),
            title: Set(req.title.clone()),
            email: Set(req.email.clone()),
            phone: Set(req.phone.clone()),
            mobile: Set(req.mobile.clone()),
            whatsapp: Set(req.whatsapp.clone()),
            wechat: Set(req.wechat.clone()),
            gender: Set(req.gender.clone()),
            birthday: Set(req.birthday.clone()),
            notes: Set(req.notes.clone()),
            update_time: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
            ..Default::default()
        };

        let update_result: UpdateResult = Contact::update_many()
            .set(payload)
            .filter(contact::Column::Id.eq(id.clone().unwrap_or_default()))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 根据ID查询联系人详情
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<contact::Model>, DbErr> {
        Contact::find_by_id(id)
            .filter(contact::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 查询联系人的当前任职关联
    pub async fn find_current_merge(db: &DbConn, contact_id: i64) -> Result<Option<customer_contact_merge::Model>, DbErr> {
        CustomerContactMerge::find()
            .filter(customer_contact_merge::Column::ContactId.eq(contact_id))
            .filter(customer_contact_merge::Column::IsCurrent.eq(true))
            .one(db)
            .await
    }

    /// 查询联系人的职业生涯履历
    pub async fn find_career_history(db: &DbConn, contact_id: i64) -> Result<Vec<CareerHistoryItem>, DbErr> {
        let results = CustomerContactMerge::find()
            .filter(customer_contact_merge::Column::ContactId.eq(contact_id))
            .order_by_desc(customer_contact_merge::Column::BoundAt)
            .all(db)
            .await?;

        let mut items = Vec::new();
        for m in results {
            let cu = customer::Entity::find_by_id(m.customer_id)
                .one(db)
                .await?;
            items.push(CareerHistoryItem {
                id: Some(m.id),
                customer_id: Some(m.customer_id),
                company_name: cu.as_ref().and_then(|c| c.company_name.clone()),
                short_name: cu.as_ref().and_then(|c| c.short_name.clone()),
                title: m.title,
                role_type: m.role_type,
                is_current: m.is_current,
                is_primary: m.is_primary,
                bound_at: m.bound_at,
                unbound_at: m.unbound_at,
                notes: m.notes,
            });
        }
        Ok(items)
    }

    /// 绑定联系人到客户（入职 / 切换公司 / 更新信息）
    /// - 如果当前已有 is_current=true 的关联：
    ///   - 同一公司：仅更新 role/title/flags，不新增履历
    ///   - 不同公司：把当前关联降级为历史，再插入新关联
    /// - 如果当前无关联：直接插入新关联
    pub async fn bind_contact(db: &DbConn, req: &ContactBindRequest) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();

        // 查询当前任职
        let current = CustomerContactMerge::find()
            .filter(customer_contact_merge::Column::ContactId.eq(req.contact_id))
            .filter(customer_contact_merge::Column::IsCurrent.eq(true))
            .one(db)
            .await?;

        if let Some(current_record) = current {
            if current_record.customer_id == req.customer_id {
                // 同一公司：只更新信息，不新增履历
                let update_payload = customer_contact_merge::ActiveModel {
                    id: Set(current_record.id),
                    title: Set(req.title.clone()),
                    role_type: Set(req.role_type.clone()),
                    is_primary: Set(req.is_primary.clone()),
                    is_billing: Set(req.is_billing.clone()),
                    is_shipping: Set(req.is_shipping.clone()),
                    update_time: Set(Option::from(now)),
                    ..Default::default()
                };
                CustomerContactMerge::update(update_payload).exec(db).await?;
                return Ok(1);
            }

            // 不同公司：当前关联降级为历史
            let downgrade = customer_contact_merge::ActiveModel {
                id: Set(current_record.id),
                is_current: Set(Some(false)),
                unbound_at: Set(Option::from(now)),
                update_time: Set(Option::from(now)),
                ..Default::default()
            };
            CustomerContactMerge::update(downgrade).exec(db).await?;
        }

        // 插入新关联（切换公司 / 首次绑定）
        let payload = customer_contact_merge::ActiveModel {
            customer_id: Set(req.customer_id),
            contact_id: Set(req.contact_id),
            title: Set(req.title.clone()),
            role_type: Set(req.role_type.clone()),
            is_current: Set(Some(true)),
            is_primary: Set(req.is_primary.clone()),
            is_billing: Set(req.is_billing.clone()),
            is_shipping: Set(req.is_shipping.clone()),
            bound_at: Set(req.bound_at.clone().or(Some(now))),
            create_time: Set(Option::from(now)),
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        CustomerContactMerge::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 解绑联系人（离职）
    pub async fn unbind_contact(db: &DbConn, req: &ContactUnbindRequest) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let update_result = CustomerContactMerge::update_many()
            .set(customer_contact_merge::ActiveModel {
                is_current: Set(Some(false)),
                unbound_at: Set(req.unbound_at.clone().or(Some(now))),
                notes: Set(req.notes.clone()),
                update_time: Set(Option::from(now)),
                ..Default::default()
            })
            .filter(customer_contact_merge::Column::ContactId.eq(req.contact_id))
            .filter(customer_contact_merge::Column::IsCurrent.eq(true))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 设置联系人角色/标记
    pub async fn set_role(db: &DbConn, req: &ContactSetRoleRequest) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let mut set_values = customer_contact_merge::ActiveModel {
            update_time: Set(Option::from(now)),
            ..Default::default()
        };

        if req.is_primary.is_some() {
            set_values.is_primary = Set(req.is_primary.clone());
        }
        if req.is_billing.is_some() {
            set_values.is_billing = Set(req.is_billing.clone());
        }
        if req.is_shipping.is_some() {
            set_values.is_shipping = Set(req.is_shipping.clone());
        }
        if req.role_type.is_some() {
            set_values.role_type = Set(req.role_type.clone());
        }

        let update_result = CustomerContactMerge::update_many()
            .set(set_values)
            .filter(customer_contact_merge::Column::ContactId.eq(req.contact_id))
            .filter(customer_contact_merge::Column::IsCurrent.eq(true))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    /// 分页查询联系人列表
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        customer_id: Option<i64>,
    ) -> Result<(Vec<contact::Model>, i64), DbErr> {
        let mut query = Contact::find()
            .filter(contact::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            query = query.filter(
                Condition::any()
                    .add(contact::Column::Name.contains(k.clone()))
                    .add(contact::Column::Email.contains(k)),
            );
        }
        if let Some(c) = customer_id {
            let contact_ids: Vec<i64> = CustomerContactMerge::find()
                .filter(customer_contact_merge::Column::CustomerId.eq(c))
                .filter(customer_contact_merge::Column::IsCurrent.eq(true))
                .all(db)
                .await?
                .iter()
                .map(|m| m.contact_id)
                .collect();
            if !contact_ids.is_empty() {
                query = query.filter(contact::Column::Id.is_in(contact_ids));
            } else {
                return Ok((Vec::new(), 0));
            }
        }

        let paginator = query.order_by_desc(contact::Column::CreateTime).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }

    /// 查询客户下的联系人（当前在职+历史）
    pub async fn find_by_customer(db: &DbConn, customer_id: i64) -> Result<(Vec<CustomerContactVO>, Vec<CustomerContactVO>), DbErr> {
        let merges = CustomerContactMerge::find()
            .filter(customer_contact_merge::Column::CustomerId.eq(customer_id))
            .order_by_desc(customer_contact_merge::Column::IsCurrent)
            .order_by_desc(customer_contact_merge::Column::IsPrimary)
            .order_by_asc(customer_contact_merge::Column::BoundAt)
            .all(db)
            .await?;

        let mut current = Vec::new();
        let mut history = Vec::new();

        for m in merges {
            let c = Contact::find_by_id(m.contact_id)
                .filter(contact::Column::Deleted.eq(0))
                .one(db)
                .await?;

            if let Some(c) = c {
                let vo = CustomerContactVO {
                    id: Some(c.id),
                    name: c.name,
                    title: m.title.clone(),
                    role_type: m.role_type,
                    email: c.email,
                    mobile: c.mobile,
                    is_primary: m.is_primary,
                    is_billing: m.is_billing,
                    is_shipping: m.is_shipping,
                    is_current: m.is_current,
                    bound_at: m.bound_at,
                    unbound_at: m.unbound_at,
                    notes: m.notes,
                };
                if m.is_current.unwrap_or(false) {
                    current.push(vo);
                } else {
                    history.push(vo);
                }
            }
        }

        Ok((current, history))
    }
}