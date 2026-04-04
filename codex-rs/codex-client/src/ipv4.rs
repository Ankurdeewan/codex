use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;

const CODEX_FORCE_IPV4_ENV: &str = "CODEX_FORCE_IPV4";

/// Returns `true` when the user has opted into forcing IPv4 connections.
///
/// On Windows systems without working IPv6, the OS may spend a long time
/// attempting IPv6 connections before falling back to IPv4, causing noticeable
/// timeouts. Setting `CODEX_FORCE_IPV4=1` skips IPv6 entirely.
pub fn should_force_ipv4() -> bool {
    matches!(
        std::env::var(CODEX_FORCE_IPV4_ENV).as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

/// Resolves the given host and port to an IPv4 socket address. Returns an
/// error when no IPv4 address could be found.
pub async fn resolve_ipv4(host: &str, port: u16) -> std::io::Result<SocketAddr> {
    use tokio::net::lookup_host;

    let addrs = lookup_host((host, port)).await?;
    for addr in addrs {
        if addr.is_ipv4() {
            return Ok(addr);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrNotAvailable,
        format!("no IPv4 address found for {host}:{port}"),
    ))
}

/// Applies the IPv4-only local address binding to a reqwest client builder
/// when `CODEX_FORCE_IPV4` is enabled.
pub fn apply_ipv4_if_forced(builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    if should_force_ipv4() {
        tracing::info!("CODEX_FORCE_IPV4 is set; restricting HTTP client to IPv4");
        builder.local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
    } else {
        builder
    }
}
