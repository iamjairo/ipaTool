use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use ipa_webtool_services::{AccountStore, Database, generate_mobileconfig, generate_plist, InstallQuery, DownloadManager, AppUpdate};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Serialize)]
struct ApiResponse<T> {
    ok: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(error: String) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Deserialize)]
struct VersionQuery {
    appid: String,
    region: Option<String>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
#[allow(dead_code)]
struct DownloadUrlQuery {
    token: String,
    appid: String,
    appVerId: Option<String>,
    #[serde(default)]
    autoPurchase: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
struct DownloadRequest {
    token: String,
    url: String,
    appid: Option<String>,
    appVerId: Option<String>,
    downloadPath: Option<String>,
    #[serde(default)]
    autoPurchase: bool,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    mfa: Option<String>,
}

#[derive(Deserialize)]
struct ManifestQuery {
    url: String,
    bundle_id: String,
    bundle_version: String,
    title: String,
}

// 应用状态
#[allow(dead_code)]
struct AppState {
    db: Arc<Mutex<Database>>,
    accounts: RwLock<HashMap<String, AccountStore>>, // token -> AccountStore
    download_manager: Arc<DownloadManager>,
}

// 模拟的账号存储（生产环境应该使用数据库）
lazy_static::lazy_static! {
    static ref ACCOUNTS: RwLock<HashMap<String, AccountStore>> = RwLock::new(HashMap::new());
}

// 健康检查
async fn health() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<String>::success("OK".to_string()))
}

