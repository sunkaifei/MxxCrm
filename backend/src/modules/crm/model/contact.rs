use sea_orm::*;
use sea_orm::prelude::{DateTime, Date};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::crm::entity::{contact, contact::Entity as Contact};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 联系人新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContactSaveRequest {
    /// 客户ID
    pub customer_id: Option<i64>,
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
    /// 是否为主联系人
    pub is_primary: Option<bool>,
    /// 是否为账单联系人
    pub is_billing: Option<bool>,
    /// 是否为收货联系人
    pub is_shipping: Option<bool>,
    /// 生日日期
    pub birthday: Option<Date>,
    /// 备注信息
    pub notes: Option<String>,
}

impl From<ContactSaveRequest> for ContactSaveDTO {
    fn from(item: ContactSaveRequest) -> Self {
        ContactSaveDTO {
            id: None,
            customer_id: item.customer_id,
            name: item.name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            whatsapp: item.whatsapp,
            wechat: item.wechat,
            is_primary: item.is_primary,
            is_billing: item.is_billing,
            is_shipping: item.is_shipping,
            birthday: item.birthday,
            notes: item.notes,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
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
    /// 客户ID
    pub customer_id: Option<i64>,
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
    /// 是否为主联系人
    pub is_primary: Option<bool>,
    /// 是否为账单联系人
    pub is_billing: Option<bool>,
    /// 是否为收货联系人
    pub is_shipping: Option<bool>,
    /// 生日日期
    pub birthday: Option<Date>,
    /// 备注信息
    pub notes: Option<String>,
}

impl From<ContactUpdateRequest> for ContactSaveDTO {
    fn from(item: ContactUpdateRequest) -> Self {
        ContactSaveDTO {
            id: item.id,
            customer_id: item.customer_id,
            name: item.name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            whatsapp: item.whatsapp,
            wechat: item.wechat,
            is_primary: item.is_primary,
            is_billing: item.is_billing,
            is_shipping: item.is_shipping,
            birthday: item.birthday,
            notes: item.notes,
            deleted: None,
            created_by: None,
            created_at: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

/// 联系人保存DTO（包含新增和更新的所有字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ContactSaveDTO {
    /// 联系人ID
    pub id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
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
    /// 是否为主联系人
    pub is_primary: Option<bool>,
    /// 是否为账单联系人
    pub is_billing: Option<bool>,
    /// 是否为收货联系人
    pub is_shipping: Option<bool>,
    /// 生日日期
    pub birthday: Option<Date>,
    /// 备注信息
    pub notes: Option<String>,
    /// 软删除标记
    pub deleted: Option<i32>,
    /// 创建人ID
    pub created_by: Option<i64>,
    /// 创建时间
    pub created_at: Option<DateTime>,
    /// 更新人ID
    pub updated_by: Option<i64>,
    /// 更新时间
    pub updated_at: Option<DateTime>,
}

/// 联系人详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContactDetailVO {
    /// 联系人ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
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
    /// 是否为主联系人
    pub is_primary: Option<bool>,
    /// 是否为账单联系人
    pub is_billing: Option<bool>,
    /// 是否为收货联系人
    pub is_shipping: Option<bool>,
    /// 生日日期
    pub birthday: Option<Date>,
    /// 备注信息
    pub notes: Option<String>,
}

impl From<contact::Model> for ContactDetailVO {
    fn from(item: contact::Model) -> Self {
        ContactDetailVO {
            id: Option::from(item.id),
            customer_id: item.customer_id,
            name: item.name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            whatsapp: item.whatsapp,
            wechat: item.wechat,
            is_primary: item.is_primary,
            is_billing: item.is_billing,
            is_shipping: item.is_shipping,
            birthday: item.birthday,
            notes: item.notes,
        }
    }
}

/// 联系人列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ContactListVO {
    /// 联系人ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 客户ID
    pub customer_id: Option<i64>,
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
    /// 是否为主联系人
    pub is_primary: Option<bool>,
}

impl From<contact::Model> for ContactListVO {
    fn from(item: contact::Model) -> Self {
        ContactListVO {
            id: Option::from(item.id),
            customer_id: item.customer_id,
            name: item.name,
            title: item.title,
            email: item.email,
            phone: item.phone,
            mobile: item.mobile,
            is_primary: item.is_primary,
        }
    }
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
    /// 客户ID
    pub customer_id: Option<i64>,
}

/// 联系人数据模型操作类
pub struct ContactModel;

impl ContactModel {
    /// 新增联系人
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `req` - 联系人保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 新增记录的ID
    pub async fn insert(db: &DbConn, req: &ContactSaveDTO) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = contact::ActiveModel {
            customer_id: Set(req.customer_id.clone()),
            name: Set(req.name.clone()),
            title: Set(req.title.clone()),
            email: Set(req.email.clone()),
            phone: Set(req.phone.clone()),
            mobile: Set(req.mobile.clone()),
            whatsapp: Set(req.whatsapp.clone()),
            wechat: Set(req.wechat.clone()),
            is_primary: Set(req.is_primary.clone()),
            is_billing: Set(req.is_billing.clone()),
            is_shipping: Set(req.is_shipping.clone()),
            birthday: Set(req.birthday.clone()),
            notes: Set(req.notes.clone()),
            created_at: Set(Option::from(now)),
            updated_at: Set(Option::from(now)),
            ..Default::default()
        };

        Contact::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 批量删除联系人（软删除）
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `ids` - 要删除的联系人ID列表
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 删除的记录数
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

    /// 更新联系人信息
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 联系人ID
    /// * `req` - 联系人保存DTO
    ///
    /// # 返回
    /// * `Result<i64, DbErr>` - 更新的记录数
    pub async fn update_by_id(db: &DbConn, id: &Option<i64>, req: &ContactSaveDTO) -> Result<i64, DbErr> {
        let payload = contact::ActiveModel {
            customer_id: Set(req.customer_id.clone()),
            name: Set(req.name.clone()),
            title: Set(req.title.clone()),
            email: Set(req.email.clone()),
            phone: Set(req.phone.clone()),
            mobile: Set(req.mobile.clone()),
            whatsapp: Set(req.whatsapp.clone()),
            wechat: Set(req.wechat.clone()),
            is_primary: Set(req.is_primary.clone()),
            is_billing: Set(req.is_billing.clone()),
            is_shipping: Set(req.is_shipping.clone()),
            birthday: Set(req.birthday.clone()),
            notes: Set(req.notes.clone()),
            updated_at: Set(Option::from(chrono::Local::now().naive_local().to_owned())),
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
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `id` - 联系人ID
    ///
    /// # 返回
    /// * `Result<Option<contact::Model>, DbErr>` - 联系人模型（未删除）
    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<contact::Model>, DbErr> {
        Contact::find_by_id(id)
            .filter(contact::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 分页查询联系人列表
    ///
    /// # 参数
    /// * `db` - 数据库连接
    /// * `page` - 页码
    /// * `per_page` - 每页大小
    /// * `keywords` - 关键词
    /// * `customer_id` - 客户ID
    ///
    /// # 返回
    /// * `Result<(Vec<contact::Model>, i64), DbErr>` - (联系人列表, 总页数)
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
            query = query.filter(contact::Column::CustomerId.eq(c));
        }

        let paginator = query.order_by_desc(contact::Column::CreatedAt).paginate(db, per_page as u64);
        let num_pages = paginator.num_pages().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, num_pages))
    }
}