use sea_orm::*;
use sea_orm::prelude::{DateTime, Decimal};
use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::purchase::entity::{supplier, supplier::Entity as Supplier};
use crate::utils::string_utils::{deserialize_string_to_u64, serialize_option_u64_to_string};

/// 供应商新增请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupplierSaveRequest {
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub level: Option<i32>,
    pub currency: Option<i32>,
    pub credit_limit: Option<Decimal>,
    pub credit_days: Option<i32>,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub tax_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<i32>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub notes: Option<String>,
    pub is_active: Option<bool>,
}

impl From<SupplierSaveRequest> for supplier::ActiveModel {
    fn from(req: SupplierSaveRequest) -> Self {
        let now = chrono::Local::now().naive_local().to_owned();
        supplier::ActiveModel {
            company_name: Set(req.company_name),
            short_name: Set(req.short_name),
            contact_name: Set(req.contact_name),
            contact_phone: Set(req.contact_phone),
            contact_email: Set(req.contact_email),
            country: Set(req.country),
            region: Set(req.region),
            address: Set(req.address),
            website: Set(req.website),
            industry: Set(req.industry),
            level: Set(req.level),
            currency: Set(req.currency),
            credit_limit: Set(req.credit_limit),
            credit_days: Set(req.credit_days),
            bank_name: Set(req.bank_name),
            bank_account: Set(req.bank_account),
            tax_id: Set(req.tax_id),
            tags: Set(req.tags),
            status: Set(req.status.or(Some(1))),
            payment_terms: Set(req.payment_terms),
            delivery_terms: Set(req.delivery_terms),
            notes: Set(req.notes),
            is_active: Set(req.is_active.or(Some(true))),
            create_time: Set(Some(now)),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        }
    }
}

/// 供应商更新请求DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupplierUpdateRequest {
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub level: Option<i32>,
    pub currency: Option<i32>,
    pub credit_limit: Option<Decimal>,
    pub credit_days: Option<i32>,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub tax_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<i32>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub notes: Option<String>,
    pub is_active: Option<bool>,
}

/// 供应商详情VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupplierDetailVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub supplier_no: Option<String>,
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub address: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub level: Option<i32>,
    pub currency: Option<i32>,
    pub credit_limit: Option<Decimal>,
    pub credit_days: Option<i32>,
    pub bank_name: Option<String>,
    pub bank_account: Option<String>,
    pub tax_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<i32>,
    pub payment_terms: Option<String>,
    pub delivery_terms: Option<String>,
    pub notes: Option<String>,
    pub is_active: Option<bool>,
    pub create_time: Option<DateTime>,
}

impl From<supplier::Model> for SupplierDetailVO {
    fn from(item: supplier::Model) -> Self {
        SupplierDetailVO {
            id: Option::from(item.id),
            supplier_no: item.supplier_no,
            company_name: item.company_name,
            short_name: item.short_name,
            contact_name: item.contact_name,
            contact_phone: item.contact_phone,
            contact_email: item.contact_email,
            country: item.country,
            region: item.region,
            address: item.address,
            website: item.website,
            industry: item.industry,
            level: item.level,
            currency: item.currency,
            credit_limit: item.credit_limit,
            credit_days: item.credit_days,
            bank_name: item.bank_name,
            bank_account: item.bank_account,
            tax_id: item.tax_id,
            tags: item.tags,
            status: item.status,
            payment_terms: item.payment_terms,
            delivery_terms: item.delivery_terms,
            notes: item.notes,
            is_active: item.is_active,
            create_time: item.create_time,
        }
    }
}

/// 供应商列表VO
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupplierListVO {
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    pub supplier_no: Option<String>,
    pub company_name: Option<String>,
    pub short_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub level: Option<i32>,
    pub status: Option<i32>,
    pub create_time: Option<DateTime>,
}

impl From<supplier::Model> for SupplierListVO {
    fn from(item: supplier::Model) -> Self {
        SupplierListVO {
            id: Option::from(item.id),
            supplier_no: item.supplier_no,
            company_name: item.company_name,
            short_name: item.short_name,
            contact_name: item.contact_name,
            contact_phone: item.contact_phone,
            contact_email: item.contact_email,
            country: item.country,
            region: item.region,
            level: item.level,
            status: item.status,
            create_time: item.create_time,
        }
    }
}

/// 供应商列表查询参数
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SupplierListQuery {
    #[serde(rename = "page")]
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
    pub keywords: Option<String>,
    pub country: Option<String>,
    pub level: Option<i32>,
    pub status: Option<i32>,
}

