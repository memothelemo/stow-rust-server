use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header;
use actix_web::{Error, HttpResponse};

use lazy_static::lazy_static;
use serde_json::json;

use std::future::{ready, Future, Ready};
use std::pin::Pin;

use crate::common::{ServerSession, NO_TOKEN_ROUTES};

lazy_static! {
    pub static ref AUTH_TOKEN: String = std::env::var("AUTH_TOKEN").unwrap();
}

pub struct Auth;

impl<S> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // check the authorization stuff there
        let mut authenticated = false;
        let headers = req.headers();
        if let Some(auth) = headers.get(header::AUTHORIZATION) {
            if let Ok(auth) = auth.to_str() {
                // strip the bearer
                if let Some(auth) = auth.strip_prefix("Bearer ") {
                    if AUTH_TOKEN.as_str() == auth {
                        authenticated = true;
                    }
                }
            }
        }

        let mut server = None;
        authenticated = if let Some(server_id) = headers.get("server") {
            if let Ok(server_id) = server_id.to_str() {
                server = Some(server_id);
                authenticated
            } else {
                false
            }
        } else {
            false
        };

        authenticated = if !NO_TOKEN_ROUTES.contains(&req.path()) {
            let mut authenticated = false;
            if let Some(token) = headers.get("token") {
                if let Ok(token) = token.to_str() {
                    let session = unsafe { req.app_data::<ServerSession>().unwrap_unchecked() };
                    if let Some(server) = session.active_servers.get(token) {
                        authenticated = server.token == token;
                    }
                }
            }
            authenticated
        } else {
            authenticated
        };

        if authenticated {
            log::debug!(
                "[Auth] authorized Roblox server ({}; {})",
                unsafe { server.unwrap_unchecked() },
                req.path()
            );
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        } else {
            Box::pin(async move {
                Ok(req.into_response(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "message": "Unauthorized.",
                }))))
            })
        }
    }
}
