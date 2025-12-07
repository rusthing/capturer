use crate::ctrl::capturer_ctrl::__path_capture_to_jpg;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(capture_to_jpg))]
pub struct CapturerApiDoc;
