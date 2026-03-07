use std::time::Duration;

use anyhow::Result;
use reqwest::Client;

use crate::{settings, types::Settings};

const DEFAULT_USER_AGENT: &str = "Fred TV";

/// Resolve the effective user-agent string from global settings, with an
/// optional per-source override.
pub fn resolve_user_agent(settings: &Settings, source_ua: Option<&str>) -> String {
    source_ua
        .filter(|s| !s.trim().is_empty())
        .or(settings.user_agent.as_deref())
        .unwrap_or(DEFAULT_USER_AGENT)
        .to_string()
}

/// Build a base reqwest client with global settings applied (proxy, timeout).
/// Does NOT set a user-agent; call `resolve_user_agent` separately and pass it
/// to `.user_agent()` on the builder if needed.
fn base_builder(settings: &Settings) -> Result<reqwest::ClientBuilder> {
    let mut builder = Client::builder();

    if let Some(proxy_url) = &settings.proxy {
        if !proxy_url.trim().is_empty() {
            let proxy = reqwest::Proxy::all(proxy_url)?;
            builder = builder.proxy(proxy);
        }
    }

    if let Some(timeout_secs) = settings.connection_timeout {
        if timeout_secs > 0 {
            let dur = Duration::from_secs(timeout_secs as u64);
            builder = builder.timeout(dur).connect_timeout(dur);
        }
    }

    Ok(builder)
}

/// Build a plain client with only global settings (UA, proxy, timeout).
/// Used for simple one-off requests like WAN IP lookup.
pub fn build_client() -> Result<Client> {
    let settings = settings::get_settings()?;
    let ua = resolve_user_agent(&settings, None);
    Ok(base_builder(&settings)?.user_agent(ua).build()?)
}

/// Build a client configured for a specific source.
/// Applies: source UA > global UA > default, plus proxy and timeout.
pub fn build_client_for_source(source_ua: Option<&str>) -> Result<Client> {
    let settings = settings::get_settings()?;
    let ua = resolve_user_agent(&settings, source_ua);
    Ok(base_builder(&settings)?.user_agent(ua).build()?)
}

/// Build a client configured for a specific channel with full header support.
/// UA priority: channel headers > source stream_user_agent > source UA > global > default.
/// Also applies Origin, Referer, and ignore-SSL from channel headers.
pub fn build_client_for_channel(
    source: &crate::types::Source,
    headers: Option<&crate::types::ChannelHttpHeaders>,
) -> Result<Client> {
    let settings = settings::get_settings()?;

    // UA priority chain
    let channel_ua = headers.and_then(|h| h.user_agent.as_deref());
    let source_stream_ua = source.stream_user_agent.as_deref();
    let source_ua = source.user_agent.as_deref();
    let ua = resolve_user_agent(
        &settings,
        channel_ua
            .or(source_stream_ua)
            .or(source_ua),
    );

    let mut builder = base_builder(&settings)?.user_agent(ua);

    // Custom headers
    let mut headers_map = reqwest::header::HeaderMap::new();
    if let Some(h) = headers {
        if let Some(origin) = h.http_origin.as_ref() {
            headers_map.insert("Origin", reqwest::header::HeaderValue::from_str(origin)?);
        }
        if let Some(referrer) = h.referrer.as_ref() {
            headers_map.insert("Referer", reqwest::header::HeaderValue::from_str(referrer)?);
        }
        if let Some(true) = h.ignore_ssl {
            builder = builder.danger_accept_invalid_certs(true);
        }
    }

    Ok(builder.default_headers(headers_map).build()?)
}
