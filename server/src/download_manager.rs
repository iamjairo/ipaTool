use crate::database::{Database, DownloadRecord};
use crate::ipa_handler::{download_ipa_with_account, AppleAuthService, DownloadParams, DownloadResult};
use reqwest::Client;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct DownloadManager {
    db: Arc<Mutex<Database>>,
    client: Client,
    max_retries: usize,
    retry_delay: Duration,
    chunk_size: usize,
}

impl DownloadManager {
    pub fn new(db: Arc<Mutex<Database>>) -> Self {
        Self {
            db,
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .connect_timeout(Duration::from_secs(30))
                .pool_idle_timeout(Duration::from_secs(90))
                .build()
                .unwrap_or_default(),
            max_retries: 5,
            retry_delay: Duration::from_millis(3000),
            chunk_size: 5 * 1024 * 1024, // 5MB chunks
        }
    }

    // 批量下载功能 - 简化版
    pub async fn start_batch_download<S: AppleAuthService + Clone + Send + Sync + 'static>(
        &self,
        task_name: &str,
        items: Vec<BatchItem<S>>,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let total_count = items.len() as i64;
        let batch_id = self.db.lock().unwrap().create_batch_task(task_name, total_count)?;

        // 添加所有项目到数据库
        for item in &items {
            self.db.lock().unwrap().add_batch_item(
                batch_id,
                &item.app_id,
                item.app_name.as_deref(),
                item.version.as_deref(),
                &item.account_email,
            )?;
        }

        // 异步执行批量下载
        let db_clone = Arc::clone(&self.db);
        tokio::spawn(async move {
            Self::process_batch_download(db_clone, batch_id, items).await;
        });

