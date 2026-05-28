use std::{
    net::{IpAddr, ToSocketAddrs},
    str::FromStr,
};

use url::Url;

use crate::error::{AppError, AppResult};

pub fn validate_url(raw: &str) -> AppResult<Url> {
    let url = Url::parse(raw).map_err(|_| AppError::BadRequest("invalid URL".to_string()))?;
    validate_parsed_url(&url)?;
    Ok(url)
}

pub fn validate_parsed_url(url: &Url) -> AppResult<()> {
    match url.scheme() {
        "http" | "https" => {}
        _ => {
            return Err(AppError::BadRequest(
                "only http and https URLs are allowed".to_string(),
            ))
        }
    }
    let host = url
        .host_str()
        .ok_or_else(|| AppError::BadRequest("URL host is required".to_string()))?;
    if host.eq_ignore_ascii_case("localhost") {
        return Err(AppError::BadRequest("localhost is not allowed".to_string()));
    }
    if let Ok(ip) = IpAddr::from_str(host.trim_matches(['[', ']'])) {
        return validate_ip(ip);
    }
    Ok(())
}

pub async fn validate_resolved_url(url: &Url) -> AppResult<()> {
    validate_parsed_url(url)?;
    let host = url
        .host_str()
        .ok_or_else(|| AppError::BadRequest("URL host is required".to_string()))?
        .to_string();
    let port = url
        .port_or_known_default()
        .ok_or_else(|| AppError::BadRequest("URL port is required".to_string()))?;
    let addrs = tokio::task::spawn_blocking(move || (host.as_str(), port).to_socket_addrs())
        .await
        .map_err(|_| AppError::Fetch("DNS lookup failed".to_string()))?
        .map_err(|_| AppError::Fetch("DNS lookup failed".to_string()))?;
    let mut found = false;
    for addr in addrs {
        found = true;
        validate_ip(addr.ip())?;
    }
    if !found {
        return Err(AppError::Fetch(
            "DNS lookup returned no addresses".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_ip(ip: IpAddr) -> AppResult<()> {
    if is_blocked_ip(ip) {
        return Err(AppError::BadRequest(format!("blocked IP address: {ip}")));
    }
    Ok(())
}

pub fn is_blocked_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_broadcast()
                || ip.is_multicast()
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || (ip.segments()[0] & 0xfe00) == 0xfc00
                || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_disallowed_schemes_and_local_hosts() {
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("http://localhost").is_err());
        assert!(validate_url("http://127.0.0.1").is_err());
        assert!(validate_url("http://[::1]").is_err());
        assert!(validate_url("https://example.com/path").is_ok());
    }

    #[test]
    fn blocks_private_and_link_local_ips() {
        assert!(is_blocked_ip("10.0.0.1".parse().unwrap()));
        assert!(is_blocked_ip("172.16.0.1".parse().unwrap()));
        assert!(is_blocked_ip("192.168.1.1".parse().unwrap()));
        assert!(is_blocked_ip("169.254.1.1".parse().unwrap()));
        assert!(is_blocked_ip("fc00::1".parse().unwrap()));
        assert!(is_blocked_ip("fe80::1".parse().unwrap()));
        assert!(!is_blocked_ip("93.184.216.34".parse().unwrap()));
    }
}
