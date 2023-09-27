use actix_web::{web, Error, HttpResponse, Result};

use crate::nako::global::{AppState, Deserialize, Serialize};
use crate::nako::http as nako_http;

use crate::app::entity::{self, guestbook as guestbook_entity};
use crate::app::model::guestbook;
use crate::app::service::http;

// 首页
pub async fn index(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut view = state.view.clone();

    let ctx = nako_http::view_data();

    Ok(nako_http::view(
        &mut view,
        "admin/guestbook/index.html",
        &ctx,
    ))
}

// ==========================

#[derive(Serialize)]
pub struct ListData {
    list: Vec<guestbook_entity::Model>,
    count: u64,
}

#[derive(Deserialize)]
pub struct ListQuery {
    page: u64,
    limit: u64,

    name: Option<String>,
    message: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    qq: Option<String>,
    weixin: Option<String>,
    status: Option<i32>,
}

// 数据列表
pub async fn list(
    state: web::Data<AppState>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, Error> {
    let db = &state.db;

    let page: u64 = query.page;
    let per_page: u64 = query.limit;

    let search_where = guestbook::GuestbookWhere {
        name: query.name.clone(),
        message: query.message.clone(),
        phone: query.phone.clone(),
        email: query.email.clone(),
        qq: query.qq.clone(),
        weixin: query.weixin.clone(),
        status: query.status,
    };
    let search_where = search_where.format();

    let (list, _num_pages) =
        guestbook::GuestbookModel::search_in_page(db, page, per_page, search_where.clone())
            .await
            .unwrap_or_default();
    let count = guestbook::GuestbookModel::search_count(db, search_where.clone())
        .await
        .unwrap_or(0);

    let res: ListData = ListData {
        list: list,
        count: count,
    };

    Ok(nako_http::success_response_json("获取成功", res))
}

// ==========================

#[derive(Deserialize)]
pub struct DetailQuery {
    id: u32,
}

// 详情
pub async fn detail(
    state: web::Data<AppState>,
    query: web::Query<DetailQuery>,
) -> Result<HttpResponse, Error> {
    let db = &state.db;
    let mut view = state.view.clone();

    if query.id == 0 {
        return Ok(http::error_admin_html(&mut view, "ID不能为空", ""));
    }

    let data = guestbook::GuestbookModel::find_by_id(db, query.id)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    if data.id == 0 {
        return Ok(http::error_admin_html(&mut view, "留言不存在", ""));
    }

    let mut ctx = nako_http::view_data();
    ctx.insert("data", &data);

    Ok(nako_http::view(
        &mut view,
        "admin/guestbook/detail.html",
        &ctx,
    ))
}

// ==========================

#[derive(Deserialize)]
pub struct DeleteForm {
    id: u32,
}

// 删除
pub async fn delete(
    state: web::Data<AppState>,
    params: web::Form<DeleteForm>,
) -> Result<HttpResponse, Error> {
    let db = &state.db;

    if params.id == 0 {
        return Ok(nako_http::error_response_json("ID不能为空"));
    }

    let data = guestbook::GuestbookModel::find_by_id(db, params.id)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    if data.id == 0 {
        return Ok(nako_http::error_response_json("要删除的留言不存在"));
    }

    let delete_data = guestbook::GuestbookModel::delete(db, params.id).await;
    if delete_data.is_err() {
        return Ok(nako_http::error_response_json("删除失败"));
    }

    Ok(nako_http::success_response_json("删除成功", ""))
}

// ==========================

#[derive(Deserialize)]
pub struct BatchDeleteForm {
    ids: String,
}

// 批量删除
pub async fn batch_delete(
    state: web::Data<AppState>,
    params: web::Form<BatchDeleteForm>,
) -> Result<HttpResponse, Error> {
    let db = &state.db;

    if params.ids.as_str() == "" {
        return Ok(nako_http::error_response_json("未选中数据"));
    }

    let ids = params.ids.split(",").collect::<Vec<&str>>();

    for id in ids {
        let delete_id = id.parse::<u32>().unwrap_or_default();

        let data = guestbook::GuestbookModel::find_by_id(db, delete_id)
            .await
            .unwrap_or_default()
            .unwrap_or_default();
        if data.id > 0 {
            let _ = guestbook::GuestbookModel::delete(db, delete_id).await;
        }
    }

    Ok(nako_http::success_response_json("批量删除成功", ""))
}

// ==========================

#[derive(Deserialize)]
pub struct UpdateStatusQuery {
    id: u32,
}

// 表单数据
#[derive(Deserialize)]
pub struct UpdateStatusForm {
    status: i32,
}

// 更新保存
pub async fn update_status(
    state: web::Data<AppState>,
    query: web::Query<UpdateStatusQuery>,
    params: web::Form<UpdateStatusForm>,
) -> Result<HttpResponse, Error> {
    let db = &state.db;

    if query.id == 0 {
        return Ok(nako_http::error_response_json("ID不能为空"));
    }

    if params.status != 0 && params.status != 1 {
        return Ok(nako_http::error_response_json("状态不能为空"));
    }

    let data = guestbook::GuestbookModel::find_by_id(db, query.id)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    if data.id == 0 {
        return Ok(nako_http::error_response_json("要更改的留言不存在"));
    }

    // 更新
    let status = guestbook::GuestbookModel::update_status_by_id(
        db,
        query.id,
        guestbook_entity::Model {
            status: Some(params.status),
            ..entity::default()
        },
    )
    .await;
    if status.is_err() {
        return Ok(nako_http::error_response_json("更新失败"));
    }

    Ok(nako_http::success_response_json("更新成功", ""))
}