        Ok(batch_id)
    }

    async fn process_batch_download<S: AppleAuthService + Clone + Send + Sync>(
        db: Arc<Mutex<Database>>,
        batch_id: i64,
        items: Vec<BatchItem<S>>,
    ) {
        let mut completed_count = 0i64;
        let mut failed_count = 0i64;

        for (index, item) in items.iter().enumerate() {
            // 获取数据库中的项目ID
            let batch_items = db.lock().unwrap().get_batch_items(batch_id).unwrap_or_default();
            let db_item = batch_items.get(index);

            if let Some(db_item) = db_item {
                let item_id = db_item.id.unwrap();

                let result = Self::download_with_retry(
                    &db,
                    &item.store,
                    &item.app_id,
                    item.version.as_deref(),
                    &item.account_email,
                    item_id,
                )
                .await;

                match result {
                    Ok(_) => {
                        completed_count += 1;
                        let _ = db.lock().unwrap().update_batch_item(item_id, "completed", 100, None, 0);
                    }
                    Err(e) => {
                        failed_count += 1;
                        let _ = db.lock().unwrap().update_batch_item(item_id, "failed", 0, Some(&e.to_string()), 5);
                    }
                }

                let status = if completed_count + failed_count == items.len() as i64 {
                    "completed"
                } else {
                    "processing"
                };

                let _ = db.lock().unwrap().update_batch_task_progress(batch_id, completed_count, failed_count, status);
            }
        }

        // 标记批量任务完成
        let _ = db.lock().unwrap().update_batch_task_progress(batch_id, completed_count, failed_count, "completed");
    }

    // 带重试的下载
    async fn download_with_retry<S: AppleAuthService + Clone + Send + Sync>(
        db: &Arc<Mutex<Database>>,
        store: &S,
        app_id: &str,
        version: Option<&str>,
        account_email: &str,
        item_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut retry_count = 0;
        let resume_position = 0u64;

        loop {
            let _start_time = Instant::now();

            match Self::download_with_resume(store, app_id, version, account_email, resume_position).await {
                Ok(result) => {
                    // 记录成功下载
                    if let Some(metadata) = result.metadata {
                        let record = DownloadRecord {
                            id: None,
                            app_name: metadata.bundle_display_name.clone(),
                            app_id: app_id.to_string(),
                            bundle_id: Some(metadata.bundle_id.clone()),
                            version: Some(metadata.bundle_short_version_string.clone()),
                            account_email: account_email.to_string(),
                            account_region: None,
                            download_date: Some(chrono::Utc::now().to_rfc3339()),
                            status: "completed".to_string(),
                            file_size: None,
                            install_url: None,
                            artwork_url: Some(metadata.artwork_url.clone()),
                            artist_name: Some(metadata.artist_name.clone()),
                            progress: Some(100),
                            error: None,
                            created_at: None,
                        };
                        let _ = db.lock().unwrap().add_download_record(&record);
                    }
                    return Ok(());
                }
                Err(e) => {
                    retry_count += 1;
                    let _ = db.lock().unwrap().update_batch_item(item_id, "retrying", 0, Some(&e.to_string()), retry_count as i64);

                    if retry_count >= 5 {
                        return Err(e);
                    }

                    // 指数退避
                    let delay = Duration::from_millis(3000 * (2_u64.pow(retry_count as u32)));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    // 断点续传下载
    async fn download_with_resume<S: AppleAuthService + Clone>(
        store: &S,
        app_id: &str,
        version: Option<&str>,
        account_email: &str,
        _resume_position: u64,
    ) -> Result<DownloadResult, Box<dyn std::error::Error + Send + Sync>> {
        let params = DownloadParams {
            store,
            email: account_email,
            appid: app_id,
            app_ver_id: version,
            download_path: "../downloads",
            auto_purchase: false,
            token: None,
            progress_callback: None,
        };

        download_ipa_with_account(params).await
    }

    // 检查应用更新
    pub async fn check_app_updates(&self) -> Result<Vec<AppUpdate>, Box<dyn std::error::Error + Send + Sync>> {
        let subscriptions = self.db.lock().unwrap().get_all_subscriptions()?;
        let mut updates = Vec::new();

        for sub in subscriptions {
            // 查询最新版本
            let versions = self.fetch_versions(&sub.app_id, sub.account_region.as_deref()).await?;

            if let Some(latest_version) = versions.first() {
                let current_version = sub.current_version.as_deref().unwrap_or("");
                let latest_version_str = latest_version.get("bundle_version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if latest_version_str != current_version {
                    // 有更新
                    let update = AppUpdate {
                        app_id: sub.app_id.clone(),
                        app_name: sub.app_name.clone(),
                        bundle_id: sub.bundle_id.clone(),
                        current_version: sub.current_version.clone().unwrap_or_default(),
                        latest_version: latest_version_str.to_string(),
                        artwork_url: sub.artwork_url.clone(),
                        artist_name: sub.artist_name.clone(),
                        account_email: sub.account_email.clone(),
                    };
                    updates.push(update);

                    // 更新数据库中的版本
                    let _ = self.db.lock().unwrap().update_subscription_version(&sub.app_id, &sub.account_email, latest_version_str);
                }

                // 更新最后检查时间
                let _ = self.db.lock().unwrap().update_subscription_version(&sub.app_id, &sub.account_email, latest_version_str);
            }
        }

        Ok(updates)
    }

    async fn fetch_versions(&self, app_id: &str, region: Option<&str>) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let region = region.unwrap_or("US");

        // 尝试第一个 API
        let url1 = format!(
            "https://api.timbrd.com/apple/app-version/index.php?id={}&country={}",
            app_id, region
        );

        let response1 = self.client.get(&url1).send().await;
        let versions = if let Ok(resp) = response1 {
            resp.json::<Value>()
                .await
                .ok()
                .and_then(|json| json.get("data").and_then(|d| d.as_array()).cloned())
        } else {
            None
        };

        let final_versions = if let Some(vers) = versions {
            vers
        } else {
            // 尝试第二个 API
            let url2 = format!(
                "https://apis.bilin.eu.org/history/{}?country={}",
                app_id, region
            );

            let response2 = self.client.get(&url2).send().await;
            if let Ok(resp) = response2 {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                        data.clone()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        };

        Ok(final_versions)
    }

    // 计算下载速度
    pub fn calculate_speed(&self, downloaded: u64, elapsed: Duration) -> f64 {
        if elapsed.as_secs() == 0 {
            return 0.0;
        }
        (downloaded as f64) / (elapsed.as_secs_f64()) / (1024.0 * 1024.0) // MB/s
    }
}

#[derive(Clone)]
pub struct BatchItem<S> {
    pub store: S,
    pub app_id: String,
    pub app_name: Option<String>,
    pub version: Option<String>,
    pub account_email: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppUpdate {
    pub app_id: String,
    pub app_name: String,
    pub bundle_id: Option<String>,
    pub current_version: String,
    pub latest_version: String,
    pub artwork_url: Option<String>,
    pub artist_name: Option<String>,
    pub account_email: String,
}
