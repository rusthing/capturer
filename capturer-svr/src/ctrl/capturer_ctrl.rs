use crate::dto::capturer_dto::CapturerCaptureRtspToJpgDto;
use crate::svc::capturer_svc::CapturerSvc;
use crate::vo::capturer_vo::CapturerCaptureVo;
use actix_web::{post, web, HttpResponse, Result};
use robotech::ctrl::ctrl_error::CtrlError;
use robotech::ro::Ro;
use validator::Validate;

#[utoipa::path(
    path = "/capturer/capture_rtsp_to_jpg",
    responses((status = OK, body = Ro<CapturerCaptureVo>))
)]
#[post("/capturer/capture_rtsp_to_jpg")]
pub async fn capture_rtsp_to_jpg(
    json_body: web::Json<CapturerCaptureRtspToJpgDto>,
) -> Result<HttpResponse, CtrlError> {
    let dto = json_body.into_inner();

    dto.validate()?;

    let result = CapturerSvc::capture_rtsp_to_jpg(dto)?;
    Ok(HttpResponse::Ok().json(result))
}
