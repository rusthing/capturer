use crate::dto::capturer_dto::{CapturerCaptureToJpegDto, CapturerGetStreamDto};
use crate::svc::capturer_svc::CapturerSvc;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Result};
use log::info;
use oss_api_client::vo::oss_obj_ref::OssObjRefVo;
use robotech::ro::Ro;
use robotech::web::ctrl_utils::get_current_user_id;
use robotech::web::CtrlError;
use validator::Validate;

#[utoipa::path(
    path = "/capturer/capture_to_jpeg",
    responses((status = OK, body = Ro<OssObjRefVo>))
)]
#[post("/capturer/capture_to_jpeg")]
pub async fn capture_to_jpg(
    json_body: web::Json<CapturerCaptureToJpegDto>,
    req: HttpRequest,
) -> Result<HttpResponse, CtrlError> {
    info!("capture_to_jpg");
    let mut dto = json_body.into_inner();

    dto.validate()?;

    // 从header中解析当前用户ID，如果没有或解析失败则抛出ApiError
    dto.current_user_id = get_current_user_id(req)?;

    let result = CapturerSvc::capture_to_jpeg(dto).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    path = "/capturer/stream.live.flv",
    responses((status = OK))
)]
#[get("/capturer/stream.live.flv")]
pub async fn stream(
    query_params: web::Query<CapturerGetStreamDto>,
    // req: HttpRequest,
) -> Result<HttpResponse, CtrlError> {
    info!("stream");
    let dto = query_params.into_inner();

    dto.validate()?;

    // TODO: 从header中解析当前用户ID，如果没有或解析失败则抛出ApiError
    // dto.current_user_id = get_current_user_id(req)?;

    Ok(HttpResponse::Ok()
        .content_type("video/x-flv")
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .streaming(CapturerSvc::stream(dto).await?))
}
