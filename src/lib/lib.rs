pub mod mtproxy;
pub mod shadowsocks;
pub mod vmess;
pub mod utils;

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[cfg(feature = "scraper")]

pub trait Proxy {}
#[derive(Debug, Serialize, Deserialize)]
pub struct VLess {
    pub host: String,
    pub port: u32,
    pub id: String,
    #[serde(flatten)]
    pub parameters: Option<HashMap<String, String>>,
}

impl VLess {
    pub fn to_url(&self) -> String {
        let url_encoded_parameters = serde_urlencoded::to_string(&self.parameters).unwrap();
        format!(
            "vless://{}@{}:{}?{}",
            self.id, self.host, self.port, url_encoded_parameters
        )
    }

    pub fn scrape(source: &str) -> Vec<Self> {
        Scraper::scrape_vless(source)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trojan {
    pub host: String,
    pub port: u32,
    pub password: String,
    #[serde(flatten)]
    pub parameters: Option<HashMap<String, String>>,
}

impl Trojan {
    pub fn to_url(&self) -> String {
        let url_encoded_parameters = serde_urlencoded::to_string(&self.parameters).unwrap();
        format!(
            "trojan://{}@{}:{}?{}",
            self.password, self.host, self.port, url_encoded_parameters
        )
    }

    pub fn scrape(source: &str) -> Vec<Self> {
        Scraper::scrape_trojan(source)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hysteria {
    pub version: u8,
    pub host: String,
    pub port: u32,
    pub auth: String,
    #[serde(flatten)]
    pub parameters: Option<HashMap<String, String>>,
}

impl Hysteria {
    pub fn to_url(&self) -> String {
        let url_encoded_parameters = serde_urlencoded::to_string(&self.parameters).unwrap();
        let hysteria_version = match self.version {
            1 => "hysteria",
            _ => "hy2",
        };

        format!(
            "{}://{}@{}:{}?{}",
            hysteria_version, self.auth, self.host, self.port, url_encoded_parameters
        )
    }

    pub fn scrape(source: &str) -> Vec<Self> {
        Scraper::scrape_hysteria(source)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TUIC {
    pub host: String,
    pub port: u32,
    pub auth: String,
    #[serde(flatten)]
    pub parameters: Option<HashMap<String, String>>,
}

impl TUIC {
    pub fn to_url(&self) -> String {
        let url_encoded_parameters = serde_urlencoded::to_string(&self.parameters).unwrap();
        format!(
            "tuic://{}@{}:{}?{}",
            self.auth, self.host, self.port, url_encoded_parameters
        )
    }

    pub fn scrape(source: &str) -> Vec<Self> {
        Scraper::scrape_tuic(source)
    }
}

/// A scraper for extracting MTProxy information from a given source string.
pub struct Scraper();

impl Scraper {
    fn seperate_links(text: &str) -> String {
        let regex = Regex::new(
            r#"\b(https|ss|vmess|vless|trojan|hysteria2|hy2|hysteria)?://[^\s<>"']+[^.,;!?)"'\s]"#,
        )
        .unwrap();
        let mut links = String::new();
        for cap in regex.captures_iter(text) {
            links.push_str(&cap[0].replace("&amp;amp;", "&").replace("%3D", "="));
            links.push('\n');
        }
        links
    }

    pub fn scrape_vless(source: &str) -> Vec<VLess> {
        let source = &Self::seperate_links(source);
        let mut proxy_list: Vec<VLess> = Vec::new();
        let regex = Regex::new(r#"vless://([a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12})@((.+):(\d+))\?(.+)#"#).unwrap();

        for captures in regex.captures_iter(source) {
            let uuid = captures.get(1).map(|id| id.as_str()).unwrap_or("");
            let host = captures.get(3).map(|host| host.as_str()).unwrap_or("");
            let port = captures.get(4).map(|port| port.as_str()).unwrap_or("");
            let url_parameters = captures.get(5).map(|params| params.as_str()).unwrap_or("");

            if uuid.is_empty() || host.is_empty() || port.is_empty() || url_parameters.is_empty() {
                continue;
            }

            let parameters: HashMap<String, String> =
                serde_urlencoded::from_str(&url_parameters).unwrap();

            let vless_proxy = VLess {
                host: host.to_string(),
                port: port.parse::<u32>().unwrap_or(0),
                id: uuid.to_string(),
                parameters: Some(parameters),
            };

            proxy_list.push(vless_proxy);
        }

        proxy_list
    }

    pub fn scrape_trojan(source: &str) -> Vec<Trojan> {
        let source = &Self::seperate_links(source);
        let mut proxy_list: Vec<Trojan> = Vec::new();
        let regex = Regex::new(r#"trojan://([A-Za-z0-9\-._~]+)@((.+):(\d+))\?(.+)#"#).unwrap();

        for captures in regex.captures_iter(source) {
            let password = captures.get(1).map(|pass| pass.as_str()).unwrap_or("");
            let host = captures.get(3).map(|host| host.as_str()).unwrap_or("");
            let port = captures.get(4).map(|port| port.as_str()).unwrap_or("");
            let url_parameters = captures.get(5).map(|params| params.as_str()).unwrap_or("");

            if password.is_empty()
                || host.is_empty()
                || port.is_empty()
                || url_parameters.is_empty()
            {
                continue;
            }

            let parameters: HashMap<String, String> =
                serde_urlencoded::from_str(&url_parameters).unwrap();

            let trojan_proxy = Trojan {
                host: host.to_string(),
                port: port.parse::<u32>().unwrap_or(0),
                password: password.to_string(),
                parameters: Some(parameters),
            };

            proxy_list.push(trojan_proxy);
        }

        proxy_list
    }

    pub fn scrape_hysteria(source: &str) -> Vec<Hysteria> {
        let source = &Self::seperate_links(source);
        let mut proxy_list: Vec<Hysteria> = Vec::new();
        let regex =
            Regex::new(r#"(hy2|hysteria2|hysteria)://([A-Za-z0-9\-._~]+)@((.+):(\d+))\?(.+)#"#)
                .unwrap();

        for captures in regex.captures_iter(source) {
            let version = captures.get(1).map(|ver| ver.as_str()).unwrap_or("");
            let auth = captures.get(2).map(|auth| auth.as_str()).unwrap_or("");
            let host = captures.get(4).map(|host| host.as_str()).unwrap_or("");
            let port = captures.get(5).map(|port| port.as_str()).unwrap_or("");
            let url_parameters = captures.get(6).map(|params| params.as_str()).unwrap_or("");

            if version.is_empty()
                || auth.is_empty()
                || host.is_empty()
                || port.is_empty()
                || url_parameters.is_empty()
            {
                continue;
            }

            let parameters: HashMap<String, String> =
                serde_urlencoded::from_str(&url_parameters).unwrap();

            let hysteria_version = match version {
                "hy2" | "hysteria2" => 2,
                _ => 1,
            };

            let hysteria_proxy = Hysteria {
                version: hysteria_version,
                host: host.to_string(),
                port: port.parse::<u32>().unwrap_or(0),
                auth: auth.to_string(),
                parameters: Some(parameters),
            };

            proxy_list.push(hysteria_proxy);
        }

        proxy_list
    }

    pub fn scrape_tuic(source: &str) -> Vec<TUIC> {
        let source = &Self::seperate_links(source);
        let mut proxy_list: Vec<TUIC> = Vec::new();
        let regex = Regex::new(r#"tuic://([A-Za-z0-9\-._~]+)@((.+):(\d+))\?(.+)#"#).unwrap();

        for captures in regex.captures_iter(source) {
            let auth = captures.get(1).map(|auth| auth.as_str()).unwrap_or("");
            let host = captures.get(3).map(|host| host.as_str()).unwrap_or("");
            let port = captures.get(4).map(|port| port.as_str()).unwrap_or("");
            let url_parameters = captures.get(5).map(|params| params.as_str()).unwrap_or("");

            if auth.is_empty() || host.is_empty() || port.is_empty() || url_parameters.is_empty() {
                continue;
            }

            let parameters: HashMap<String, String> =
                serde_urlencoded::from_str(&url_parameters).unwrap();

            let tuic_proxy = TUIC {
                host: host.to_string(),
                port: port.parse::<u32>().unwrap_or(0),
                auth: auth.to_string(),
                parameters: Some(parameters),
            };

            proxy_list.push(tuic_proxy);
        }

        proxy_list
    }
}
