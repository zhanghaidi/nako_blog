use actix_web::{web, Error, HttpResponse, Result};

use crate::nako::global::AppState;
use crate::nako::{app, http as nako_http};

use crate::app::model::{art, cate, friendlink, tag};

/// 首页
pub async fn index(state: web::Data<AppState>) -> Result<HttpResponse, Error> {
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
    let (arts, _num_pages) = art::ArtModel::search_in_page(db, 1, 4, search_where.clone())
        .await
        .unwrap_or_default();

    let hot_arts = art::ArtModel::find_one_year_hot(db, 6)
        .await
        .unwrap_or_default();
    let cates = cate::CateModel::find_open_cate(db)
        .await
        .unwrap_or_default();
    let tags = tag::TagModel::find_open_tags(db, 6)
        .await
        .unwrap_or_default();

    let friendlinks = friendlink::FriendlinkModel::list_open(db)
        .await
        .unwrap_or_default();

    let mut ctx = nako_http::view_data();
    ctx.insert("arts", &arts);
    ctx.insert("hot_arts", &hot_arts);
    ctx.insert("cates", &cates);
    ctx.insert("tags", &tags);

    ctx.insert("friendlinks", &friendlinks);

    Ok(nako_http::view(
        &mut view,
        app::view_path("index.html").as_str(),
        &ctx,
    ))
}
