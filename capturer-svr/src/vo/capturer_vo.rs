use serde::Serialize;
use serde_with::skip_serializing_none;
use utoipa::ToSchema;

#[skip_serializing_none] // 忽略空字段(好像必须放在#[derive(o2o, Serialize)]的上方才能起效)
#[derive(ToSchema, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CapturerCaptureVo {
    /// 数据
    pub data: Vec<u8>,
}
