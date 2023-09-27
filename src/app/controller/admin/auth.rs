use actix_web::{
    http::{header::ContentType, StatusCode},
    web, Error, HttpRequest, HttpResponse, Result,
};
use serde::Deserialize;

use captcha::filters::{Dots, Noise, Wave};
use captcha::Captcha;

use crate::nako::{
    auth as nako_auth,
    global::{AppState, Session, Validate},
    http as nako_http, rsa, utils,
};

use crate::app::model::user;

const AUTH_KEY: &str = "nako:auth_key";

// 验证码
pub async fn captcha(session: Session) -> Result<HttpResponse> {
    let mut c = Captcha::new();

    let c = c
        .add_chars(4)
        .apply_filter(Noise::new(0.4))
        .apply_filter(Wave::new(2.0, 20.0).horizontal())
        // .apply_filter(Wave::new(2.0, 20.0).vertical())
        .apply_filter(Dots::new(15))
        .view(260, 96);

    if let Some((data, png_data)) = c.as_tuple() {
        if !session.insert("auth_captcha", data).is_err() {
            return Ok(HttpResponse::build(StatusCode::OK)
                .content_type(ContentType::png())
                .body(png_data));
        }
    }

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type(ContentType::plaintext())
        .body("nodata".to_string()))
}

// 登陆
pub async fn login(
    session: Session,
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let login_id = session
        .get::<u32>("login_id")
        .unwrap_or_default()
        .unwrap_or_default();
    if login_id > 0 {
        let redirect_url: String = utils::url_for_static(req, "admin.index");

        return Ok(nako_http::redirect(redirect_url));
    }

    let mut view = state.view.clone();

    let mut ctx = nako_http::view_data();

    if let Ok((pri_key, pub_key)) = rsa::generate_key(1024) {
        let pub_key = pub_key
            .replace("-----BEGIN PUBLIC KEY-----", "")
            .replace("-----END PUBLIC KEY-----", "")
            .replace("\r\n", "")
            .replace("\r", "")
            .replace("\n", "")
            .replace(" ", "");

        ctx.insert("pub_key", &pub_key.trim());

        if let Ok(_) = session.insert(AUTH_KEY, pri_key.clone()) {}
    }

    Ok(nako_http::view(&mut view, "admin/auth/login.html", &ctx))
}

#[derive(Deserialize, Clone)]
pub struct LoginParams {
    name: String,
    password: String,
    captcha: String,
}

#[derive(Debug, Validate, Deserialize, Clone)]
pub struct LoginValidate {
    #[validate(required(message = "账号不能为空"))]
    name: Option<String>,
    #[validate(required(message = "密码不能为空"))]
    password: Option<String>,
    #[validate(
        required(message = "验证码不能为空"),
        length(min = 4, message = "验证码位数错误")
    )]
    captcha: Option<String>,
}

// 提交登陆
pub async fn login_check(
    session: Session,
    params: web::Form<LoginParams>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let login_id = session
        .get::<u32>("login_id")
        .unwrap_or_default()
        .unwrap_or_default();
    if login_id > 0 {
        return Ok(nako_http::error_response_json("你已经登陆了"));
    }

    let vali_data = LoginValidate {
        name: Some(params.name.clone()),
        password: Some(params.password.clone()),
        captcha: Some(params.captcha.clone()),
    };

    let vali = vali_data.validate();
    if vali.is_err() {
        return Ok(nako_http::error_response_json(
            format!("{}", vali.unwrap_err()).as_str(),
        ));
    }

    let auth_captcha = session
        .get::<String>("auth_captcha")
        .unwrap_or_default()
        .unwrap_or_default();
    if params.captcha.to_uppercase() != auth_captcha.to_uppercase() {
        return Ok(nako_http::error_response_json("验证码错误"));
    }

    let db = &state.db;
    let user_info = user::UserModel::find_user_by_name(db, params.name.as_str())
        .await
        .unwrap_or_default()
        .unwrap_or_default();

    if user_info.id == 0 {
        return Ok(nako_http::error_response_json("账号或者密码错误"));
    }

    let pass = user_info.password.unwrap_or("".to_string());

    // 私钥
    let prikey = session
        .get::<String>(AUTH_KEY)
        .unwrap_or_default()
        .unwrap_or_default();

    // 解出密码
    let params_pass = utils::base64_decode(params.password.clone());
    let depass = rsa::decrypt(prikey.as_str(), params_pass.as_slice()).unwrap_or_default();

    let depass = String::from_utf8(depass).unwrap_or("".to_string());

    // 验证密码
    if !nako_auth::password_verify(depass.as_str(), pass.as_str()) {
        return Ok(nako_http::error_response_json("账号或者密码错误"));
    }

    let status = user_info.status.unwrap_or(0);
    if status == 0 {
        return Ok(nako_http::error_response_json("账号不存在或者已被禁用"));
    }

    if session.insert("login_id", user_info.id).is_err() {
        return Ok(nako_http::error_response_json("登陆失败"));
    }

    session.remove(AUTH_KEY);

    Ok(nako_http::success_response_json("登陆成功", ""))
}

// 退出
pub async fn logout(req: HttpRequest, session: Session) -> Result<HttpResponse, Error> {
    let login_id = session
        .get::<u32>("login_id")
        .unwrap_or_default()
        .unwrap_or_default();
    if login_id > 0 {
        session.remove("login_id");
    }

    let redirect_url: String = utils::url_for_static(req, "admin.auth-login");

    return Ok(nako_http::redirect(redirect_url));
}
