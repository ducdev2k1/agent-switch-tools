// OAuth token refresh for Antigravity saved profiles.
//
// Background: Antigravity stores its OAuth credentials in `state.vscdb` under two keys:
//   1. `antigravityAuthStatus` — JSON with `apiKey` (short-lived access token, ~1h TTL)
//   2. `antigravityUnifiedStateSync.oauthToken` — nested protobuf containing access_token,
//      refresh_token, and expiry timestamp
//
// When the IDE is running it auto-refreshes #1. Saved profiles (switched-away accounts)
// freeze at whatever `apiKey` was present at save time → quota API returns 401 after ~1h.
//
// Solution: parse the protobuf blob to extract refresh_token, then exchange it for a fresh
// access_token via Google's OAuth endpoint. Client ID/secret are the same values Antigravity
// itself uses (reverse-engineered from the extension bundle, consistent with Antigravity-Manager).

use serde::Deserialize;

// Antigravity Enterprise OAuth client credentials.
//
// These are the IDE's embedded public OAuth client (same as
// github.com/lbjlaq/Antigravity-Manager). They are NOT user secrets — Google OAuth for
// installed/desktop applications requires embedding client_id + client_secret in the
// binary. See https://developers.google.com/identity/protocols/oauth2/native-app
//
// We load them via `ANTIGRAVITY_OAUTH_CLIENT_ID` and `ANTIGRAVITY_OAUTH_CLIENT_SECRET`
// env vars at compile time (CI provides them from GitHub Secrets). Local developers can
// also `export` these, or extract from an Antigravity install (search the extension
// bundle for `apps.googleusercontent.com`).
const CLIENT_ID: &str = match option_env!("ANTIGRAVITY_OAUTH_CLIENT_ID") {
    Some(v) => v,
    None => "",
};
const CLIENT_SECRET: &str = match option_env!("ANTIGRAVITY_OAUTH_CLIENT_SECRET") {
    Some(v) => v,
    None => "",
};
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Clone)]
pub struct OAuthTokenInfo {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at_unix: Option<u64>,
}

/// Parse the double-wrapped protobuf blob at `antigravityUnifiedStateSync.oauthToken`.
///
/// Wire format (discovered empirically):
///   outer message {
///     field 1 (message) {
///       field 1 (string) = "oauthTokenInfoSentinelKey"
///       field 2 (message) {
///         field 1 (string) = base64-encoded inner payload
///       }
///     }
///   }
///   inner payload (after base64 decode) {
///     field 1 (string) = access_token (ya29.*)
///     field 2 (string) = "Bearer"
///     field 3 (string) = refresh_token (1//*)
///     field 4 (message) { field 1 (varint) = expiry_unix_seconds }
///   }
pub fn parse_oauth_token_blob(base64_blob: &str) -> Option<OAuthTokenInfo> {
    let outer_bytes = data_encoding::BASE64.decode(base64_blob.trim().as_bytes()).ok()?;

    // Navigate outer → f1 msg → f2 msg → f1 string (= inner base64)
    let inner_b64 = extract_inner_base64(&outer_bytes)?;
    let inner_bytes = data_encoding::BASE64.decode(inner_b64.as_bytes()).ok()?;

    // Parse inner fields
    let mut access: Option<String> = None;
    let mut refresh: Option<String> = None;
    let mut expires_at: Option<u64> = None;

    let mut pos = 0usize;
    while pos < inner_bytes.len() {
        let (tag, new_pos) = read_varint(&inner_bytes, pos)?;
        pos = new_pos;
        let field = tag >> 3;
        let wire = tag & 7;
        match (field, wire) {
            (1, 2) => {
                let (s, p) = read_length_delimited(&inner_bytes, pos)?;
                access = Some(std::str::from_utf8(s).ok()?.to_string());
                pos = p;
            }
            (3, 2) => {
                let (s, p) = read_length_delimited(&inner_bytes, pos)?;
                refresh = Some(std::str::from_utf8(s).ok()?.to_string());
                pos = p;
            }
            (4, 2) => {
                let (msg, p) = read_length_delimited(&inner_bytes, pos)?;
                // field 1 (varint) inside expiry message
                let (inner_tag, ip) = read_varint(msg, 0)?;
                if inner_tag >> 3 == 1 && inner_tag & 7 == 0 {
                    let (v, _) = read_varint(msg, ip)?;
                    expires_at = Some(v as u64);
                }
                pos = p;
            }
            (_, 0) => {
                let (_, p) = read_varint(&inner_bytes, pos)?;
                pos = p;
            }
            (_, 2) => {
                let (_, p) = read_length_delimited(&inner_bytes, pos)?;
                pos = p;
            }
            _ => return None,
        }
    }

    Some(OAuthTokenInfo {
        access_token: access?,
        refresh_token: refresh?,
        expires_at_unix: expires_at,
    })
}

