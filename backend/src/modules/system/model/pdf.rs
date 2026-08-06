//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::kit::global::{Deserialize, Serialize};
use crate::modules::system::entity::{
    pdf_record, pdf_record::Entity as PdfRecord, pdf_template, pdf_template::Entity as PdfTemplate,
};
use crate::utils::string_utils::{
    deserialize_string_to_i32, deserialize_string_to_u64, serialize_option_u64_to_string,
};
use chrono::Local;
use sea_orm::*;

// ============================ PDF 模板 ============================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PdfTemplateSaveRequest {
    /// 模板名称
    pub name: Option<String>,
    /// 模板编码
    pub template_code: Option<String>,
    /// 单据类型：quotation/order/contract
    pub doc_type: Option<String>,
    /// 模板内容（typst 语法）
    pub content: Option<String>,
    /// 页眉 typst 片段
    pub header_content: Option<String>,
    /// 页脚 typst 片段
    pub footer_content: Option<String>,
    /// 纸张大小：a4/a3/letter（默认 a4）
    pub paper_size: Option<String>,
    /// 方向：portrait/landscape（默认 portrait）
    pub orientation: Option<String>,
    /// 上边距(pt)
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub margin_top: Option<i32>,
    /// 下边距(pt)
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub margin_bottom: Option<i32>,
    /// 左边距(pt)
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub margin_left: Option<i32>,
    /// 右边距(pt)
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub margin_right: Option<i32>,
    /// 主字体
    pub font_family: Option<String>,
    /// 是否默认模板（0否 1是）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub is_default: Option<i32>,
    /// 状态（1启用 0禁用）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub status: Option<i32>,
    /// 排序
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PdfTemplateUpdateRequest {
    /// 主键ID
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub id: Option<i64>,
    /// 模板名称
    pub name: Option<String>,
    /// 模板编码
    pub template_code: Option<String>,
    /// 单据类型：quotation/order/contract
    pub doc_type: Option<String>,
    /// 模板内容（typst 语法）
    pub content: Option<String>,
    /// 页眉 typst 片段
    pub header_content: Option<String>,
    /// 页脚 typst 片段
    pub footer_content: Option<String>,
    /// 纸张大小：a4/a3/letter（默认 a4）
    pub paper_size: Option<String>,
    /// 方向：portrait/landscape（默认 portrait）
    pub orientation: Option<String>,
    /// 上边距(pt)
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub margin_top: Option<i32>,
    /// 下边距(pt)
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub margin_bottom: Option<i32>,
    /// 左边距(pt)
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub margin_left: Option<i32>,
    /// 右边距(pt)
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub margin_right: Option<i32>,
    /// 主字体
    pub font_family: Option<String>,
    /// 是否默认模板（0否 1是）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub is_default: Option<i32>,
    /// 状态（1启用 0禁用）
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub status: Option<i32>,
    /// 排序
    #[serde(deserialize_with = "deserialize_string_to_i32")]
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfTemplateListQuery {
    #[serde(rename = "page")]
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    /// 模板名称
    pub name: Option<String>,
    /// 单据类型
    pub doc_type: Option<String>,
    /// 状态（1启用 0禁用）
    #[serde(deserialize_with = "deserialize_string_to_i32", default)]
    pub status: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PdfTemplateVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 模板名称
    pub name: Option<String>,
    /// 模板编码
    pub template_code: Option<String>,
    /// 单据类型
    pub doc_type: Option<String>,
    /// 模板内容（typst 语法）
    pub content: Option<String>,
    /// 页眉 typst 片段
    pub header_content: Option<String>,
    /// 页脚 typst 片段
    pub footer_content: Option<String>,
    /// 纸张大小
    pub paper_size: Option<String>,
    /// 方向
    pub orientation: Option<String>,
    /// 上边距(pt)
    pub margin_top: Option<i32>,
    /// 下边距(pt)
    pub margin_bottom: Option<i32>,
    /// 左边距(pt)
    pub margin_left: Option<i32>,
    /// 右边距(pt)
    pub margin_right: Option<i32>,
    /// 主字体
    pub font_family: Option<String>,
    /// 是否默认模板（0否 1是）
    pub is_default: Option<i32>,
    /// 状态（1启用 0禁用）
    pub status: Option<i32>,
    /// 排序
    pub sort: Option<i32>,
    /// 备注
    pub remark: Option<String>,
    /// 创建人ID
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<String>,
    /// 更新人ID
    pub update_by: Option<i64>,
    /// 更新时间
    pub update_time: Option<String>,
}

impl From<pdf_template::Model> for PdfTemplateVO {
    fn from(m: pdf_template::Model) -> Self {
        Self {
            id: Option::from(m.id),
            name: m.name,
            template_code: m.template_code,
            doc_type: m.doc_type,
            content: m.content,
            header_content: m.header_content,
            footer_content: m.footer_content,
            paper_size: m.paper_size,
            orientation: m.orientation,
            margin_top: m.margin_top,
            margin_bottom: m.margin_bottom,
            margin_left: m.margin_left,
            margin_right: m.margin_right,
            font_family: m.font_family,
            is_default: m.is_default,
            status: m.status,
            sort: m.sort,
            remark: m.remark,
            create_by: m.create_by,
            create_time: m.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
            update_by: m.update_by,
            update_time: m.update_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PdfTemplateOptionVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 模板名称
    pub name: Option<String>,
    /// 模板编码
    pub template_code: Option<String>,
    /// 是否默认模板（0否 1是）
    pub is_default: Option<i32>,
}

// ============================ PDF 生成请求 ============================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PdfGenerateRequest {
    /// 单据类型：quotation/order/contract
    pub doc_type: Option<String>,
    /// 单据ID
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub doc_id: Option<i64>,
    /// 模板ID（不传则使用该单据类型的默认模板）
    #[serde(default, deserialize_with = "deserialize_string_to_u64")]
    pub template_id: Option<i64>,
}

// ============================ PDF 记录 ============================

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfRecordListQuery {
    #[serde(rename = "page")]
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    /// 单据类型
    pub doc_type: Option<String>,
    /// 单据ID
    #[serde(deserialize_with = "deserialize_string_to_u64", default)]
    pub doc_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PdfRecordVO {
    /// 主键ID
    #[serde(serialize_with = "serialize_option_u64_to_string")]
    pub id: Option<i64>,
    /// 单据类型
    pub doc_type: Option<String>,
    /// 单据ID
    pub doc_id: Option<i64>,
    /// 单据编号
    pub doc_no: Option<String>,
    /// 使用的模板ID
    pub template_id: Option<i64>,
    /// 模板名称（冗余）
    pub template_name: Option<String>,
    /// 生成的文件名
    pub file_name: Option<String>,
    /// 服务器存储路径
    pub file_path: Option<String>,
    /// 访问URL
    pub file_url: Option<String>,
    /// 文件大小(字节)
    pub file_size: Option<i64>,
    /// 页数
    pub page_count: Option<i32>,
    /// 触发方式：auto=审批自动 / manual=手动
    pub trigger_type: Option<String>,
    /// 状态（1成功 0失败）
    pub status: Option<i32>,
    /// 失败原因
    pub error_msg: Option<String>,
    /// 创建人ID
    pub create_by: Option<i64>,
    /// 创建时间
    pub create_time: Option<String>,
}

impl From<pdf_record::Model> for PdfRecordVO {
    fn from(m: pdf_record::Model) -> Self {
        Self {
            id: Option::from(m.id),
            doc_type: m.doc_type,
            doc_id: m.doc_id,
            doc_no: m.doc_no,
            template_id: m.template_id,
            template_name: m.template_name,
            file_name: m.file_name,
            file_path: m.file_path,
            file_url: m.file_url,
            file_size: m.file_size,
            page_count: m.page_count,
            trigger_type: m.trigger_type,
            status: m.status,
            error_msg: m.error_msg,
            create_by: m.create_by,
            create_time: m.create_time.map(|s| s.format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}

/// PDF 记录写入 DTO
#[derive(Debug, Clone)]
pub struct PdfRecordSaveDTO {
    /// 单据类型
    pub doc_type: Option<String>,
    /// 单据ID
    pub doc_id: Option<i64>,
    /// 单据编号
    pub doc_no: Option<String>,
    /// 使用的模板ID
    pub template_id: Option<i64>,
    /// 模板名称（冗余）
    pub template_name: Option<String>,
    /// 生成的文件名
    pub file_name: Option<String>,
    /// 服务器存储路径
    pub file_path: Option<String>,
    /// 访问URL
    pub file_url: Option<String>,
    /// 文件大小(字节)
    pub file_size: Option<i64>,
    /// 页数
    pub page_count: Option<i32>,
    /// 触发方式：auto=审批自动 / manual=手动
    pub trigger_type: Option<String>,
    /// 状态（1成功 0失败）
    pub status: Option<i32>,
    /// 失败原因
    pub error_msg: Option<String>,
    /// 创建人ID
    pub create_by: Option<i64>,
}

// ============================ Model 层数据访问 ============================

pub struct PdfTemplateModel;

impl PdfTemplateModel {
    /// 新增模板
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        req: &PdfTemplateSaveRequest,
        user_id: Option<i64>,
    ) -> Result<i64, DbErr> {
        let now = Local::now().naive_utc();
        let payload = pdf_template::ActiveModel {
            name: Set(req.name.clone()),
            template_code: Set(req.template_code.clone()),
            doc_type: Set(req.doc_type.clone()),
            content: Set(req.content.clone()),
            header_content: Set(req.header_content.clone()),
            footer_content: Set(req.footer_content.clone()),
            paper_size: Set(req.paper_size.clone()),
            orientation: Set(req.orientation.clone()),
            margin_top: Set(req.margin_top),
            margin_bottom: Set(req.margin_bottom),
            margin_left: Set(req.margin_left),
            margin_right: Set(req.margin_right),
            font_family: Set(req.font_family.clone()),
            is_default: Set(req.is_default),
            status: Set(req.status),
            sort: Set(req.sort),
            remark: Set(req.remark.clone()),
            create_by: Set(user_id),
            create_time: Set(Some(now)),
            update_by: Set(user_id),
            update_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        PdfTemplate::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 修改模板
    pub async fn update<C: ConnectionTrait>(
        db: &C,
        id: i64,
        req: &PdfTemplateUpdateRequest,
        user_id: Option<i64>,
    ) -> Result<i64, DbErr> {
        let payload = pdf_template::ActiveModel {
            name: Set(req.name.clone()),
            template_code: Set(req.template_code.clone()),
            doc_type: Set(req.doc_type.clone()),
            content: Set(req.content.clone()),
            header_content: Set(req.header_content.clone()),
            footer_content: Set(req.footer_content.clone()),
            paper_size: Set(req.paper_size.clone()),
            orientation: Set(req.orientation.clone()),
            margin_top: Set(req.margin_top),
            margin_bottom: Set(req.margin_bottom),
            margin_left: Set(req.margin_left),
            margin_right: Set(req.margin_right),
            font_family: Set(req.font_family.clone()),
            is_default: Set(req.is_default),
            status: Set(req.status),
            sort: Set(req.sort),
            remark: Set(req.remark.clone()),
            update_by: Set(user_id),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        let r = PdfTemplate::update_many()
            .set(payload)
            .filter(pdf_template::Column::Id.eq(id))
            .filter(pdf_template::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(r.rows_affected as i64)
    }

    /// 分页查询
    pub async fn select_in_page(
        db: &DbConn,
        page: i64,
        page_size: i64,
        name: Option<String>,
        doc_type: Option<String>,
        status: Option<i32>,
    ) -> Result<(Vec<pdf_template::Model>, u64), DbErr> {
        let paginator = PdfTemplate::find()
            .filter(pdf_template::Column::Deleted.eq(0))
            .apply_if(name, |q, v| {
                q.filter(pdf_template::Column::Name.contains(format!("%{}%", v)))
            })
            .apply_if(doc_type, |q, v| {
                q.filter(pdf_template::Column::DocType.eq(v))
            })
            .apply_if(status, |q, v| q.filter(pdf_template::Column::Status.eq(v)))
            .order_by_desc(pdf_template::Column::Id)
            .paginate(db, page_size as u64);
        let num_pages = paginator.num_pages().await?;
        let list = paginator.fetch_page((page - 1) as u64).await?;
        Ok((list, num_pages))
    }

    /// 数量统计
    pub async fn select_count(
        db: &DbConn,
        name: Option<String>,
        doc_type: Option<String>,
        status: Option<i32>,
    ) -> Result<i64, DbErr> {
        PdfTemplate::find()
            .filter(pdf_template::Column::Deleted.eq(0))
            .apply_if(name, |q, v| {
                q.filter(pdf_template::Column::Name.contains(format!("%{}%", v)))
            })
            .apply_if(doc_type, |q, v| {
                q.filter(pdf_template::Column::DocType.eq(v))
            })
            .apply_if(status, |q, v| q.filter(pdf_template::Column::Status.eq(v)))
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 根据 ID 查询
    pub async fn find_by_id(
        db: &DbConn,
        id: i64,
    ) -> Result<Option<pdf_template::Model>, DbErr> {
        PdfTemplate::find_by_id(id)
            .filter(pdf_template::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 查询指定单据类型的默认模板
    pub async fn find_default(
        db: &DbConn,
        doc_type: &str,
    ) -> Result<Option<pdf_template::Model>, DbErr> {
        PdfTemplate::find()
            .filter(pdf_template::Column::DocType.eq(doc_type))
            .filter(pdf_template::Column::IsDefault.eq(1))
            .filter(pdf_template::Column::Status.eq(1))
            .filter(pdf_template::Column::Deleted.eq(0))
            .one(db)
            .await
    }

    /// 查询指定单据类型的模板选项列表
    pub async fn find_options(
        db: &DbConn,
        doc_type: &str,
    ) -> Result<Vec<pdf_template::Model>, DbErr> {
        PdfTemplate::find()
            .filter(pdf_template::Column::DocType.eq(doc_type))
            .filter(pdf_template::Column::Status.eq(1))
            .filter(pdf_template::Column::Deleted.eq(0))
            .order_by_asc(pdf_template::Column::Sort)
            .all(db)
            .await
    }

    /// 设为默认（先取消同类型其他默认，再设置当前为默认）
    pub async fn set_default<C: ConnectionTrait>(
        db: &C,
        id: i64,
        doc_type: &str,
    ) -> Result<i64, DbErr> {
        // 先取消同类型其他默认
        let reset_payload = pdf_template::ActiveModel {
            is_default: Set(Some(0)),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        PdfTemplate::update_many()
            .set(reset_payload)
            .filter(pdf_template::Column::DocType.eq(doc_type))
            .filter(pdf_template::Column::IsDefault.eq(1))
            .filter(pdf_template::Column::Deleted.eq(0))
            .exec(db)
            .await?;

        // 再设置当前为默认
        let set_payload = pdf_template::ActiveModel {
            is_default: Set(Some(1)),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        let r = PdfTemplate::update_many()
            .set(set_payload)
            .filter(pdf_template::Column::Id.eq(id))
            .filter(pdf_template::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(r.rows_affected as i64)
    }

    /// 软删除
    pub async fn bath_delete<C: ConnectionTrait>(
        db: &C,
        ids: &[i64],
    ) -> Result<i64, DbErr> {
        let payload = pdf_template::ActiveModel {
            deleted: Set(Some(1)),
            update_time: Set(Some(Local::now().naive_utc())),
            ..Default::default()
        };
        let r = PdfTemplate::update_many()
            .set(payload)
            .filter(pdf_template::Column::Id.is_in(ids.to_vec()))
            .filter(pdf_template::Column::Deleted.eq(0))
            .exec(db)
            .await?;
        Ok(r.rows_affected as i64)
    }
}

pub struct PdfRecordModel;

impl PdfRecordModel {
    /// 写入 PDF 记录
    pub async fn insert<C: ConnectionTrait>(
        db: &C,
        dto: &PdfRecordSaveDTO,
    ) -> Result<i64, DbErr> {
        let now = Local::now().naive_utc();
        let payload = pdf_record::ActiveModel {
            doc_type: Set(dto.doc_type.clone()),
            doc_id: Set(dto.doc_id),
            doc_no: Set(dto.doc_no.clone()),
            template_id: Set(dto.template_id),
            template_name: Set(dto.template_name.clone()),
            file_name: Set(dto.file_name.clone()),
            file_path: Set(dto.file_path.clone()),
            file_url: Set(dto.file_url.clone()),
            file_size: Set(dto.file_size),
            page_count: Set(dto.page_count),
            trigger_type: Set(dto.trigger_type.clone()),
            status: Set(dto.status),
            error_msg: Set(dto.error_msg.clone()),
            create_by: Set(dto.create_by),
            create_time: Set(Some(now)),
            deleted: Set(Some(0)),
            ..Default::default()
        };
        PdfRecord::insert(payload)
            .exec(db)
            .await
            .map(|r| r.last_insert_id)
    }

    /// 按单据分页查询记录
    pub async fn select_by_doc(
        db: &DbConn,
        doc_type: &str,
        doc_id: i64,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<pdf_record::Model>, u64), DbErr> {
        let paginator = PdfRecord::find()
            .filter(pdf_record::Column::DocType.eq(doc_type))
            .filter(pdf_record::Column::DocId.eq(doc_id))
            .filter(pdf_record::Column::Deleted.eq(0))
            .order_by_desc(pdf_record::Column::Id)
            .paginate(db, page_size as u64);
        let num_pages = paginator.num_pages().await?;
        let list = paginator.fetch_page((page - 1) as u64).await?;
        Ok((list, num_pages))
    }

    /// 按单据统计记录数量
    pub async fn select_count_by_doc(
        db: &DbConn,
        doc_type: &str,
        doc_id: i64,
    ) -> Result<i64, DbErr> {
        PdfRecord::find()
            .filter(pdf_record::Column::DocType.eq(doc_type))
            .filter(pdf_record::Column::DocId.eq(doc_id))
            .filter(pdf_record::Column::Deleted.eq(0))
            .count(db)
            .await
            .map(|c| c as i64)
    }

    /// 全局分页查询记录（支持按单据类型/单据编号/触发方式筛选）
    pub async fn select_all(
        db: &DbConn,
        doc_type: Option<&str>,
        doc_no: Option<&str>,
        trigger_type: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<pdf_record::Model>, u64), DbErr> {
        let mut query = PdfRecord::find()
            .filter(pdf_record::Column::Deleted.eq(0));

        if let Some(dt) = doc_type {
            if !dt.is_empty() {
                query = query.filter(pdf_record::Column::DocType.eq(dt));
            }
        }
        if let Some(dn) = doc_no {
            if !dn.is_empty() {
                query = query.filter(pdf_record::Column::DocNo.contains(dn));
            }
        }
        if let Some(tt) = trigger_type {
            if !tt.is_empty() {
                query = query.filter(pdf_record::Column::TriggerType.eq(tt));
            }
        }

        let paginator = query
            .order_by_desc(pdf_record::Column::Id)
            .paginate(db, page_size as u64);
        let num_pages = paginator.num_pages().await?;
        let list = paginator.fetch_page((page - 1) as u64).await?;
        Ok((list, num_pages))
    }

    /// 全局统计记录数量
    pub async fn select_count_all(
        db: &DbConn,
        doc_type: Option<&str>,
        doc_no: Option<&str>,
        trigger_type: Option<&str>,
    ) -> Result<i64, DbErr> {
        let mut query = PdfRecord::find()
            .filter(pdf_record::Column::Deleted.eq(0));

        if let Some(dt) = doc_type {
            if !dt.is_empty() {
                query = query.filter(pdf_record::Column::DocType.eq(dt));
            }
        }
        if let Some(dn) = doc_no {
            if !dn.is_empty() {
                query = query.filter(pdf_record::Column::DocNo.contains(dn));
            }
        }
        if let Some(tt) = trigger_type {
            if !tt.is_empty() {
                query = query.filter(pdf_record::Column::TriggerType.eq(tt));
            }
        }

        query.count(db).await.map(|c| c as i64)
    }
}
