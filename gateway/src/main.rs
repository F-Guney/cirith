mod rate_limit;

use async_trait::async_trait;
use pingora::Result;
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::server::Server;
use pingora::upstreams::peer::HttpPeer;
use pingora_proxy::{ProxyHttp, Session};
// imports
use crate::rate_limit::RateLimiter;
use cirith_shared::storage::{Database, DbRoute};
use cirith_shared::{auth::AuthValidator, config::Config};

struct CirithGateway {
    config: Config,
    rate_limit: RateLimiter,
    auth_validator: AuthValidator,
    routes: Vec<DbRoute>,
}

#[async_trait]
impl ProxyHttp for CirithGateway {
    type CTX = String;

    fn new_ctx(&self) -> Self::CTX {
        String::new()
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();
        let route = self
            .routes
            .iter()
            .filter(|r| path.starts_with(&r.path))
            .max_by_key(|r| r.path.len());

        match route {
            Some(r) => {
                tracing::info!(path = %path, upstream = %r.upstream, "Routing request");

                let is_https = r.upstream.starts_with("https://");
                let host = r
                    .upstream
                    .trim_start_matches("http://")
                    .trim_start_matches("https://");

                let port: u16 = if is_https { 443 } else { 80 };
                *ctx = host.to_string();
                let peer = HttpPeer::new((host, port), is_https, host.to_string());
                Ok(Box::new(peer))
            }
            None => {
                tracing::warn!(path = %path, "No route found");
                Err(pingora::Error::new_str("No route found"))
            }
        }
    }

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        let path = session.req_header().uri.path();
        if path == "/health" {
            let header = ResponseHeader::build(200, None)?;
            session.set_keepalive(None);
            session
                .write_response_header(Box::new(header), false)
                .await?;

            session.write_response_body(Some("OK".into()), true).await?;

            return Ok(true);
        }

        if self.auth_validator.is_enabled() {
            let api_key = session
                .req_header()
                .headers
                .get("x-api-key")
                .and_then(|v| v.to_str().ok());

            match api_key {
                Some(key) => {
                    if !self.auth_validator.validate(key) {
                        tracing::warn!("Invalid API key");

                        let header = ResponseHeader::build(401, None)?;
                        session.set_keepalive(None);
                        session
                            .write_response_header(Box::new(header), true)
                            .await?;

                        return Ok(true);
                    }
                }
                None => {
                    tracing::warn!("Missing API key");
                    let header = ResponseHeader::build(401, None)?;
                    session.set_keepalive(None);
                    session
                        .write_response_header(Box::new(header), false)
                        .await?;

                    return Ok(true);
                }
            }
        }

        let client_ip = session
            .client_addr()
            .map(|addr| addr.as_inet().map(|inet| inet.ip()))
            .flatten();

        let ip = match client_ip {
            Some(ip) => ip,
            None => {
                tracing::warn!("Could not get client IP");
                return Ok(false);
            }
        };

        if !self.rate_limit.check(ip) {
            tracing::warn!(ip = %ip, "Rate limit exceeded");

            let mut header = ResponseHeader::build(429, None)?;
            header.insert_header(
                "X-Rate-Limit-Limit",
                self.config.rate_limit.max_requests.to_string(),
            )?;

            header.insert_header("X-Rate-Limit-Remaining", "0")?;
            session.set_keepalive(None);
            session
                .write_response_header(Box::new(header), true)
                .await?;

            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request.insert_header("Host", ctx.as_str())?;
        Ok(())
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Cirith Gateway...");

    let config = Config::load("config.yml").expect("Failed to load config");
    let port = config.server.gateway_port;

    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let rate_limit = RateLimiter::new(
        config.rate_limit.max_requests,
        config.rate_limit.window_secs,
    );

    let rt = tokio::runtime::Runtime::new().unwrap();
    let routes = rt
        .block_on(async {
            let database = Database::new(&config.database.url).await?;
            database.get_routes().await
        })
        .unwrap();

    let auth_validator = AuthValidator::new(&config.auth);
    let gateway = CirithGateway {
        config,
        rate_limit,
        auth_validator,
        routes,
    };

    let mut proxy = pingora_proxy::http_proxy_service(&server.configuration, gateway);
    proxy.add_tcp(&format!("0.0.0.0:{}", port));
    tracing::info!("Listening on 0.0.0.0:{}", port);

    server.add_service(proxy);
    server.run_forever();
}
