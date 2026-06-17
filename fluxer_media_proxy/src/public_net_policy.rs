// SPDX-License-Identifier: AGPL-3.0-or-later

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use thiserror::Error;
use tokio::net::lookup_host;

const MAX_URL_LEN: usize = 8192;

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum Error {
    #[error("invalid URL")]
    InvalidUrl,
    #[error("blocked URL")]
    BlockedUrl,
    #[error("DNS lookup failed")]
    DnsLookupFailed,
    #[error("host resolved to no address")]
    HostResolvedToNoAddress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsedUrl<'a> {
    pub scheme: &'a str,
    pub authority: &'a str,
    pub host: &'a str,
    pub path_query: &'a str,
}

fn contains_ctl(value: &str) -> bool {
    value.bytes().any(|ch| ch < 0x20 || ch == 0x7f)
}

pub fn parse_url(url: &str) -> Result<ParsedUrl<'_>, Error> {
    if url.is_empty() || url.len() > MAX_URL_LEN || contains_ctl(url) {
        return Err(Error::InvalidUrl);
    }
    let scheme_end = url.find("://").ok_or(Error::InvalidUrl)?;
    let scheme = &url[..scheme_end];
    if !(scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")) {
        return Err(Error::BlockedUrl);
    }
    let mut rest = &url[scheme_end + 3..];
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    if authority_end == 0 {
        return Err(Error::InvalidUrl);
    }
    let authority = &rest[..authority_end];
    if authority.contains('@') {
        return Err(Error::BlockedUrl);
    }
    rest = &rest[authority_end..];
    let host = if let Some(after_bracket) = authority.strip_prefix('[') {
        let close = after_bracket.find(']').ok_or(Error::InvalidUrl)?;
        let host = &after_bracket[..close];
        let suffix = &after_bracket[close + 1..];
        if !suffix.is_empty() {
            let port = suffix.strip_prefix(':').ok_or(Error::InvalidUrl)?;
            validate_port(port)?;
        }
        host
    } else if let Some(colon) = authority.find(':') {
        if authority[colon + 1..].contains(':') {
            return Err(Error::InvalidUrl);
        }
        validate_port(&authority[colon + 1..])?;
        &authority[..colon]
    } else {
        authority
    };
    if host.is_empty() {
        return Err(Error::InvalidUrl);
    }
    let path_query = if rest.is_empty() {
        "/"
    } else {
        &rest[..rest.find('#').unwrap_or(rest.len())]
    };
    Ok(ParsedUrl {
        scheme,
        authority,
        host,
        path_query,
    })
}

fn validate_port(raw: &str) -> Result<(), Error> {
    if raw.is_empty() {
        return Err(Error::InvalidUrl);
    }
    let port = raw.parse::<u16>().map_err(|_| Error::InvalidUrl)?;
    if port == 0 {
        return Err(Error::InvalidUrl);
    }
    Ok(())
}

fn normalize_host(raw: &str) -> Result<String, Error> {
    let trimmed = raw.trim_matches([' ', '\t', '\r', '\n']);
    let without_dot = trimmed.strip_suffix('.').unwrap_or(trimmed);
    if without_dot.is_empty() {
        return Err(Error::InvalidUrl);
    }
    Ok(without_dot.to_ascii_lowercase())
}

fn ipv4_to_u32(ip: Ipv4Addr) -> u32 {
    u32::from_be_bytes(ip.octets())
}

fn ipv4_in(ip: Ipv4Addr, prefix: Ipv4Addr, bits: u32) -> bool {
    let mask = if bits == 0 {
        0
    } else {
        u32::MAX << (32 - bits)
    };
    (ipv4_to_u32(ip) & mask) == (ipv4_to_u32(prefix) & mask)
}

fn ipv6_to_u128(ip: Ipv6Addr) -> u128 {
    u128::from_be_bytes(ip.octets())
}

fn ipv6_in(ip: Ipv6Addr, prefix: Ipv6Addr, bits: u32) -> bool {
    let mask = if bits == 0 {
        0
    } else {
        u128::MAX << (128 - bits)
    };
    (ipv6_to_u128(ip) & mask) == (ipv6_to_u128(prefix) & mask)
}

fn blocked_ipv4(ip: Ipv4Addr) -> bool {
    ipv4_in(ip, Ipv4Addr::new(0, 0, 0, 0), 8)
        || ipv4_in(ip, Ipv4Addr::new(10, 0, 0, 0), 8)
        || ipv4_in(ip, Ipv4Addr::new(100, 64, 0, 0), 10)
        || ipv4_in(ip, Ipv4Addr::new(127, 0, 0, 0), 8)
        || ipv4_in(ip, Ipv4Addr::new(169, 254, 0, 0), 16)
        || ipv4_in(ip, Ipv4Addr::new(172, 16, 0, 0), 12)
        || ipv4_in(ip, Ipv4Addr::new(192, 0, 0, 0), 24)
        || ipv4_in(ip, Ipv4Addr::new(192, 0, 2, 0), 24)
        || ipv4_in(ip, Ipv4Addr::new(192, 88, 99, 0), 24)
        || ipv4_in(ip, Ipv4Addr::new(192, 168, 0, 0), 16)
        || ipv4_in(ip, Ipv4Addr::new(198, 18, 0, 0), 15)
        || ipv4_in(ip, Ipv4Addr::new(198, 51, 100, 0), 24)
        || ipv4_in(ip, Ipv4Addr::new(203, 0, 113, 0), 24)
        || ipv4_in(ip, Ipv4Addr::new(224, 0, 0, 0), 4)
        || ipv4_in(ip, Ipv4Addr::new(240, 0, 0, 0), 4)
        || ip == Ipv4Addr::new(255, 255, 255, 255)
}

