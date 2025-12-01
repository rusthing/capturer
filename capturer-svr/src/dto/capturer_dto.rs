use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(ToSchema, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CapturerCaptureRtspToJpgDto {
    /// RTSP地址
    #[validate(
        required(message = "RTSP地址不能为空"),
        length(min = 1, message = "RTSP地址不能为空")
    )]
    pub rtsp_url: Option<String>,
    /// 存储桶
    pub bucket: Option<String>,
}
