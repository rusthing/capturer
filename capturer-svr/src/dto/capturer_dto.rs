use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(ToSchema, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CapturerCaptureToJpegDto {
    /// 抓拍流的地址
    #[validate(
        required(message = "抓拍流的地址不能为空"),
        length(min = 1, message = "抓拍流的地址不能为空")
    )]
    pub stream_url: Option<String>,
    /// 存储桶
    pub bucket: Option<String>,
    /// 当前用户ID
    #[serde(skip_deserializing)]
    pub current_user_id: u64,
}