fn embedded_ipv4(ip: Ipv6Addr) -> Option<Ipv4Addr> {
    let octets = ip.octets();
    if ipv6_in(ip, Ipv6Addr::new(0x0064, 0xff9b, 0, 0, 0, 0, 0, 0), 96)
        || ipv6_in(ip, Ipv6Addr::UNSPECIFIED, 96)
    {
        return Some(Ipv4Addr::new(
            octets[12], octets[13], octets[14], octets[15],
        ));
    }
    if ipv6_in(ip, Ipv6Addr::new(0x2002, 0, 0, 0, 0, 0, 0, 0), 16) {
        return Some(Ipv4Addr::new(octets[2], octets[3], octets[4], octets[5]));
    }
    None
}

fn blocked_ipv6(ip: Ipv6Addr) -> bool {
    if let Some(mapped) = ip.to_ipv4_mapped() {
        return blocked_ipv4(mapped);
    }
    if let Some(embedded) = embedded_ipv4(ip) {
        return blocked_ipv4(embedded);
    }
    ip.is_unspecified()
        || ip.is_loopback()
        || ipv6_in(ip, Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0), 32)
        || ipv6_in(ip, Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7)
        || ipv6_in(ip, Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10)
        || ipv6_in(ip, Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0), 8)
}

pub fn is_blocked_ip_literal(raw: &str) -> bool {
    match raw.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) => blocked_ipv4(ip),
        Ok(IpAddr::V6(ip)) => blocked_ipv6(ip),
        Err(_) => true,
    }
}

fn validate_resolved_ip(ip: IpAddr) -> Result<(), Error> {
    match ip {
        IpAddr::V4(ip) if blocked_ipv4(ip) => Err(Error::BlockedUrl),
        IpAddr::V6(ip) if blocked_ipv6(ip) => Err(Error::BlockedUrl),
        _ => Ok(()),
    }
}

pub struct PinnedDnsResolver;

impl Resolve for PinnedDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            let host = name.as_str().to_owned();
            let resolved: Vec<SocketAddr> = lookup_host((host.as_str(), 0)).await?.collect();
            if resolved.is_empty() {
                return Err(Box::new(Error::HostResolvedToNoAddress)
                    as Box<dyn std::error::Error + Send + Sync>);
            }
            for addr in &resolved {
                validate_resolved_ip(addr.ip())?;
            }
            Ok(Box::new(resolved.into_iter()) as Addrs)
        })
    }
}

pub fn is_valid_public_hostname(host: &str) -> bool {
    if host.is_empty() || host.len() > 253 || !host.contains('.') {
        return false;
    }
    let mut last = "";
    for label in host.split('.') {
        if label.is_empty() || label.len() > 63 {
            return false;
        }
        let bytes = label.as_bytes();
        if !bytes[0].is_ascii_alphanumeric() || !bytes[bytes.len() - 1].is_ascii_alphanumeric() {
            return false;
        }
        if !bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'-')
        {
            return false;
        }
        last = label;
    }
    !last.bytes().all(|b| b.is_ascii_digit())
}

pub fn validate_url(url: &str) -> Result<(), Error> {
    let parsed = parse_url(url)?;
    let host = normalize_host(parsed.host)?;
    if let Ok(ip) = host.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(ip) if blocked_ipv4(ip) => Err(Error::BlockedUrl),
            IpAddr::V6(ip) if blocked_ipv6(ip) => Err(Error::BlockedUrl),
            _ => Ok(()),
        };
    }
    if !is_valid_public_hostname(&host) {
        return Err(Error::BlockedUrl);
    }
    let mut seen = false;
    for addr in (host.as_str(), 80)
        .to_socket_addrs()
        .map_err(|_| Error::DnsLookupFailed)?
    {
        seen = true;
        match addr.ip() {
            IpAddr::V4(ip) if blocked_ipv4(ip) => return Err(Error::BlockedUrl),
            IpAddr::V6(ip) if blocked_ipv6(ip) => return Err(Error::BlockedUrl),
            _ => {}
        }
    }
    if seen {
        Ok(())
    } else {
        Err(Error::HostResolvedToNoAddress)
    }
}

