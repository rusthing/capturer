use crate::api_doc::capturer_api_doc::CapturerApiDoc;
use crate::ctrl::capturer_ctrl;
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};

/// # 配置WebService
pub fn web_service_config(cfg: &mut web::ServiceConfig) {
    cfg.service(capturer_ctrl::capture_to_jpg);
    cfg.service(SwaggerUi::new("/swagger-ui/{_:.*}").urls(vec![(
        Url::new("抓拍器", "/ctrl-docs/capturer-openapi.json"),
        CapturerApiDoc::openapi(),
    )]));
}
