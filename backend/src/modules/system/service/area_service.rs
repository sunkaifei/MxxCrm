use sea_orm::DatabaseConnection;
use crate::modules::system::model::area::{AreaSaveDTO, AreaDetailVO, AreaTreeVO, AreaCascaderVO, AreaListQuery, AreaModel};
use crate::utils::response::ApiResponse;

pub struct AreaService;

macro_rules! api_ok {
    ($expr:expr, $msg:expr) => {
        match $expr.await {
            Ok(data) => ApiResponse::success_with_data($msg, data),
            Err(e) => ApiResponse::error(500, format!("{}: {}", $msg, e)),
        }
    };
}

impl AreaService {
    pub async fn get_area_tree(db: &DatabaseConnection) -> ApiResponse {
        let areas = AreaModel::find_all(db).await;
        let areas = match areas {
            Ok(a) => a,
            Err(e) => return ApiResponse::error(500, format!("查询地区数据失败: {}", e)),
        };

        let root_list: Vec<AreaTreeVO> = areas
            .iter()
            .filter(|a| a.parent_id == Some(0) || a.parent_id.is_none())
            .cloned()
            .map(|a| AreaTreeVO::from_model(&a, &areas))
            .collect();

        fn collect_children(pid: i64, all: &[crate::modules::system::entity::area::Model]) -> Vec<AreaTreeVO> {
            all.iter()
                .filter(|a| a.parent_id == Some(pid))
                .map(|a| {
                    let mut vo = AreaTreeVO::from_model(a, all);
                    vo.children = {
                        let ch = collect_children(a.id, all);
                        if ch.is_empty() { None } else { Some(ch) }
                    };
                    vo
                })
                .collect()
        }

        let tree: Vec<AreaTreeVO> = root_list.into_iter().map(|mut item| {
            if let Some(id) = item.id {
                let ch = collect_children(id, &areas);
                if !ch.is_empty() {
                    item.children = Some(ch);
                }
            }
            item
        }).collect();

        ApiResponse::success_with_data("获取成功", tree)
    }

    pub async fn get_cascader_data(db: &DatabaseConnection) -> ApiResponse {
        let areas = AreaModel::find_all(db).await;
        let areas = match areas {
            Ok(a) => a,
            Err(e) => return ApiResponse::error(500, format!("查询地区数据失败: {}", e)),
        };

        fn build_cascader(pid: i64, all: &[crate::modules::system::entity::area::Model]) -> Vec<AreaCascaderVO> {
            all.iter()
                .filter(|a| a.parent_id == Some(pid))
                .map(|a| AreaCascaderVO {
                    value: Some(a.id.to_string()),
                    label: a.name.clone(),
                    label_en: a.name_en.clone(),
                    children: {
                        let ch = build_cascader(a.id, all);
                        if ch.is_empty() { None } else { Some(ch) }
                    },
                })
                .collect()
        }

        let cascader = build_cascader(0, &areas);
        ApiResponse::success_with_data("获取成功", cascader)
    }

    pub async fn get_countries(db: &DatabaseConnection) -> ApiResponse {
        api_ok!(AreaModel::find_countries(db), "获取国家列表")
    }

    pub async fn get_provinces(db: &DatabaseConnection, country_code: String) -> ApiResponse {
        let areas = AreaModel::find_by_country_code(db, &country_code).await;
        let areas = match areas {
            Ok(a) => a,
            Err(e) => return ApiResponse::error(500, format!("查询省份数据失败: {}", e)),
        };

        let provinces: Vec<AreaDetailVO> = areas
            .into_iter()
            .filter(|a| a.level == Some(2))
            .map(AreaDetailVO::from)
            .collect();
        ApiResponse::success_with_data("获取成功", provinces)
    }

    pub async fn get_children(db: &DatabaseConnection, parent_id: String) -> ApiResponse {
        let pid: i64 = match parent_id.parse() {
            Ok(id) => id,
            Err(_) => return ApiResponse::error(400, "参数错误: parent_id 格式不正确".to_string()),
        };

        let children = AreaModel::find_by_parent_id(db, pid).await;
        let children = match children {
            Ok(c) => c,
            Err(e) => return ApiResponse::error(500, format!("查询子级数据失败: {}", e)),
        };

        let result: Vec<AreaDetailVO> = children.into_iter().map(AreaDetailVO::from).collect();
        ApiResponse::success_with_data("获取成功", result)
    }

    pub async fn get_detail(db: &DatabaseConnection, id: String) -> ApiResponse {
        let area_id: i64 = match id.parse() {
            Ok(id) => id,
            Err(_) => return ApiResponse::error(400, "参数错误: id 格式不正确".to_string()),
        };

        let area = AreaModel::find_by_id(db, area_id).await;
        let area = match area {
            Ok(a) => a,
            Err(e) => return ApiResponse::error(500, format!("查询详情失败: {}", e)),
        };

        match area {
            Some(a) => ApiResponse::success_with_data("获取成功", AreaDetailVO::from(a)),
            None => ApiResponse::error(404, "地区不存在".to_string()),
        }
    }

    pub async fn get_by_page(db: &DatabaseConnection, query: AreaListQuery) -> ApiResponse {
        let page = query.page_num.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(20);

        let result = AreaModel::select_in_page(
            db, page, page_size, query.name, query.level, query.country_code,
        ).await;

        let (list, total) = match result {
            Ok(r) => r,
            Err(e) => return ApiResponse::error(500, format!("分页查询失败: {}", e)),
        };

        let result: Vec<AreaDetailVO> = list.into_iter().map(AreaDetailVO::from).collect();
        ApiResponse::success_with_pagination("获取成功", result, total, page, page_size)
    }

    pub async fn insert(db: &DatabaseConnection, form_data: AreaSaveDTO) -> ApiResponse {
        let result = AreaModel::insert(db, &form_data).await;
        match result {
            Ok(id) => ApiResponse::success_with_data("新增成功", id),
            Err(e) => ApiResponse::error(500, format!("新增失败: {}", e)),
        }
    }

    pub async fn update(db: &DatabaseConnection, id: String, form_data: AreaSaveDTO) -> ApiResponse {
        let area_id: i64 = match id.parse() {
            Ok(id) => id,
            Err(_) => return ApiResponse::error(400, "参数错误: id 格式不正确".to_string()),
        };

        let affected = AreaModel::update_by_id(db, area_id, form_data).await;
        let affected = match affected {
            Ok(a) => a,
            Err(e) => return ApiResponse::error(500, format!("更新失败: {}", e)),
        };

        if affected > 0 {
            ApiResponse::success("更新成功")
        } else {
            ApiResponse::error(404, "地区不存在".to_string())
        }
    }

    pub async fn batch_delete(db: &DatabaseConnection, ids: Vec<String>) -> ApiResponse {
        let id_list: Vec<i64> = ids.into_iter()
            .filter_map(|s| s.parse::<i64>().ok())
            .collect();

        if id_list.is_empty() {
            return ApiResponse::error(400, "请选择要删除的地区".to_string());
        }

        let affected = AreaModel::batch_delete_by_ids(db, id_list).await;
        match affected {
            Ok(a) => ApiResponse::success_with_data("删除成功", a),
            Err(e) => ApiResponse::error(500, format!("删除失败: {}", e)),
        }
    }
}
