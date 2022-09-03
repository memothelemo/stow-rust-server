use dashmap::DashMap;
use rbxcloud::rbx::RbxCloud;
use std::{sync::Arc, time::SystemTime};
use uuid::Uuid;

/// Routes that do not require server token given from [`register`](crate::routes::register) route
pub const NO_TOKEN_ROUTES: &[&str] = &[
    "/register",
    // we're going to be soft with logout route because I don't know
    // how to deal with this in Postman (API testing program)
    #[cfg(debug_assertions)]
    "/logout",
];

/// Performs inactive registered servers periodically (for 5 minutes) to avoid
/// filling up the entire memory of the system and maybe more?
///
/// It will check if the inactivity time from any Roblox
/// servers is 10 minutes or longer.
pub async fn run_cleanup(servers: ActiveServerMap) {
    log::info!("[Cleanup] starting garbage collection");

    const INTERVAL: tokio::time::Duration = tokio::time::Duration::from_secs(60 * 5);
    const MAXIMUM_INACTIVITY: std::time::Duration = std::time::Duration::from_secs(60 * 10);
    loop {
        log::trace!(
            "[Cleanup] loop iteration started, sleeping for {:?}",
            INTERVAL
        );
        tokio::time::sleep(INTERVAL).await;
        log::debug!("[Cleanup] begin collecting inactive servers");
        for entry in servers.iter() {
            let inactivity_elapsed = entry.last_activity.elapsed().unwrap_or_default();
            if inactivity_elapsed > MAXIMUM_INACTIVITY {
                let server_id = entry.key();
                log::warn!(
                    "Inactive server detected (server = {}; >= {:?}), logging out",
                    server_id,
                    MAXIMUM_INACTIVITY,
                );
                servers.remove(server_id);
            }
        }
        log::trace!("[Cleanup] loop iteration ended");
    }
}

pub type ActiveServerMap = Arc<DashMap<String, ActiveServerInfo>>;

/// The server state during its session.
#[derive(Debug)]
pub struct ServerSession {
    /// Active servers during the server's session.
    ///
    /// It will be wipe clean after its session is terminated.
    pub active_servers: ActiveServerMap,

    /// Roblox Open Cloud API client
    pub rbxcloud: RbxCloud,
}

impl ServerSession {
    /// Registers an active server.
    ///
    /// It returns the token provided from this method
    /// in order to authenticate on later use.
    pub fn register_server(&self, id: impl Into<String>) -> String {
        let token = Uuid::new_v4().to_string();
        self.active_servers.insert(
            id.into(),
            ActiveServerInfo {
                last_activity: SystemTime::now(),
                token: token.to_string(),
            },
        );
        token
    }

    /// Logs out the active server.
    ///
    /// It will return `false` if it already logs out
    /// from the previous method call gave from the same server id.
    pub fn logout_server(&self, id: &str) -> bool {
        self.active_servers.remove(id).is_some()
    }
}

/// Information of the active server.
#[derive(Debug)]
pub struct ActiveServerInfo {
    /// Last activity requested to the server
    pub last_activity: SystemTime,

    /// Token provided from [`/register`](crate::routes::register) route.
    pub token: String,
}
