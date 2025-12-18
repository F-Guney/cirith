use std::net::IpAddr;
use url::Url;

static RESTRICTED_HOSTS: &[&str] = &["localhost", "metadata.google.internal"];

pub fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err(String::from("Empty path"));
    }

    if !path.starts_with('/') {
        return Err(String::from("Invalid path"));
    }

    if path.contains("..") {
        return Err(String::from("Invalid path"));
    }

    if path.contains("\0") {
        return Err(String::from("Invalid path"));
    }

    Ok(())
}

pub fn validate_upstream_url(upstream: &str) -> Result<(), String> {
    let upstream_url = Url::parse(upstream).map_err(|e| format!("Invalid URL: {}", e))?;

    match upstream_url.scheme() {
        "http" | "https" => {
            let host = upstream_url.host_str().ok_or("URL has no host")?;
            if let Ok(ip) = host.parse::<IpAddr>() {
                match ip {
                    IpAddr::V4(ipv4) => {
                        if ipv4.is_loopback() || ipv4.is_private() || ipv4.is_link_local() {
                            return Err(String::from("Invalid IP address"));
                        }
                        Ok(())
                    }
                    IpAddr::V6(ipv6) => {
                        if ipv6.is_loopback()
                            || ipv6.is_unspecified()
                            || ipv6.is_unicast_link_local()
                            || ipv6.is_unique_local()
                            || ipv6.is_multicast()
                        {
                            return Err(String::from("Invalid IP address"));
                        }
                        Ok(())
                    }
                }
            } else {
                let h = host.to_lowercase();
                if RESTRICTED_HOSTS
                    .iter()
                    .any(|restricted| h == *restricted || h.ends_with(&format!(".{}", restricted)))
                {
                    return Err(String::from("Invalid IP address"));
                }

                Ok(())
            }
        }
        _ => Err(format!("Invalid scheme: {}", upstream_url)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_upstream_url() {
        assert!(validate_upstream_url("https://google.com").is_ok());
        assert!(validate_upstream_url("http://google.com").is_ok());
        assert!(validate_upstream_url("http://10.0.0.1").is_err());
        assert!(validate_upstream_url("http://172.16.0.1").is_err());
        assert!(validate_upstream_url("http://169.254.1.1").is_err());
        assert!(validate_upstream_url("http://LOCALHOST").is_err());
        assert!(validate_upstream_url("http://test.localhost").is_err());
        assert!(validate_upstream_url("ftp://google.com").is_err());
    }

    #[test]
    fn test_validate_path() {
        assert!(validate_path("/").is_ok());
        assert!(validate_path("/abc").is_ok());
        assert!(validate_path("").is_err());
        assert!(validate_path("no-slash").is_err());
        assert!(validate_path("/a/../b").is_err());
    }
}
