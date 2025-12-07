use crate::dto::capturer_dto::CapturerCaptureToJpegDto;
use crate::svc::capturer_svc::CapturerSvc;
use actix_web::{post, web, HttpRequest, HttpResponse, Result};
use oss_api::vo::oss_obj_ref::OssObjRefVo;
use robotech::ctrl::ctrl_error::CtrlError;
use robotech::ctrl::ctrl_utils::get_current_user_id;
use robotech::ro::Ro;
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
    let mut dto = json_body.into_inner();

    dto.validate()?;

    // 从header中解析当前用户ID，如果没有或解析失败则抛出ApiError
    dto.current_user_id = get_current_user_id(req)?;

    let result = CapturerSvc::capture_to_jpeg(dto).await?;
    Ok(HttpResponse::Ok().json(result))
}
