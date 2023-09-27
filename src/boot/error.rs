use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    error::{InternalError, JsonPayloadError, PathError, QueryPayloadError, UrlencodedError},
    http::Method,
    middleware::ErrorHandlerResponse,
    web, Error, HttpRequest, HttpResponse, Responder, Result,
};

use crate::nako::{app, global::AppState, http as nako_http};

pub(crate) async fn app_default(req: HttpRequest) -> impl Responder {
    get_error_response(&req, "404 Not Found")
}

pub(crate) fn json_parser_error(err: JsonPayloadError, req: &HttpRequest) -> Error {
    let mut err_message = err.to_string();
    if !app::is_debug() {
        err_message = "json error".to_string();
    }

    let resp = get_error_response(&req, err_message.as_str());

    InternalError::from_response(err, resp).into()
}

pub(crate) fn form_parser_error(err: UrlencodedError, req: &HttpRequest) -> Error {
    let mut err_message = err.to_string();
    if !app::is_debug() {
        err_message = "form empty".to_string();
    }

    let resp = get_error_response(&req, err_message.as_str());

    InternalError::from_response(err, resp).into()
}

pub(crate) fn query_parser_error(err: QueryPayloadError, req: &HttpRequest) -> Error {
    let mut err_message = err.to_string();
    if !app::is_debug() {
        err_message = "query empty".to_string();
    }

    let resp = get_error_response(&req, err_message.as_str());

    InternalError::from_response(err, resp).into()
}

pub(crate) fn path_parser_error(err: PathError, req: &HttpRequest) -> Error {
    let mut err_message = err.to_string();
    if !app::is_debug() {
        err_message = "path error".to_string();
    }

    let resp = get_error_response(&req, err_message.as_str());

    InternalError::from_response(err, resp).into()
}

// 404
pub(crate) fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<BoxBody>> {
    let req = res.request();

    let response = get_error_response(&req, "Page not found");

    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        res.into_parts().0,
        response.map_into_left_body(),
    )))
}

// 获取响应
fn get_error_response(req: &HttpRequest, error: &str) -> HttpResponse {
    if let Some(state) = req.app_data::<web::Data<AppState>>() {
        let mut view = state.view.clone();

        if req.method() == Method::POST {
            return nako_http::error_response_json(error);
        }

        return nako_http::error_response_html(&mut view, error, "");
    }

    nako_http::text(error.to_string())
}