fn extract_inner_base64(buf: &[u8]) -> Option<String> {
    // f1 msg
    let (tag, pos) = read_varint(buf, 0)?;
    if tag != (1 << 3 | 2) {
        return None;
    }
    let (outer_msg, _) = read_length_delimited(buf, pos)?;

    // Within outer_msg: skip f1 string, take f2 msg
    let mut p = 0usize;
    let (t1, p1) = read_varint(outer_msg, p)?;
    if t1 & 7 != 2 {
        return None;
    }
    let (_, p2) = read_length_delimited(outer_msg, p1)?;
    p = p2;

    let (t2, p3) = read_varint(outer_msg, p)?;
    if t2 >> 3 != 2 || t2 & 7 != 2 {
        return None;
    }
    let (f2_msg, _) = read_length_delimited(outer_msg, p3)?;

    // Within f2_msg: f1 string (the base64 payload)
    let (t3, p4) = read_varint(f2_msg, 0)?;
    if t3 != (1 << 3 | 2) {
        return None;
    }
    let (payload_bytes, _) = read_length_delimited(f2_msg, p4)?;
    std::str::from_utf8(payload_bytes).ok().map(String::from)
}

fn read_varint(buf: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    loop {
        if pos >= buf.len() {
            return None;
        }
        let b = buf[pos];
        pos += 1;
        result |= ((b & 0x7f) as u64) << shift;
        if b & 0x80 == 0 {
            return Some((result, pos));
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
}

fn read_length_delimited(buf: &[u8], pos: usize) -> Option<(&[u8], usize)> {
    let (len, new_pos) = read_varint(buf, pos)?;
    let end = new_pos.checked_add(len as usize)?;
    if end > buf.len() {
        return None;
    }
    Some((&buf[new_pos..end], end))
}

// ========== Refresh Flow ==========

#[derive(Debug, Deserialize)]
struct RefreshResponse {
    access_token: String,
    expires_in: u64,
}

/// Exchange a refresh_token for a fresh access_token via Google OAuth.
pub async fn refresh_access_token(refresh_token: &str) -> Result<(String, u64), String> {
    if CLIENT_ID.is_empty() || CLIENT_SECRET.is_empty() {
        return Err(
            "OAuth client not configured. Set ANTIGRAVITY_OAUTH_CLIENT_ID and \
             ANTIGRAVITY_OAUTH_CLIENT_SECRET env vars at build time.".into(),
        );
    }
    let client = &crate::modules::shared::http::CLIENT;
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];
    let res = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("OAuth refresh request failed: {}", e))?;

    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(format!("OAuth refresh HTTP {}: {}", status, body));
    }

    let parsed: RefreshResponse = res
        .json()
        .await
        .map_err(|e| format!("OAuth refresh JSON parse: {}", e))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok((parsed.access_token, now + parsed.expires_in))
}

/// Given auth-keys (from saved profile or live vscdb), return a non-expired access token.
/// Strategy:
///   1. Try `antigravityAuthStatus.apiKey` first (fast path — valid while IDE-managed).
///   2. If caller's test request fails OR expiry is near, parse oauthToken blob and refresh.
///
/// Returns (access_token, refresh_token_opt). refresh_token is returned so caller can
/// persist updates if a refresh happened.
pub async fn get_fresh_access_token(
    auth_data: &std::collections::HashMap<String, String>,
) -> Option<(String, Option<String>)> {
    // Fast path: trust apiKey (IDE-refreshed while running)
    let current_apikey = auth_data
        .get("antigravityAuthStatus")
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("apiKey").and_then(|x| x.as_str()).map(String::from));

    // If we have the protobuf blob, extract refresh_token for potential renewal
    let token_info = auth_data
        .get("antigravityUnifiedStateSync.oauthToken")
        .and_then(|blob| parse_oauth_token_blob(blob));

    // Decide: use apiKey if still valid (best-effort — we don't know actual expiry of apiKey)
    // or refresh if we have refresh_token and token_info says expired
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let is_expired = token_info
        .as_ref()
        .and_then(|t| t.expires_at_unix)
        .map(|exp| now >= exp.saturating_sub(60)) // 60s skew
        .unwrap_or(false);

    if !is_expired {
        if let Some(k) = current_apikey {
            return Some((k, token_info.map(|t| t.refresh_token)));
        }
    }

    // Need refresh
    let refresh = token_info.as_ref().map(|t| t.refresh_token.clone())?;
    match refresh_access_token(&refresh).await {
        Ok((new_access, _exp)) => Some((new_access, Some(refresh))),
        Err(e) => {
            eprintln!("[antigravity] refresh failed: {}", e);
            // Last resort: return apiKey even if expired (caller may 401)
            current_apikey.map(|k| (k, Some(refresh)))
        }
    }
}
