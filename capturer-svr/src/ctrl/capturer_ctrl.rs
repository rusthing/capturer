use crate::dto::capturer_dto::{CapturerCaptureToJpegDto, CapturerGetStreamDto};
use crate::stream::stream_manager::StreamManager;
use crate::svc::capturer_svc::CapturerSvc;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Result};
use oss_api::vo::oss_obj_ref::OssObjRefVo;
use robotech::ctrl::ctrl_error::CtrlError;
use robotech::ctrl::ctrl_utils::get_current_user_id;
use robotech::ro::Ro;
use tokio::sync::broadcast;
use validator::Validate;
use wheel_rs::runtime::Error::RuntimeXError;

#[utoipa::path(
    path = "/capturer/capture_to_jpeg",
    responses((status = OK, body = Ro<OssObjRefVo>))
)]
#[post("/capturer/capture_to_jpeg")]
pub async fn capture_to_jpg(
    json_body: web::Json<CapturerCaptureToJpegDto>,
    req: HttpRequest,
) -> Result<HttpResponse, CtrlError> {
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
    let mut dto = query_params.into_inner();

    dto.validate()?;

    // 从header中解析当前用户ID，如果没有或解析失败则抛出ApiError
    // dto.current_user_id = get_current_user_id(req)?;

    let mut receiver = StreamManager::get_receiver(dto.stream_url.unwrap().as_str())
        .await
        .map_err(|e| RuntimeXError("获取流异常".to_string(), Box::new(e)))?;

    let stream = async_stream::stream! {
        loop {
            match receiver.recv().await {
                Ok(data) => yield Ok::<_, actix_web::Error>(data),
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    };
    Ok(HttpResponse::Ok()
        .content_type("video/x-flv")
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Cache-Control", "no-cache"))
        .append_header(("Connection", "keep-alive"))
        .streaming(stream))
}
