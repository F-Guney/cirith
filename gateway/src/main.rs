use async_trait::async_trait;
use pingora::Result;
use pingora::http::RequestHeader;
use pingora::server::Server;
use pingora::upstreams::peer::HttpPeer;
use pingora_proxy::{ProxyHttp, Session};
// imports
use cirith_shared::config::Config;

struct CirithGateway {
    config: Config,
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
            .config
            .routes
            .iter()
            .filter(|r| path.starts_with(&r.path))
            .max_by_key(|r| r.path.len());

        match route {
            Some(r) => {
                tracing::info!(path = %path, upstream = %r.upstream, "Routing request");

                let host = r
                    .upstream
                    .trim_start_matches("http://")
                    .trim_start_matches("https://");

                *ctx = host.to_string();
                let peer = HttpPeer::new((host, 80), false, String::new());
                Ok(Box::new(peer))
            }
            None => {
                tracing::warn!(path = %path, "No route found");
                Err(pingora::Error::new_str("No route found"))
            }
        }
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
    tracing::info!("Loaded {} routes", config.routes.len());

    let mut server = Server::new(None).unwrap();
    server.bootstrap();
    let gateway = CirithGateway { config };
    let mut proxy = pingora_proxy::http_proxy_service(&server.configuration, gateway);
    proxy.add_tcp("0.0.0.0:6191");
    tracing::info!("Listening on 0.0.0.0:6191");

    server.add_service(proxy);
    server.run_forever();
}
