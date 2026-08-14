use crate::dto::capturer_dto::{CapturerCaptureToJpegDto, CapturerGetStreamDto};
use crate::svc::capturer_svc::CapturerSvc;
use axum::body::Body;
use axum::extract::Query;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{debug_handler, Json};
use oss_api_client::vo::oss_obj_ref::OssObjRefVo;
use robotech::macros::log_call;
use robotech::ro::Ro;
use robotech::web::ctrl_utils::get_current_user_id;
use robotech::web::CtrlError;
use validator::Validate;

#[utoipa::path(
    post,
    path = "/capturer/capture_to_jpeg",
    responses((status = OK, body = Ro<OssObjRefVo>))
)]
#[log_call]
#[debug_handler]
pub async fn capture_to_jpeg(
    headers: HeaderMap,
    Json(mut dto): Json<CapturerCaptureToJpegDto>,
) -> Result<Json<Ro<serde_json::Value>>, CtrlError> {
    // 从header中解析当前用户ID，如果没有或解析失败则抛出ApiError
    dto._current_user_id = get_current_user_id(&headers)?;

    let result = CapturerSvc::capture_to_jpeg(dto).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/capturer/stream.live.flv",
    responses((status = OK))
)]
#[log_call]
#[debug_handler]
pub async fn stream(
    headers: HeaderMap,
    Query(dto): Query<CapturerGetStreamDto>,
    // req: HttpRequest,
) -> Result<Response, CtrlError> {
    dto.validate()?;

    // TODO: 从header中解析当前用户ID，如果没有或解析失败则抛出ApiError
    // dto._current_user_id = get_current_user_id(&headers)?;

    // 把 SvcError 转换为 CtrlError，再转为 Box<dyn std::error::Error + Send + Sync>
    let stream = CapturerSvc::stream(dto).await?;

    let body = Body::from_stream(stream);

    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("video/x-flv"),
    );
    response_headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    response_headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response_headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));

    Ok((StatusCode::OK, response_headers, body).into_response())
}
