use actix_web::{web, Error, HttpResponse, Result};

use crate::nako::global::{AppState, Session};
use crate::nako::http as nako_http;

use crate::app::model::{art, cate, comment, tag, user};

// 首页
pub async fn index(state: web::Data<AppState>, session: Session) -> Result<HttpResponse, Error> {
    let db = &state.db;
    let mut view = state.view.clone();

    let id = session
        .get::<u32>("login_id")
        .unwrap_or_default()
        .unwrap_or_default();
    let user_info = user::UserModel::find_user_by_id(db, id)
        .await
        .unwrap_or_default()
        .unwrap_or_default();

    let mut ctx = nako_http::view_data();
    ctx.insert("login_user", &user_info);

    Ok(nako_http::view(&mut view, "admin/index/index.html", &ctx))
}

// 控制台
pub async fn console(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let db = &state.db;
    let mut view = state.view.clone();

    let search_where = art::ArtWhere {
        title: None,
        uuid: None,
        tag: None,
        cate_id: None,
        user_id: None,
        is_top: None,
        status: Some(1),
    };
    let (new_arts, _) = art::ArtModel::search_in_page(db, 1, 6, search_where.clone())
        .await
        .unwrap_or_default();

    let art_count = art::ArtModel::find_count(db).await.unwrap_or(0);
    let cate_count = cate::CateModel::find_count(db).await.unwrap_or(0);
    let comment_count = comment::CommentModel::find_count(db).await.unwrap_or(0);
    let tag_count = tag::TagModel::find_count(db).await.unwrap_or(0);

    let mut ctx = nako_http::view_data();
    ctx.insert("new_arts", &new_arts);

    ctx.insert("art_count", &art_count);
    ctx.insert("cate_count", &cate_count);
    ctx.insert("comment_count", &comment_count);
    ctx.insert("tag_count", &tag_count);

    Ok(nako_http::view(&mut view, "admin/index/console.html", &ctx))
}