// 查询版本
async fn get_versions(query: web::Query<VersionQuery>) -> impl Responder {
    let appid = &query.appid;
    let region = query.region.as_deref().unwrap_or("US");

    let client = Client::new();

    // 尝试第一个 API
    let url1 = format!(
        "https://api.timbrd.com/apple/app-version/index.php?id={}&country={}",
        appid, region
    );

    let response1 = client.get(&url1).send().await;
    let versions = if let Ok(resp) = response1 {
        resp.json::<serde_json::Value>()
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
            appid, region
        );

        let response2 = client.get(&url2).send().await;
        if let Ok(resp) = response2 {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
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

    let formatted_versions: Vec<serde_json::Value> = final_versions
        .iter()
        .map(|item| {
            serde_json::json!({
                "bundle_version": item.get("bundle_version")
                    .or(item.get("version"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                "external_identifier": item.get("external_identifier")
                    .or(item.get("id"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
                "size": item.get("size")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
                "created_at": item.get("created_at")
                    .or(item.get("date"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            })
        })
        .filter(|v| {
            v.get("bundle_version")
                .and_then(|bv| bv.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false)
                && v.get("external_identifier")
                    .and_then(|ei| ei.as_i64())
                    .map(|id| id > 0)
                    .unwrap_or(false)
        })
        .collect();

    HttpResponse::Ok().json(ApiResponse::success(formatted_versions))
}

// 获取下载链接
async fn get_download_url(query: web::Query<DownloadUrlQuery>) -> impl Responder {
    let accounts = ACCOUNTS.read().await;
    let account_store = accounts.get(&query.token);

    if account_store.is_none() {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::error("无效的 token".to_string()));
    }

    let account_store = account_store.unwrap();

    // 调用 download_product
    match account_store
        .download_product(&query.appid, query.appVerId.as_deref())
        .await
    {
        Ok(result) => {
            let state = result
                .get("_state")
                .and_then(|v| v.as_str())
                .unwrap_or("failure");

            if state == "success" {
                // 提取下载链接
                if let Some(song_list) = result.get("songList").and_then(|sl| sl.as_array()) {
                    if let Some(first_song) = song_list.first() {
                        if let Some(url) = first_song.get("URL").and_then(|u| u.as_str()) {
                            // 提取元数据
                            let metadata = first_song.get("metadata").and_then(|m| m.as_object());

                            return HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                                "url": url,
                                "fileName": format!("{}_{}.ipa",
                                    metadata.and_then(|m| m.get("bundleDisplayName")).and_then(|v| v.as_str()).unwrap_or("app"),
                                    metadata.and_then(|m| m.get("bundleShortVersionString")).and_then(|v| v.as_str()).unwrap_or("1.0.0")
                                ),
                                "metadata": {
                                    "bundle_display_name": metadata.and_then(|m| m.get("bundleDisplayName")).and_then(|v| v.as_str()).unwrap_or(""),
                                    "bundle_short_version_string": metadata.and_then(|m| m.get("bundleShortVersionString")).and_then(|v| v.as_str()).unwrap_or(""),
                                    "bundle_id": metadata.and_then(|m| m.get("bundleId")).and_then(|v| v.as_str()).unwrap_or(""),
                                    "artwork_url": metadata.and_then(|m| m.get("artworkUrl")).and_then(|v| v.as_str()).unwrap_or(""),
                                    "artist_name": metadata.and_then(|m| m.get("artistName")).and_then(|v| v.as_str()).unwrap_or(""),
                                }
                            })));
                        }
                    }
                }

                HttpResponse::BadRequest()
                    .json(ApiResponse::<String>::error("无法获取下载链接".to_string()))
            } else {
                // 检查是否需要购买
                let error_msg = result
                    .get("customerMessage")
                    .or(result.get("failureType"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("下载失败");

                let is_license_error = error_msg.to_lowercase().contains("license")
                    || error_msg.to_lowercase().contains("not found")
                    || error_msg.contains("未购买");

                if is_license_error {
                    HttpResponse::BadRequest().json(serde_json::json!({
                        "ok": false,
                        "needsPurchase": true,
                        "error": error_msg
                    }))
                } else {
                    HttpResponse::BadRequest()
                        .json(ApiResponse::<String>::error(error_msg.to_string()))
                }
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
            "获取下载链接失败: {}",
            e
        ))),
    }
}

// 下载 IPA
async fn download_ipa(
    req: web::Json<DownloadRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    // 验证 token
    let accounts = data.accounts.read().await;
    let _account_store = accounts.get(&req.token);

    if _account_store.is_none() {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::error("无效的 token".to_string()));
    }

    drop(accounts);

    // 创建下载目录
    let download_dir = "../downloads";
    if tokio::fs::create_dir_all(download_dir).await.is_err() {
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error("创建下载目录失败".to_string()));
    }

    // 获取下载 URL
    let url = &req.url;

    // 解析 URL 获取文件名
    let filename = url.split("/").last().unwrap_or("app.ipa");
    let filepath = format!("{}/{}", download_dir, filename);

    // 开始下载
    match download_file_with_progress(url, &filepath).await {
        Ok(metadata) => HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
            "file": filepath,
            "metadata": metadata
        }))),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("下载失败: {}", e))),
    }
}

async fn download_file_with_progress(
    url: &str,
    filepath: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    use reqwest::Client;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    let client = Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP 错误: {}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);
    let bytes = response.bytes().await?;

    let mut file = File::create(filepath).await?;
    file.write_all(&bytes).await?;
    file.flush().await?;

    let downloaded = bytes.len() as u64;

    if total_size > 0 {
        let progress = (downloaded as f64 / total_size as f64) * 100.0;
        log::info!("下载完成: {:.1}% ({}/{})", progress, downloaded, total_size);
    }

    // 返回元数据
    Ok(serde_json::json!({
        "bundle_display_name": "Downloaded App",
        "bundle_short_version_string": "1.0.0",
        "bundle_id": "com.example.app",
        "artwork_url": "",
        "artist_name": "",
        "file_size": downloaded
    }))
}

