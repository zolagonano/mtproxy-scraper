use regex::Regex;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

/// Represents a Shadowsocks proxy.
#[derive(Debug)]
pub struct Shadowsocks {
    /// The host address of the Shadowsocks proxy.
    pub host: String,
    /// The port number for the Shadowsocks proxy.
    pub port: u32,
    /// The password associated with the Shadowsocks proxy.
    pub password: String,
    /// The encryption method used by the Shadowsocks proxy.
    pub method: String,
}

impl Shadowsocks {
    pub fn to_url(&self) -> String {
        let base64_part = URL_SAFE.encode(format!("{}:{}", self.method, self.password));
        format!("ss://{}@{}:{}", base64_part, self.host, self.port)
    }

    pub fn scrape(source: &str) -> Vec<Self> {
        let source = crate::utils::seperate_links(source);
        let mut proxy_list: Vec<Shadowsocks> = Vec::new();
        let regex = Regex::new(
            r#"ss://((?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?)@((.+):(\d+))#"#,
        )
        .unwrap();

        for captures in regex.captures_iter(&source) {
            let base64_part = captures.get(1).map(|b64| b64.as_str()).unwrap_or("");
            let host = captures.get(3).map(|h| h.as_str()).unwrap_or("");
            let port: u32 = captures
                .get(4)
                .map(|p| p.as_str())
                .unwrap_or("0")
                .parse::<u32>()
                .unwrap();

            if base64_part.is_empty() || host.is_empty() || port == 0 {
                continue;
            }

            let decoded_base64_part =
                String::from_utf8(URL_SAFE.decode(&base64_part).unwrap()).unwrap();
            let parts: Vec<&str> = decoded_base64_part.split(":").collect();

            let method = parts[0].to_string();
            let password = parts[1].to_string();

            proxy_list.push(Shadowsocks {
                host: host.to_string(),
                port,
                password,
                method,
            });
        }
        proxy_list
    }
}


