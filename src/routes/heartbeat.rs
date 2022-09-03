use crate::common::ServerSession;
use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

#[actix_web::get("/heartbeat")]
pub async fn heartbeat(data: web::Data<ServerSession>, req: HttpRequest) -> HttpResponse {
    let server_id = req.headers().get("server").unwrap().to_str().unwrap();
    if let Some(info) = data.active_servers.get_mut(server_id) {
        todo!()
    } else {
        HttpResponse::Forbidden().json(json!({
            "success": false,
            "message": "Not registered",
        }))
    }
}
