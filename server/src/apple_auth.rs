use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

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
            // 更新为符合 Apple 最新要求的 User-Agent (2025/2026)
            // 使用 Configurator 2.3.0 + macOS 13.x + Safari 605
            "Configurator/2.3.0 (Macintosh; OS X 13.5.0; 22G74) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15"
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
        let url = format!(
            "https://auth.itunes.apple.com/auth/v1/native/fast?guid={}",
            self.guid
        );

        let mut auth_data = HashMap::new();
        auth_data.insert("appleId", Value::String(email.to_string()));
        auth_data.insert(
            "attempt",
            Value::Number(serde_json::Number::from(if mfa.is_some() { 2 } else { 4 })),
        );
        auth_data.insert("createSession", Value::String("true".to_string()));
        auth_data.insert("guid", Value::String(self.guid.clone()));
        auth_data.insert(
            "password",
            Value::String(format!("{}{}", password, mfa.unwrap_or(""))),
        );
        auth_data.insert("rmp", Value::Number(serde_json::Number::from(0)));
        auth_data.insert("why", Value::String("signIn".to_string()));

        let response = self
            .client
            .post(&url)
            .headers(Self::get_headers())
            .form(&auth_data)
            .send()
            .await?;

        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();

        log::info!(
            "Apple auth response: status={}, body_len={}",
            status,
            body_text.len()
        );

        // Try to parse JSON; if it fails, build a structured error
        let result: HashMap<String, Value> = if body_text.trim().starts_with('{') || body_text.trim().starts_with('<') {
            // Attempt JSON parse; if Apple returned non-JSON, synthesize a failure
            match serde_json::from_str(&body_text) {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("Apple returned non-JSON response ({} bytes): parse error {}", body_text.len(), e);
                    let mut m = HashMap::new();
                    m.insert("_state".to_string(), Value::String("failure".to_string()));
                    m.insert("failureType".to_string(), Value::String("NonJSONResponse".to_string()));
                    m.insert("customerMessage".to_string(), Value::String(
                        "Apple 返回了非预期响应，请检查网络环境或稍后重试".to_string()
                    ));
                    return Ok(m);
                }
            }
        } else {
            // Empty or unexpected body
            log::warn!("Apple returned unexpected body ({} bytes): {:100}", body_text.len(), body_text);
            let mut m = HashMap::new();
            m.insert("_state".to_string(), Value::String("failure".to_string()));
            m.insert("failureType".to_string(), Value::String("EmptyResponse".to_string()));
            m.insert("customerMessage".to_string(), Value::String(
                if status.as_u16() == 403 {
                    "Apple 拒绝了登录请求 (403)，可能需要检查 IP 环境或使用应用专用密码".to_string()
                } else if status.as_u16() == 429 {
                    "Apple 登录请求过于频繁 (429)，请稍后再试".to_string()
                } else {
                    format!("Apple 返回了空响应 (HTTP {})", status)
                }
            ));
            return Ok(m);
        };

        let mut final_result = result.clone();

        // Check for explicit success indicators
        let has_success_fields = result.contains_key("dsPersonId")
            || result.contains_key("passwordToken")
            || result.contains_key("adsid");

        if has_success_fields && !result.contains_key("failureType") {
            final_result.insert("_state".to_string(), Value::String("success".to_string()));
        } else if result.contains_key("failureType") {
            final_result.insert("_state".to_string(), Value::String("failure".to_string()));
            let ft = result.get("failureType").and_then(|v| v.as_str()).unwrap_or("");
            let cm = result.get("customerMessage").and_then(|v| v.as_str()).unwrap_or("");
            log::warn!("Apple auth failure: type={}, message={}", ft, cm);
        } else {
            // Ambiguous — could be a redirect or new auth flow requirement
            final_result.insert("_state".to_string(), Value::String("failure".to_string()));
            log::warn!("Apple auth ambiguous response (no success fields, no failureType): {:?}",
                result.keys().collect::<Vec<_>>());
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
