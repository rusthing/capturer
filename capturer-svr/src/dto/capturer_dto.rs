use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(ToSchema, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CapturerCaptureRtspToJpgDto {
    #[validate(
        required(message = "URL不能为空"),
        length(min = 1, message = "URL不能为空")
    )]
    pub url: Option<String>,
}