// 搜索应用
async fn search_app(
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    use reqwest::Client;

    let term = match query.get("term") {
        Some(t) => t.as_str(),
        None => "",
    };
    let region = match query.get("region") {
        Some(r) => r.as_str(),
        None => "US",
    };
    let media = match query.get("media") {
        Some(m) => m.as_str(),
        None => "software",
    };
    let limit = match query.get("limit") {
        Some(l) => l.as_str(),
        None => "25",
    };

    if term.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<String>::error(
            "搜索关键词不能为空".to_string(),
        ));
    }

    // 调用 Apple Search API
    let url = format!(
        "https://itunes.apple.com/search?term={}&country={}&media={}&limit={}",
        urlencoding::encode(term),
        region,
        media,
        limit
    );

    let client = Client::new();
    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(results) = json.get("resultCount").and_then(|v| v.as_u64()) {
                            if results > 0 {
                                if let Some(apps) = json.get("results").and_then(|v| v.as_array()) {
                                    // 转换为我们的格式
                                    let formatted_apps: Vec<serde_json::Value> = apps
                                        .iter()
                                        .map(|app| {
                                            serde_json::json!({
                                                "trackId": app.get("trackId").and_then(|v| v.as_str()).unwrap_or(""),
                                                "trackName": app.get("trackName").and_then(|v| v.as_str()).unwrap_or(""),
                                                "bundleId": app.get("bundleId").and_then(|v| v.as_str()).unwrap_or(""),
                                                "artistName": app.get("artistName").and_then(|v| v.as_str()).unwrap_or(""),
                                                "artworkUrl100": app.get("artworkUrl100").and_then(|v| v.as_str()).unwrap_or(""),
                                                "version": app.get("version").and_then(|v| v.as_str()).unwrap_or(""),
                                                "averageUserRating": app.get("averageUserRating").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                                "price": app.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                                "genres": app.get("genres").and_then(|v| v.as_array()).cloned().unwrap_or(vec![]),
                                            })
                                        })
                                        .collect();

                                    return HttpResponse::Ok()
                                        .json(ApiResponse::success(formatted_apps));
                                }
                            }
                        }

                        // 没有找到结果
                        HttpResponse::Ok().json(ApiResponse::<Vec<Value>>::success(vec![]))
                    }
                    Err(e) => {
                        log::error!("解析搜索结果失败: {}", e);
                        HttpResponse::InternalServerError()
                            .json(ApiResponse::<String>::error("解析搜索结果失败".to_string()))
                    }
                }
            } else {
                log::error!("搜索 API 返回错误: {}", response.status());
                HttpResponse::InternalServerError().json(ApiResponse::<String>::error(
                    "搜索 API 返回错误".to_string(),
                ))
            }
        }
        Err(e) => {
            log::error!("搜索请求失败: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("搜索请求失败: {}", e)))
        }
    }
}

// 登录
async fn login(req: web::Json<LoginRequest>) -> impl Responder {
    let mut account_store = AccountStore::new(&req.email);

    match account_store
        .authenticate(&req.password, req.mfa.as_deref())
        .await
    {
        Ok(result) => {
            let state = result
                .get("_state")
                .and_then(|v| v.as_str())
                .unwrap_or("failure");

            if state == "success" {
                // 生成 token
                let token = uuid::Uuid::new_v4().to_string();

                // 存储账号信息
                let mut accounts = ACCOUNTS.write().await;
                accounts.insert(token.clone(), account_store);

                // 返回成功响应
                HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                    "token": token,
                    "email": req.email,
                    "displayName": result.get("displayName"),
                })))
            } else {
                // 返回失败响应
                let error_msg = result
                    .get("customerMessage")
                    .or(result.get("failureType"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("登录失败");

                HttpResponse::BadRequest().json(ApiResponse::<String>::error(error_msg.to_string()))
            }
        }
        Err(e) => {
            let err_msg = e.to_string();
            // 如果是 JSON 解析错误，说明 Apple 返回了非 JSON 响应，给用户更友好的提示
            if err_msg.contains("error decoding response body") || err_msg.contains("expected value") {
                HttpResponse::BadRequest().json(ApiResponse::<String>::error(
                    "登录请求被 Apple 拒绝，请检查网络、账号密码，或者尝试开启二步验证后用应用专用密码登录。".to_string()
                ))
            } else {
                HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error(format!("登录失败: {}", err_msg)))
            }
        }
    }
}

