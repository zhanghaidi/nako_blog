use actix_web::{
    web, 
    Error,
    HttpRequest,
    HttpResponse, 
    Result,
    http::{
        header::ContentType,
        StatusCode,
    },
};
use serde::{
    Deserialize
};

use captcha::Captcha;
use captcha::filters::{Noise, Wave, Dots};

use crate::nako::auth as nako_auth;
use crate::nako::http as nako_http;
use crate::nako::global::{
    Session, 
    AppState,
};

use crate::app::model::{
    user,
};

// 验证码
pub async fn captcha(
    session: Session, 
) -> Result<HttpResponse>{
    let mut c = Captcha::new();

    let c = c.add_chars(4) 
        .apply_filter(Noise::new(0.4))
        .apply_filter(Wave::new(2.0, 20.0).horizontal())
        .apply_filter(Wave::new(2.0, 20.0).vertical())
        .view(260, 96)
        .apply_filter(Dots::new(15));

    if let Some((data, png_data)) = c.as_tuple() {
        session.insert("auth_captcha", data).unwrap();

        // response
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::png())
            .body(png_data))
    } else { 
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::plaintext())
            .body("nodata".to_string()))
    }
}

// 登陆
pub async fn login(
    req: HttpRequest,
    session: Session, 
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    if let Some(_) = session.get::<u32>("login_id")? {
        let redirect_url: String = match req.url_for("admin.index", &[""]) {
            Ok(data) => data.into(),
            Err(_) => "/".into(),
        };
        
        return Ok(nako_http::redirect(redirect_url));
    } 

    let view = &state.view;

    let ctx = nako_http::view_ctx_new();

    Ok(nako_http::view(view, "admin/auth/login.html", &ctx))
}

#[derive(Deserialize)]
pub struct LoginParams {
    name: String,
    password: String,
    captcha: String,
}

// 提交登陆
pub async fn login_check(
    session: Session, 
    params: web::Form<LoginParams>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    if let Some(_) = session.get::<String>("login_id")? {
        return Ok(nako_http::error_response_json("你已经登陆了"));
    }

    if params.name.as_str() == "" {
        return Ok(nako_http::error_response_json("账号不能为空"));
    }
    if params.password.as_str() == "" {
        return Ok(nako_http::error_response_json("密码不能为空"));
    }
    if params.captcha.as_str() == "" {
        return Ok(nako_http::error_response_json("验证码不能为空"));
    }

    if let Some(auth_captcha) = session.get::<String>("auth_captcha")? {
        if params.captcha.to_uppercase() != auth_captcha.to_uppercase() {
            return Ok(nako_http::error_response_json("验证码错误"));
        }
    } else {
        return Ok(nako_http::error_response_json("验证码错误"));
    }

    let db = &state.db;
    let user_data = user::UserModel::find_user_by_name(db, params.name.as_str()).await;

    if let Ok(Some(user_info)) = user_data {
        let pass = match user_info.password {
            Some(x) => x,
            None => "".to_string()
        };

        if !nako_auth::password_verify(params.password.as_str(), pass.as_str()) {
            return Ok(nako_http::error_response_json("账号或者密码错误"));
        }

        let user_id: u32 = user_info.id;
        if user_id == 0 {
            return Ok(nako_http::error_response_json("账号或者密码错误"));
        }

        session.insert("login_id", user_id)?;
    } else {
        return Ok(nako_http::error_response_json("账号或者密码错误"));
    }

    Ok(nako_http::success_response_json("登陆成功", ""))
}

// 退出
pub async fn logout(
    req: HttpRequest,
    session: Session, 
) -> Result<HttpResponse, Error> {
    let id: Option<u32> = session.get("login_id")?;
    if let Some(_) = id {
        session.remove("login_id");
    }

    let redirect_url: String = match req.url_for("admin.auth-login", &[""]) {
        Ok(data) => data.into(),
        Err(_) => "/".into(),
    };
    
    return Ok(nako_http::redirect(redirect_url));
}

