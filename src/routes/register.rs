use crate::common::ServerSession;
use actix_web::{web, HttpRequest, HttpResponse};
use serde_json::json;

#[actix_web::post("/register")]
pub async fn register(session: web::Data<ServerSession>, req: HttpRequest) -> HttpResponse {
    // getting the server id
    let server_id = req.headers().get("server").unwrap().to_str().unwrap();
    if session.active_servers.contains_key(server_id) {
        HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Already registered!",
        }))
    } else {
        log::info!("Registered server (server = {})", server_id);
        HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Registered",
            "token": session.register_server(server_id),
        }))
    }
}