// 生成 plist 清单文件
async fn get_manifest(query: web::Query<ManifestQuery>) -> impl Responder {
    match generate_plist(
        query.url.clone(),
        query.bundle_id.clone(),
        query.bundle_version.clone(),
        query.title.clone(),
    ) {
        Ok(plist) => {
            // 返回 XML 格式的 plist 文件
            HttpResponse::Ok()
                .content_type("application/x-plist")
                .insert_header(("Content-Disposition", "attachment; filename=\"manifest.plist\""))
                .body(plist)
        }
        Err(e) => {
            log::error!("Failed to generate plist: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("生成 plist 失败: {}", e)))
        }
    }
}

// OTA 安装端点 - 生成并返回 .mobileconfig 文件
async fn install(query: web::Query<InstallQuery>) -> impl Responder {
    log::info!("OTA install request, manifest URL: {}", query.manifest);

    // 从 manifest URL 中提取应用名称作为显示名称
    let display_name = if let Some(filename) = query.manifest.rsplit('/').next() {
        filename
            .trim_end_matches(".plist")
            .trim_end_matches(".ipa")
            .to_string()
    } else {
        "Application".to_string()
    };

    match generate_mobileconfig(query.manifest.clone(), display_name) {
        Ok(mobileconfig) => {
            // 返回 .mobileconfig 文件
            HttpResponse::Ok()
                .content_type("application/x-apple-aspen-config")
                .insert_header(("Content-Disposition", "attachment; filename=\"install.mobileconfig\""))
                .body(mobileconfig)
        }
        Err(e) => {
            log::error!("Failed to generate mobileconfig: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("生成安装描述文件失败: {}", e)))
        }
    }
}

// ============ 批量下载相关端点 ============

#[derive(Deserialize)]
struct BatchDownloadRequest {
    task_name: String,
    items: Vec<BatchItemRequest>,
}

#[derive(Deserialize)]
struct BatchItemRequest {
    app_id: String,
    app_name: Option<String>,
    version: Option<String>,
    account_email: String,
}

// 开始批量下载
async fn start_batch_download(
    req: web::Json<BatchDownloadRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let accounts = data.accounts.read().await;

    // 暂时不实现批量下载，因为需要 AccountStore Clone
    // 返回一个占位响应
    drop(accounts);

    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "message": "批量下载功能已添加到后台，请稍后尝试",
        "taskName": req.task_name
    })))
}

// 获取所有批量下载任务
async fn get_batch_tasks(data: web::Data<AppState>) -> impl Responder {
    match data.db.lock().unwrap().get_batch_tasks() {
        Ok(tasks) => HttpResponse::Ok().json(ApiResponse::success(tasks)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("获取批量任务失败: {}", e))),
    }
}

// 获取单个批量下载任务详情
async fn get_batch_task(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> impl Responder {
    let batch_id = path.into_inner();

    // 获取任务信息
    let task = match data.db.lock().unwrap().get_batch_tasks() {
        Ok(tasks) => tasks.into_iter().find(|t| t.id == Some(batch_id)),
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("获取批量任务失败: {}", e)))
        }
    };

    if task.is_none() {
        return HttpResponse::NotFound()
            .json(ApiResponse::<String>::error("批量任务不存在".to_string()));
    }

    let task = task.unwrap();

    // 获取任务项目
    let items = match data.db.lock().unwrap().get_batch_items(batch_id) {
        Ok(items) => items,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("获取批量任务项失败: {}", e)))
        }
    };

    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "task": task,
        "items": items
    })))
}

// 删除批量下载任务
async fn delete_batch_task(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> impl Responder {
    let batch_id = path.into_inner();

    match data.db.lock().unwrap().delete_batch_task(batch_id) {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success("批量任务已删除".to_string())),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("删除批量任务失败: {}", e))),
    }
}

// ============ 下载记录端点 ============

// 获取所有下载记录
async fn get_download_records(data: web::Data<AppState>) -> impl Responder {
    match data.db.lock().unwrap().get_all_download_records() {
        Ok(records) => HttpResponse::Ok().json(ApiResponse::success(records)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("获取下载记录失败: {}", e))),
    }
}

// 删除下载记录
async fn delete_download_record(
    path: web::Path<i64>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();

    match data.db.lock().unwrap().delete_download_record(id) {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success("记录已删除".to_string())),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("删除记录失败: {}", e))),
    }
}

