use crate::ctrl::capturer_ctrl::{__path_capture_to_jpg, __path_stream};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(capture_to_jpg, stream))]
pub struct CapturerApiDoc;
