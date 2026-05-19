//! Media unlock checker — tests whether proxies can access geo-restricted streaming services.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// A single media unlock test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockResult {
    pub service: String,
    pub icon: String,
    pub status: UnlockStatus,
    pub region: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnlockStatus {
    Unlocked,
    Locked,
    Failed,
    Checking,
}

impl UnlockStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unlocked => "YES",
            Self::Locked => "NO",
            Self::Failed => "ERR",
            Self::Checking => "...",
        }
    }
}

/// Known media unlock test definitions.
const CHECKERS: &[(&str, &str, &str)] = &[
    ("Netflix", "🎬", "https://www.netflix.com/title/70143836"),
    ("YouTube", "▶️", "https://www.youtube.com/premium"),
    ("Disney+", "🐭", "https://www.disneyplus.com/"),
    ("Spotify", "🎵", "https://www.spotify.com/"),
    ("Bilibili", "📺", "https://www.bilibili.com/"),
    ("DMM", "🎮", "https://www.dmm.co.jp/"),
    ("Abema", "📡", "https://abema.tv/"),
    ("Bahamut", "🐉", "https://ani.gamer.com.tw/"),
    ("HBO Max", "📼", "https://www.hbomax.com/"),
];

/// Run all media unlock checks via the given proxy URL.
pub async fn check_all(proxy_url: &str) -> Vec<UnlockResult> {
    let client = match reqwest::Client::builder()
        .proxy(
            reqwest::Proxy::all(proxy_url)
                .unwrap_or_else(|_| reqwest::Proxy::http("http://127.0.0.1:7899").unwrap()),
        )
        .timeout(std::time::Duration::from_secs(5))
        .danger_accept_invalid_certs(true)
        .build()
    {
        Ok(c) => c,
        Err(_) => {
            return CHECKERS
                .iter()
                .map(|(name, icon, _)| UnlockResult {
                    service: name.to_string(),
                    icon: icon.to_string(),
                    status: UnlockStatus::Failed,
                    region: None,
                })
                .collect();
        }
    };

    let mut results = Vec::new();
    for (name, icon, url) in CHECKERS {
        match check_single(&client, url).await {
            Ok(Some(region)) => results.push(UnlockResult {
                service: name.to_string(),
                icon: icon.to_string(),
                status: UnlockStatus::Unlocked,
                region: Some(region),
            }),
            Ok(None) => results.push(UnlockResult {
                service: name.to_string(),
                icon: icon.to_string(),
                status: UnlockStatus::Locked,
                region: None,
            }),
            Err(_) => results.push(UnlockResult {
                service: name.to_string(),
                icon: icon.to_string(),
                status: UnlockStatus::Failed,
                region: None,
            }),
        }
    }

    results
}

async fn check_single(client: &reqwest::Client, url: &str) -> Result<Option<String>> {
    let resp = client.get(url).send().await?;
    let status = resp.status().as_u16();

    match status {
        // 200: accessible
        200 => Ok(None),
        // 301/302: redirect — extract location for region info
        301 | 302 => {
            let location = resp
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();

            let region = if location.contains("netflix.com") {
                Some("Global".into())
            } else if let Some(pos) = location.find("//") {
                let rest = &location[pos + 2..];
                let tld = rest.split('/').next().unwrap_or("");
                // Extract region from TLD like "netflix.com/jp" -> "JP"
                tld.split('/')
                    .nth(1)
                    .map(|s| s.to_uppercase())
                    .or_else(|| tld.rsplit('.').next().map(|s| s.to_uppercase()))
            } else {
                None
            };

            Ok(region.or(Some("Redirect".into())))
        }
        // 403/451: geo-restricted
        403 | 451 => Err(anyhow::anyhow!("geo-restricted")),
        // Other: might be accessible
        _ => Ok(None),
    }
}