pub fn resolve_redirect(base_url: &str, location: &str) -> Result<String, Error> {
    if location.is_empty() || location.len() > MAX_URL_LEN || contains_ctl(location) {
        return Err(Error::InvalidUrl);
    }
    let fragment = location.find('#').unwrap_or(location.len());
    let loc = location[..fragment].trim_matches([' ', '\t', '\r', '\n']);
    if loc
        .get(..7)
        .is_some_and(|s| s.eq_ignore_ascii_case("http://"))
        || loc
            .get(..8)
            .is_some_and(|s| s.eq_ignore_ascii_case("https://"))
    {
        return Ok(loc.to_owned());
    }
    let base = parse_url(base_url)?;
    if loc.starts_with("//") {
        return Ok(format!("{}:{loc}", base.scheme));
    }
    if loc.starts_with('/') {
        return Ok(format!("{}://{}{}", base.scheme, base.authority, loc));
    }
    let q = base.path_query.find('?').unwrap_or(base.path_query.len());
    let base_path = &base.path_query[..q];
    if loc.starts_with('?') {
        return Ok(format!(
            "{}://{}{}{}",
            base.scheme, base.authority, base_path, loc
        ));
    }
    let slash = base_path.rfind('/').unwrap_or(0);
    let prefix = if slash == 0 {
        "/"
    } else {
        &base_path[..slash + 1]
    };
    let joined = format!("{prefix}{loc}");
    Ok(format!(
        "{}://{}{}",
        base.scheme,
        base.authority,
        remove_dot_segments(&joined)
    ))
}

fn remove_dot_segments(path_query: &str) -> String {
    let q = path_query.find('?').unwrap_or(path_query.len());
    let path = &path_query[..q];
    let query = &path_query[q..];
    let mut segments = Vec::new();
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            segments.pop();
        } else {
            segments.push(segment);
        }
    }
    let mut out = String::from("/");
    out.push_str(&segments.join("/"));
    if path.len() > 1 && path.ends_with('/') && !out.ends_with('/') {
        out.push('/');
    }
    out.push_str(query);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_private_and_special_ip_literals() {
        assert!(is_blocked_ip_literal("127.0.0.1"));
        assert!(is_blocked_ip_literal("10.1.2.3"));
        assert!(is_blocked_ip_literal("::1"));
        assert!(is_blocked_ip_literal("::ffff:192.168.1.1"));
        assert!(!is_blocked_ip_literal("8.8.8.8"));
        assert!(!is_blocked_ip_literal("2606:4700:4700::1111"));
    }

    #[test]
    fn validates_public_host_syntax() {
        assert!(is_valid_public_hostname("example.com"));
        assert!(is_valid_public_hostname("xn--bcher-kva.example"));
        assert!(!is_valid_public_hostname("localhost"));
        assert!(!is_valid_public_hostname("example"));
        assert!(!is_valid_public_hostname("bad_name.example"));
        assert!(!is_valid_public_hostname("example.123"));
    }

    #[test]
    fn resolves_relative_redirects() {
        assert_eq!(
            "https://example.com/a/d?y=2",
            resolve_redirect("https://example.com/a/b/c?x=1", "../d?y=2#ignored").unwrap()
        );
        assert_eq!(
            "https://example.com/z",
            resolve_redirect("https://example.com/a/b/c", "/z").unwrap()
        );
    }

    #[test]
    fn blocks_every_documented_ipv4_ssrf_range() {
        for ip in [
            "0.0.0.0",
            "10.0.0.1",
            "100.64.0.1",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.5.5",
            "192.0.0.1",
            "192.0.2.5",
            "192.88.99.5",
            "192.168.1.1",
            "198.18.0.1",
            "198.51.100.5",
            "203.0.113.5",
            "224.0.0.1",
            "240.0.0.1",
            "255.255.255.255",
        ] {
            assert!(is_blocked_ip_literal(ip), "{ip}");
        }
    }

    #[test]
    fn blocks_every_documented_ipv6_ssrf_range() {
        for ip in [
            "::",
            "::1",
            "2001:db8::1",
            "fc00::1",
            "fd00::1",
            "fe80::1",
            "ff00::1",
            "64:ff9b::a9fe:a9fe",
            "::7f00:1",
            "2002:a9fe:a9fe::",
        ] {
            assert!(is_blocked_ip_literal(ip), "{ip}");
        }
    }

    #[test]
    fn rejects_urls_with_userinfo() {
        assert_eq!(
            Err(Error::BlockedUrl),
            validate_url("https://user:pass@example.com/")
        );
    }

    #[test]
    fn rejects_non_http_s_schemes() {
        assert_eq!(Err(Error::BlockedUrl), validate_url("file:///etc/passwd"));
        assert_eq!(
            Err(Error::BlockedUrl),
            validate_url("gopher://example.com/")
        );
    }

    #[test]
    fn redirect_that_returns_to_same_url_is_allowed() {
        assert_eq!(
            "https://example.com/path",
            resolve_redirect("https://example.com/path", "/path").unwrap()
        );
    }

    #[test]
    fn redirect_with_dot_dot_cannot_escape_host() {
        let r = resolve_redirect("https://example.com/a", "../../../etc").unwrap();
        assert!(r.starts_with("https://example.com/"));
    }
}