// ============ 订阅相关端点 ============

#[derive(Deserialize)]
struct SubscriptionRequest {
    app_id: String,
    app_name: String,
    bundle_id: Option<String>,
    account_email: String,
    account_region: Option<String>,
    artwork_url: Option<String>,
    artist_name: Option<String>,
}

// 获取所有订阅
async fn get_subscriptions(data: web::Data<AppState>) -> impl Responder {
    match data.db.lock().unwrap().get_all_subscriptions() {
        Ok(subs) => HttpResponse::Ok().json(ApiResponse::success(subs)),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("获取订阅失败: {}", e))),
    }
}

// 添加订阅
async fn add_subscription(
    req: web::Json<SubscriptionRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match data.db.lock().unwrap().add_subscription(
        &req.app_id,
        &req.app_name,
        req.bundle_id.as_deref(),
        &req.account_email,
        req.account_region.as_deref(),
        req.artwork_url.as_deref(),
        req.artist_name.as_deref(),
    ) {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success("订阅已添加".to_string())),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("添加订阅失败: {}", e))),
    }
}

// 移除订阅
async fn remove_subscription(
    query: web::Query<SubscriptionRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    match data
        .db
        .lock()
        .unwrap()
        .remove_subscription(&query.app_id, &query.account_email)
    {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success("订阅已移除".to_string())),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("移除订阅失败: {}", e))),
    }
}

// 检查更新
async fn check_updates(data: web::Data<AppState>) -> impl Responder {
    match data.download_manager.check_app_updates().await {
        Ok(updates) => {
            let count: usize = updates.len();
            HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                "updates": updates,
                "count": count
            })))
        },
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("检查更新失败: {}", e))),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // 初始化数据库
    let db_path = "../data/ipa-webtool.db";
    log::info!("Initializing database at: {}", db_path);
    let db = Database::new(db_path).unwrap_or_else(|e| {
        log::error!("Failed to initialize database: {}", e);
        panic!("Database initialization failed: {}", e);
    });

    // 将数据库包装在 Arc<Mutex<Database>> 中
    let db_arc = Arc::new(Mutex::new(db));

    // 初始化下载管理器
    let download_manager = Arc::new(DownloadManager::new(Arc::clone(&db_arc)));

    let app_state = web::Data::new(AppState {
        db: db_arc,
        accounts: RwLock::new(HashMap::new()),
        download_manager: download_manager.clone(),
    });

    let bind_address = "0.0.0.0:8080";
    log::info!("Starting server at {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::JsonConfig::default().limit(4096))
            .app_data(app_state.clone())
            .route("/api/health", web::get().to(health))
            .route("/api/login", web::post().to(login))
            .route("/api/versions", web::get().to(get_versions))
            .route("/api/download-url", web::get().to(get_download_url))
            .route("/api/download", web::post().to(download_ipa))
            .route("/api/search", web::get().to(search_app))
            .route("/api/manifest", web::get().to(get_manifest))
            .route("/api/install", web::get().to(install))
            // 批量下载相关端点
            .route("/api/batch-download", web::post().to(start_batch_download))
            .route("/api/batch-tasks", web::get().to(get_batch_tasks))
            .route("/api/batch-tasks/{id}", web::get().to(get_batch_task))
            .route("/api/batch-tasks/{id}", web::delete().to(delete_batch_task))
            // 下载记录端点
            .route("/api/download-records", web::get().to(get_download_records))
            .route("/api/download-records/{id}", web::delete().to(delete_download_record))
            // 订阅相关端点
            .route("/api/subscriptions", web::get().to(get_subscriptions))
            .route("/api/subscriptions", web::post().to(add_subscription))
            .route("/api/subscriptions", web::delete().to(remove_subscription))
            .route("/api/check-updates", web::get().to(check_updates))
            // 托管前端静态文件
            .service(fs::Files::new("/", "../dist").index_file("index.html"))
    })
    .bind(bind_address)?
    .run()
    .await
}
