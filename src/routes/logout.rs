use crate::common::ServerSession;
use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

#[actix_web::post("/logout")]
pub async fn logout(session: web::Data<ServerSession>, req: HttpRequest) -> HttpResponse {
    let server_id = req.headers().get("server").unwrap().to_str().unwrap();
    if session.logout_server(server_id) {
        log::info!("Logged out server (server = {})", server_id);
        HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Logged out",
        }))
    } else {
        HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Already logged out",
        }))
    }
}
