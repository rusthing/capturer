use crate::dto::capturer_dto::CapturerCaptureRtspToJpgDto;
use crate::svc::capturer_svc::CapturerSvc;
use actix_web::{post, web, HttpResponse, Result};
use oss_api::vo::oss_obj_ref::OssObjRefVo;
use robotech::ctrl::ctrl_error::CtrlError;
use robotech::ro::Ro;
use validator::Validate;

#[utoipa::path(
    path = "/capturer/capture_rtsp_to_jpg",
    responses((status = OK, body = Ro<OssObjRefVo>))
)]
#[post("/capturer/capture_rtsp_to_jpg")]
pub async fn capture_rtsp_to_jpg(
    json_body: web::Json<CapturerCaptureRtspToJpgDto>,
) -> Result<HttpResponse, CtrlError> {
    let dto = json_body.into_inner();

    dto.validate()?;

    let result = CapturerSvc::capture_rtsp_to_jpg(dto).await?;
    Ok(HttpResponse::Ok().json(result))
}
