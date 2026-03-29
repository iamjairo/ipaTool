use reqwest::{header, Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// Parse Apple's XML plist response into a HashMap<String, Value>.
/// Apple's auth endpoint returns XML plist (not JSON), so we need to convert.
fn parse_apple_plist_response(body: &str) -> Result<HashMap<String, Value>, String> {
    // Apple returns XML plist — parse with the plist crate
    let parsed: Result<plist::Value, _> = plist::from_bytes(body.as_bytes());
    match parsed {
        Ok(plist_val) => {
            let mut map = HashMap::new();
            if let Some(dict) = plist_val.as_dictionary() {
                for (key, val) in dict {
                    let json_val = plist_value_to_json(val);
                    map.insert(key.clone(), json_val);
                }
            }
            Ok(map)
        }
        Err(e) => {
            // Fallback: try JSON parse (in case Apple ever returns JSON)
            if body.trim().starts_with('{') {
                match serde_json::from_str::<HashMap<String, Value>>(body) {
                    Ok(m) => return Ok(m),
                    Err(je) => log::warn!("JSON fallback also failed: {}", je),
                }
            }
            // Neither plist nor JSON — log raw body for debugging
            log::error!(
                "Apple returned unparseable response ({} bytes): {:300}",
                body.len(),
                body
            );
            Err(format!("Failed to parse Apple response: {} (body_len={})", e, body.len()))
        }
    }
}

/// Convert a plist::Value to serde_json::Value
fn plist_value_to_json(val: &plist::Value) -> Value {
    match val {
        plist::Value::String(s) => Value::String(s.clone()),
        plist::Value::Boolean(b) => Value::Bool(*b),
        plist::Value::Integer(i) => {
            Value::String(i.to_string())
        }
        plist::Value::Real(f) => {
            Value::String(format!("{}", f))
        }
        plist::Value::Data(d) => {
            use base64::Engine;
            Value::String(base64::engine::general_purpose::STANDARD.encode(d))
        }
        plist::Value::Date(d) => {
            // plist::Date doesn't implement Display — use Debug as fallback
            Value::String(format!("{:?}", d))
        }
        plist::Value::Array(arr) => {
            Value::Array(arr.iter().map(plist_value_to_json).collect())
        }
        plist::Value::Dictionary(dict) => {
            let m: std::collections::HashMap<String, Value> = dict
                .iter()
                .map(|(k, v)| (k.clone(), plist_value_to_json(v)))
                .collect();
            Value::Object(m.into_iter().collect())
        }
        _ => Value::Null,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub ds_person_id: Option<String>,
    pub password_token: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Store {
    pub client: Client,
    pub guid: String,
}

impl Store {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .unwrap();

        // 生成 GUID（使用 MAC 地址或随机 UUID）
        let guid = Self::generate_guid();

        Store { client, guid }
    }

    fn generate_guid() -> String {
        // 简单的 GUID 生成
        uuid::Uuid::new_v4()
            .to_string()
            .to_uppercase()
            .replace("-", "")
    }

    pub fn get_headers() -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "User-Agent",
            // Match ipatool reference: Configurator/2.17 + macOS 15.2
            "Configurator/2.17 (Macintosh; OS X 15.2; 24C5089c) AppleWebKit/0620.1.16.11.6"
                .parse()
                .unwrap(),
        );
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        headers
    }

    pub async fn authenticate(
        &self,
        email: &str,
        password: &str,
        mfa: Option<&str>,
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = "https://auth.itunes.apple.com/auth/v1/native/fast";
        let mut url = format!("{}?guid={}", base_url, self.guid);
        let mut last_result: Option<HashMap<String, Value>> = None;

        // auth.itunes.apple.com/auth/v1/native/fast behaves differently from
        // buy.itunes.apple.com/WebObjects/MZFinance.woa/wa/authenticate:
        // - initial password-only login can start at attempt=1
        // - MFA retry must start at attempt=2, otherwise Apple often returns HTTP 500
        let start_attempt = if mfa.is_some() { 2 } else { 1 };

        for attempt in start_attempt..=4u32 {
            let combined_password = format!("{}{}", password, mfa.unwrap_or("").replace(" ", ""));

            let mut auth_data = HashMap::new();
            auth_data.insert("appleId", email.to_string());
            auth_data.insert("attempt", attempt.to_string());
            auth_data.insert("guid", self.guid.clone());
            auth_data.insert("password", combined_password);
            auth_data.insert("rmp", "0".to_string());
            auth_data.insert("why", "signIn".to_string());

            log::info!(
                "Apple auth attempt {}: url={}, has_mfa={}, guid={}",
                attempt, url, mfa.is_some(), self.guid
            );

            let response = self
                .client
                .post(&url)
                .headers(Self::get_headers())
                .form(&auth_data)
                .send()
                .await?;

            let status = response.status();

            // Handle 302 redirect — follow to new URL
            if status == StatusCode::FOUND || status == StatusCode::MOVED_PERMANENTLY {
                if let Some(location) = response.headers().get("location") {
                    if let Ok(loc_str) = location.to_str() {
                        url = loc_str.to_string();
                        log::info!("Apple auth redirect -> {}", url);
                        continue;
                    }
                }
            }

            let body_text = response.text().await.unwrap_or_default();

            log::info!(
                "Apple auth response: status={}, body_len={}",
                status,
                body_text.len()
            );

            // Parse plist response
            let result = match parse_apple_plist_response(&body_text) {
                Ok(m) => m,
                Err(e) => {
                    log::error!(
                        "Failed to parse Apple response ({} bytes, attempt {}): {}",
                        body_text.len(), attempt, e
                    );
                    let mut m = HashMap::new();
                    m.insert("_state".to_string(), Value::String("failure".to_string()));
                    m.insert("failureType".to_string(), Value::String("ParseError".to_string()));
                    m.insert("customerMessage".to_string(), Value::String(
                        "无法解析 Apple 的响应，请稍后重试".to_string()
                    ));
                    return Ok(m);
                }
            };

            let failure_type = result.get("failureType").and_then(|v| v.as_str()).unwrap_or("");

            // Check for -5000 (InvalidCredentials) on attempt 1 → auto retry
            if attempt == 1 && failure_type == "-5000" {
                log::info!("Apple auth: InvalidCredentials on attempt 1, auto-retrying");
                last_result = Some(result);
                continue;
            }

            last_result = Some(result);
            break;
        }

        let result = last_result.unwrap_or_default();
        let mut final_result = result.clone();

        // Check for success
        let has_success = result.contains_key("dsPersonId")
            || result.contains_key("passwordToken");

        if has_success {
            final_result.insert("_state".to_string(), Value::String("success".to_string()));
            log::info!("Apple auth SUCCESS for {}", email);
        } else {
            final_result.insert("_state".to_string(), Value::String("failure".to_string()));
            let ft = result.get("failureType").and_then(|v| v.as_str()).unwrap_or("");
            let cm = result.get("customerMessage").and_then(|v| v.as_str()).unwrap_or("");
            log::warn!("Apple auth failure: type='{}', msg='{}'", ft, cm);
        }

        Ok(final_result)
    }

    pub async fn ensure_license(
        &self,
        app_identifier: &str,
        app_ver_id: Option<&str>,
        auth_info: &AuthInfo,
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://p25-buy.itunes.apple.com/WebObjects/MZFinance.woa/wa/buyProduct?guid={}",
            self.guid
        );

        let mut purchase_data = HashMap::new();
        purchase_data.insert("guid", Value::String(self.guid.clone()));
        purchase_data.insert("salableAdamId", Value::String(app_identifier.to_string()));
        if let Some(ver_id) = app_ver_id {
            purchase_data.insert("externalVersionId", Value::String(ver_id.to_string()));
            purchase_data.insert("appExtVrsId", Value::String(ver_id.to_string()));
        }
        purchase_data.insert("pricingParameters", Value::String("STDQ".to_string()));

        let mut headers = Self::get_headers();
        if let Some(ds_id) = &auth_info.ds_person_id {
            headers.insert("X-Dsid", ds_id.parse().unwrap());
            headers.insert("iCloud-DSID", ds_id.parse().unwrap());
        }
        if let Some(token) = &auth_info.password_token {
            headers.insert("X-Token", token.parse().unwrap());
        }

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .form(&purchase_data)
            .send()
            .await?;

        let mut result: HashMap<String, Value> = response.json().await?;
        result.insert("_state".to_string(), Value::String("success".to_string()));
        Ok(result)
    }

    pub async fn download_product(
        &self,
        app_identifier: &str,
        app_ver_id: Option<&str>,
        auth_info: &AuthInfo,
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://p25-buy.itunes.apple.com/WebObjects/MZFinance.woa/wa/volumeStoreDownloadProduct?guid={}",
            self.guid
        );

        let mut download_data = HashMap::new();
        download_data.insert("creditDisplay", Value::String("".to_string()));
        download_data.insert("guid", Value::String(self.guid.clone()));
        download_data.insert("salableAdamId", Value::String(app_identifier.to_string()));
        if let Some(ver_id) = app_ver_id {
            download_data.insert("externalVersionId", Value::String(ver_id.to_string()));
        }

        let mut headers = Self::get_headers();
        if let Some(ds_id) = &auth_info.ds_person_id {
            headers.insert("X-Dsid", ds_id.parse().unwrap());
            headers.insert("iCloud-DSID", ds_id.parse().unwrap());
        }
        if let Some(token) = &auth_info.password_token {
            headers.insert("X-Token", token.parse().unwrap());
        }

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .form(&download_data)
            .send()
            .await?;

        let mut result: HashMap<String, Value> = response.json().await?;
        result.insert("_state".to_string(), Value::String("success".to_string()));
        Ok(result)
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AccountStore {
    pub store: Store,
    pub account_email: String,
    pub auth_info: Option<AuthInfo>,
}

