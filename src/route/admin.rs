use actix_web::web;
use actix_web_lab::middleware::from_fn;

use crate::app::middleware::{admin_auth};
use crate::app::controller::admin::{
    index,
    auth,
    user,
};

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("admin")
            .service(
                // 后台首页
                web::scope("/index")
                    .service(
                        web::resource("")
                            .route(web::get().to(index::index)).name("admin.index"),
                    )
                    .service(
                        web::resource("/menu")
                            .route(web::get().to(index::menu))
                            .name("admin.index-menu"),
                    )
                    .service(
                        web::resource("/console")
                            .route(web::get().to(index::console))
                            .name("admin.index-console"),
                    )
            )
            .service(
                // 登陆相关
                web::scope("/auth")
                    .service(
                        web::resource("/captcha")
                            .route(web::get().to(auth::captcha))
                            .name("admin.auth-captcha"),
                    )
                    .service(
                        web::resource("/login")
                            .route(web::get().to(auth::login))
                            .route(web::post().to(auth::login_check))
                            .name("admin.auth-login"),
                    )
                    .service(
                        web::resource("/logout")
                            .route(web::get().to(auth::logout))
                            .name("admin.auth-logout"),
                    )
            )
            .service(
                // 用户
                web::scope("/user")
                    .service(
                        web::resource("/index")
                            .route(web::get().to(user::index))
                            .name("admin.user-index"),
                    )
                    .service(
                        web::resource("/list")
                            .route(web::get().to(user::list))
                            .name("admin.user-list"),
                    )
                    .service(
                        web::resource("/detail")
                            .route(web::get().to(user::detail))
                            .name("admin.user-detail"),
                    )
                    .service(
                        web::resource("/create")
                            .route(web::get().to(user::create))
                            .route(web::post().to(user::create_save))
                            .name("admin.user-create"),
                    )
                    .service(
                        web::resource("/update")
                            .route(web::get().to(user::update))
                            .route(web::post().to(user::update_save))
                            .name("admin.user-update"),
                    )
                    .service(
                        web::resource("/status")
                            .route(web::post().to(user::update_status))
                            .name("admin.user-status"),
                    )
                    .service(
                        web::resource("/update-password")
                            .route(web::get().to(user::update_password))
                            .route(web::post().to(user::update_password_save))
                            .name("admin.user-update-password"),
                    )
                    .service(
                        web::resource("/delete")
                            .route(web::post().to(user::delete))
                            .name("admin.user-delete"),
                    )
            )
            .wrap(from_fn(admin_auth::auth)),
    );
}