/// 供应商数据模型操作类
pub struct SupplierModel;

impl SupplierModel {
    pub async fn insert(db: &DbConn, req: &SupplierSaveRequest, created_by: i64) -> Result<i64, DbErr> {
        let mut active: supplier::ActiveModel = req.clone().into();
        active.created_by = Set(Some(created_by));
        active.supplier_no = Set(Some(Self::generate_supplier_no(db).await?));
        Supplier::insert(active)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    async fn generate_supplier_no(db: &DbConn) -> Result<String, DbErr> {
        let today = chrono::Local::now().format("%Y%m%d").to_string();
        let prefix = format!("SUP{}", today);
        let count = Supplier::find()
            .filter(supplier::Column::SupplierNo.starts_with(&prefix))
            .count(db)
            .await?;
        Ok(format!("{}{:04}", prefix, count + 1))
    }

    pub async fn batch_delete_by_ids(db: &DbConn, ids: &Vec<i64>) -> Result<i64, DbErr> {
        Supplier::update_many()
            .set(supplier::ActiveModel {
                deleted: Set(Some(1)),
                ..Default::default()
            })
            .filter(supplier::Column::Id.is_in(ids.clone()))
            .exec(db)
            .await
            .map(|r| r.rows_affected as i64)
    }

    pub async fn update_by_id(db: &DbConn, id: i64, req: &SupplierUpdateRequest, updated_by: i64) -> Result<i64, DbErr> {
        let now = chrono::Local::now().naive_local().to_owned();
        let payload = supplier::ActiveModel {
            company_name: Set(req.company_name.clone()),
            short_name: Set(req.short_name.clone()),
            contact_name: Set(req.contact_name.clone()),
            contact_phone: Set(req.contact_phone.clone()),
            contact_email: Set(req.contact_email.clone()),
            country: Set(req.country.clone()),
            region: Set(req.region.clone()),
            address: Set(req.address.clone()),
            website: Set(req.website.clone()),
            industry: Set(req.industry.clone()),
            level: Set(req.level),
            currency: Set(req.currency),
            credit_limit: Set(req.credit_limit),
            credit_days: Set(req.credit_days),
            bank_name: Set(req.bank_name.clone()),
            bank_account: Set(req.bank_account.clone()),
            tax_id: Set(req.tax_id.clone()),
            tags: Set(req.tags.clone()),
            status: Set(req.status),
            payment_terms: Set(req.payment_terms.clone()),
            delivery_terms: Set(req.delivery_terms.clone()),
            notes: Set(req.notes.clone()),
            is_active: Set(req.is_active),
            updated_by: Set(Some(updated_by)),
            update_time: Set(Some(now)),
            ..Default::default()
        };

        let update_result = Supplier::update_many()
            .set(payload)
            .filter(supplier::Column::Id.eq(id))
            .filter(supplier::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        Ok(update_result.rows_affected as i64)
    }

    pub async fn find_by_id(db: &DbConn, id: i64) -> Result<Option<supplier::Model>, DbErr> {
        Supplier::find_by_id(id)
            .filter(supplier::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        per_page: i64,
        keywords: Option<String>,
        level: Option<i32>,
        country: Option<String>,
        status: Option<i32>,
    ) -> Result<(Vec<supplier::Model>, i64), DbErr> {
        let mut query = Supplier::find()
            .filter(supplier::Column::Deleted.eq(0));

        if let Some(k) = keywords {
            if !k.is_empty() {
                query = query.filter(
                    Condition::any()
                        .add(supplier::Column::CompanyName.contains(k.clone()))
                        .add(supplier::Column::ShortName.contains(k.clone()))
                        .add(supplier::Column::ContactName.contains(k.clone()))
                        .add(supplier::Column::ContactPhone.contains(k))
                );
            }
        }
        if let Some(l) = level {
            query = query.filter(supplier::Column::Level.eq(l));
        }
        if let Some(c) = country {
            if !c.is_empty() {
                query = query.filter(supplier::Column::Country.eq(c));
            }
        }
        if let Some(s) = status {
            query = query.filter(supplier::Column::Status.eq(s));
        }

        let paginator = query.order_by_desc(supplier::Column::CreateTime).paginate(db, per_page as u64);
        let total = paginator.num_items().await? as i64;

        paginator.fetch_page((page - 1) as u64).await.map(|p| (p, total))
    }
}