impl AccountStore {
    pub fn new(email: &str) -> Self {
        AccountStore {
            store: Store::new(),
            account_email: email.to_string(),
            auth_info: None,
        }
    }

    pub async fn authenticate(
        &mut self,
        password: &str,
        mfa: Option<&str>,
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error + Send + Sync>> {
        let result = self
            .store
            .authenticate(&self.account_email, password, mfa)
            .await?;

        // 提取认证信息
        if result.get("_state").and_then(|v| v.as_str()) == Some("success") {
            let auth_info = AuthInfo {
                ds_person_id: result
                    .get("dsPersonId")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                password_token: result
                    .get("passwordToken")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                display_name: result
                    .get("displayName")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                email: Some(self.account_email.clone()),
            };
            self.auth_info = Some(auth_info);
        }

        Ok(result)
    }

    pub async fn download_product(
        &self,
        app_identifier: &str,
        app_ver_id: Option<&str>,
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error + Send + Sync>> {
        let auth_info = self.auth_info.as_ref().ok_or("Not authenticated")?;
        self.store
            .download_product(app_identifier, app_ver_id, auth_info)
            .await
    }

    pub async fn ensure_license(
        &self,
        app_identifier: &str,
        app_ver_id: Option<&str>,
    ) -> Result<HashMap<String, Value>, Box<dyn std::error::Error + Send + Sync>> {
        let auth_info = self.auth_info.as_ref().ok_or("Not authenticated")?;
        self.store
            .ensure_license(app_identifier, app_ver_id, auth_info)
            .await
    }
}
