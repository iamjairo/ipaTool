use actix_files as fs;
use actix_web::{
    body::{EitherBody, MessageBody},
    cookie::{time::Duration as CookieDuration, Cookie, SameSite},
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    middleware::{from_fn, Next},
    web, App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use chrono::{Duration, Utc};
use futures_util::future::{ready, Ready};
use ipa_webtool_services::{
    generate_mobileconfig, generate_plist, AccountStore, AdminUser, BatchItem, Database,
    DownloadManager, InstallQuery,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use uuid::Uuid;

const ADMIN_SESSION_COOKIE: &str = "ipa_admin_session";
const SESSION_TTL_DAYS: i64 = 30;

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
struct AppleLoginRequest {
    email: String,
    password: String,
    mfa: Option<String>,
    save_credentials: Option<bool>,
}

#[derive(Deserialize)]
struct AdminLoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

#[derive(Serialize, Clone)]
struct AuthUserPayload {
    username: String,
    is_default: bool,
}

impl From<&AdminUser> for AuthUserPayload {
    fn from(user: &AdminUser) -> Self {
        Self {
            username: user.username.clone(),
            is_default: user.is_default,
        }
    }
}

#[derive(Debug, Clone)]
struct AuthenticatedAdmin {
    username: String,
    is_default: bool,
    session_token: String,
}

#[derive(Deserialize)]
struct ManifestQuery {
    url: String,
    bundle_id: String,
    bundle_version: String,
    title: String,
}

// 应用状态
struct AppState {
    db: Arc<Mutex<Database>>,
    download_manager: Arc<DownloadManager>,
}

// 模拟的账号存储（生产环境应该使用数据库）
lazy_static::lazy_static! {
    static ref ACCOUNTS: RwLock<HashMap<String, AccountStore>> = RwLock::new(HashMap::new());
}

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

fn session_expires_at() -> String {
    (Utc::now() + Duration::days(SESSION_TTL_DAYS))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn build_session_cookie(token: &str) -> Cookie<'static> {
    Cookie::build(ADMIN_SESSION_COOKIE, token.to_string())
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::days(SESSION_TTL_DAYS))
        .finish()
}

fn clear_session_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::build(ADMIN_SESSION_COOKIE, "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish();
    cookie.make_removal();
    cookie
}

fn unauthorized_response() -> HttpResponse {
    HttpResponse::Unauthorized().json(ApiResponse::<String>::error("未登录或登录已过期".to_string()))
}

fn resolve_admin_session(app_state: &AppState, token: &str) -> Result<AuthenticatedAdmin, String> {
    let db = app_state
        .db
        .lock()
        .map_err(|_| "认证服务暂时不可用".to_string())?;

    let session = db
        .get_session(token)
        .map_err(|e| format!("查询登录态失败: {}", e))?
        .ok_or_else(|| "未登录或登录已过期".to_string())?;

    let user = db
        .get_admin_user(&session.username)
        .map_err(|e| format!("查询管理员失败: {}", e))?
        .ok_or_else(|| {
            let _ = db.delete_session(token);
            "管理员账号不存在".to_string()
        })?;

    Ok(AuthenticatedAdmin {
        username: user.username,
        is_default: user.is_default,
        session_token: session.token,
    })
}

impl FromRequest for AuthenticatedAdmin {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(data) => data.clone(),
            None => return ready(Err(ErrorUnauthorized("认证服务未初始化"))),
        };

        let session_cookie = match req.cookie(ADMIN_SESSION_COOKIE) {
            Some(cookie) => cookie,
            None => return ready(Err(ErrorUnauthorized("未登录或登录已过期"))),
        };

        ready(
            resolve_admin_session(app_state.get_ref(), session_cookie.value())
                .map_err(ErrorUnauthorized),
        )
    }
}

async fn require_auth<B>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<EitherBody<B>>, Error>
where
    B: MessageBody + 'static,
{
    let app_state = match req.app_data::<web::Data<AppState>>() {
        Some(data) => data.clone(),
        None => {
            return Ok(req
                .into_response(unauthorized_response())
                .map_into_right_body())
        }
    };

    let Some(session_cookie) = req.cookie(ADMIN_SESSION_COOKIE) else {
        return Ok(req
            .into_response(unauthorized_response())
            .map_into_right_body());
    };

    if let Err(error_message) = resolve_admin_session(app_state.get_ref(), session_cookie.value()) {
        return Ok(req
            .into_response(HttpResponse::Unauthorized().json(ApiResponse::<String>::error(error_message)))
            .map_into_right_body());
    }

    Ok(next.call(req).await?.map_into_left_body())
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
    _data: web::Data<AppState>,
) -> impl Responder {
    // 验证 token
    let accounts = ACCOUNTS.read().await;
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
async fn apple_login(
    req: web::Json<AppleLoginRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
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
                let dsid = result
                    .get("dsPersonId")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // 存储账号到内存
                let mut accounts = ACCOUNTS.write().await;
                accounts.insert(token.clone(), account_store);

                // 持久化账号到 DB
                if let Ok(db) = data.db.lock() {
                    let db_account = ipa_webtool_services::Account {
                        id: None,
                        token: token.clone(),
                        email: req.email.clone(),
                        region: "US".to_string(),
                        guid: None,
                        cookie_user: None,
                        cookies: None,
                        created_at: None,
                        updated_at: None,
                    };
                    let _ = db.save_account(&db_account);

                    // 可选：加密保存凭证
                    if req.save_credentials.unwrap_or(false) {
                        if let Ok(enc_key) = ipa_webtool_services::crypto::ensure_encryption_key(&db) {
                            if let Ok((ct, iv, tag)) = ipa_webtool_services::crypto::encrypt(&req.password, &enc_key) {
                                let key_id = db.get_current_encryption_key()
                                    .ok()
                                    .flatten()
                                    .map(|k| k.key_id)
                                    .unwrap_or_default();
                                let creds = ipa_webtool_services::Credentials {
                                    id: None,
                                    email: req.email.clone(),
                                    password_encrypted: ct,
                                    key_id,
                                    iv,
                                    auth_tag: tag,
                                    created_at: None,
                                    updated_at: None,
                                };
                                let _ = db.save_credentials(&creds);
                            }
                        }
                    }
                }

                // 返回成功响应
                HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                    "token": token,
                    "email": req.email,
                    "dsid": dsid,
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

// 获取已登录的 Apple 账号列表
async fn get_account_list(data: web::Data<AppState>) -> impl Responder {
    let accounts = ACCOUNTS.read().await;
    let mut list: Vec<serde_json::Value> = Vec::new();

    for (token, store) in accounts.iter() {
        let dsid = store
            .auth_info
            .as_ref()
            .and_then(|ai| ai.ds_person_id.clone())
            .unwrap_or_default();
        let email = store
            .auth_info
            .as_ref()
            .and_then(|ai| ai.email.clone())
            .unwrap_or_else(|| store.account_email.clone());
        let display_name = store
            .auth_info
            .as_ref()
            .and_then(|ai| ai.display_name.clone());

        list.push(serde_json::json!({
            "token": token,
            "email": email,
            "dsid": dsid,
            "region": "US",
            "displayName": display_name,
        }));
    }

    // 补充 DB 中有但内存中没有的账号（服务重启后恢复）
    if let Ok(db) = data.db.lock() {
        if let Ok(db_accounts) = db.get_all_accounts() {
            let tokens_set: std::collections::HashSet<_> =
                accounts.keys().cloned().collect();
            for acc in db_accounts {
                if !tokens_set.contains(&acc.token) {
                    list.push(serde_json::json!({
                        "token": acc.token,
                        "email": acc.email,
                        "dsid": "",
                        "region": acc.region,
                    }));
                }
            }
        }
    }

    HttpResponse::Ok().json(ApiResponse::success(list))
}

// 删除 Apple 账号
async fn delete_account(
    token: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let token = token.into_inner();

    // 从内存删除
    let mut accounts = ACCOUNTS.write().await;
    let removed = accounts.remove(&token).is_some();

    // 从 DB 删除
    if let Ok(db) = data.db.lock() {
        let _ = db.delete_account(&token);
        // 同时删除该 email 的凭证
        if let Some(email) = accounts // already removed, check if there's another entry
            .values()
            .find(|a| a.account_email.is_empty())
            .map(|a| a.account_email.clone())
        {
            let _ = db.delete_credentials(&email);
        }
    }

    if removed {
        HttpResponse::Ok().json(ApiResponse::success("已删除"))
    } else {
        HttpResponse::Ok().json(ApiResponse::success("已删除（仅数据库记录）"))
    }
}

// 获取已保存的凭证邮箱列表（不返回密码）
async fn get_credentials_list(data: web::Data<AppState>) -> impl Responder {
    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error("数据库不可用".to_string())),
    };

    match db.get_all_credentials() {
        Ok(creds) => {
            let emails: Vec<serde_json::Value> = creds
                .iter()
                .map(|c| serde_json::json!({ "email": c.email }))
                .collect();
            HttpResponse::Ok().json(ApiResponse::success(emails))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("获取凭证失败: {}", e))),
    }
}

// 自动登录所有保存的凭证
async fn auto_login_all(data: web::Data<AppState>) -> impl Responder {
    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error("数据库不可用".to_string())),
    };

    let credentials = match db.get_all_credentials() {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("获取凭证失败: {}", e)))
        }
    };

    // 确保 encryption key 可用
    let enc_key = match ipa_webtool_services::crypto::ensure_encryption_key(&db) {
        Ok(k) => k,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("加密密钥初始化失败: {}", e)))
        }
    };

    // 释放 DB 锁，后续操作需要异步
    drop(db);

    let mut success = Vec::new();
    let mut need_code = Vec::new();
    let mut failed = Vec::new();

    let accounts = ACCOUNTS.read().await;
    // 收集已登录的邮箱列表
    let logged_in_emails: std::collections::HashSet<String> = accounts
        .values()
        .map(|a| a.account_email.clone())
        .collect();
    drop(accounts);

    for cred in &credentials {
        // 解密密码
        let db2 = match data.db.lock() {
            Ok(d) => d,
            Err(_) => continue,
        };
        let password = match ipa_webtool_services::crypto::decrypt(
            &cred.password_encrypted,
            &cred.iv,
            &cred.auth_tag,
            &enc_key,
        ) {
            Ok(p) => p,
            Err(_) => {
                failed.push(serde_json::json!({ "email": cred.email, "error": "解密失败" }));
                continue;
            }
        };
        drop(db2);

        // 检查是否已登录
        if logged_in_emails.contains(&cred.email) {
            success.push(serde_json::json!({
                "email": cred.email,
                "alreadyLoggedIn": true,
            }));
            continue;
        }

        // 尝试登录
        let mut store = AccountStore::new(&cred.email);
        match store.authenticate(&password, None).await {
            Ok(result) => {
                let state = result.get("_state").and_then(|v| v.as_str()).unwrap_or("failure");
                if state == "success" {
                    let token = uuid::Uuid::new_v4().to_string();
                    let dsid = result
                        .get("dsPersonId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    let mut accounts = ACCOUNTS.write().await;
                    accounts.insert(token.clone(), store);

                    // 持久化
                    if let Ok(db) = data.db.lock() {
                        let db_account = ipa_webtool_services::Account {
                            id: None,
                            token: token.clone(),
                            email: cred.email.clone(),
                            region: "US".to_string(),
                            guid: None,
                            cookie_user: None,
                            cookies: None,
                            created_at: None,
                            updated_at: None,
                        };
                        let _ = db.save_account(&db_account);
                    }

                    success.push(serde_json::json!({
                        "email": cred.email,
                        "token": token,
                        "dsid": dsid,
                        "alreadyLoggedIn": false,
                    }));
                } else {
                    let err_msg = result
                        .get("customerMessage")
                        .or(result.get("failureType"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("登录失败");

                    if err_msg.contains("verification code") || err_msg.contains("two-factor") || err_msg.contains("MFA") {
                        need_code.push(serde_json::json!({ "email": cred.email }));
                    } else {
                        failed.push(serde_json::json!({ "email": cred.email, "error": err_msg }));
                    }
                }
            }
            Err(e) => {
                failed.push(serde_json::json!({ "email": cred.email, "error": e.to_string() }));
            }
        }
    }

    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "results": { "success": success, "needCode": need_code, "failed": failed }
    })))
}

// 刷新账号会话（重新认证）
async fn refresh_login(
    req: web::Json<serde_json::Value>,
    data: web::Data<AppState>,
) -> impl Responder {
    let token = match req.get("token").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::error("缺少 token".to_string())),
    };

    // 查找现有账号
    let accounts = ACCOUNTS.read().await;
    let email = match accounts.get(&token) {
        Some(store) => store.account_email.clone(),
        None => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<String>::error("账号不存在".to_string()))
        }
    };
    drop(accounts);

    // 尝试从 DB 获取保存的凭证
    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error("数据库不可用".to_string())),
    };

    let cred = match db.get_credentials(&email) {
        Ok(Some(c)) => c,
        _ => {
            return HttpResponse::BadRequest()
                .json(ApiResponse::<String>::error("未找到保存的密码，无法自动刷新。请重新登录。".to_string()))
        }
    };

    let enc_key = match ipa_webtool_services::crypto::ensure_encryption_key(&db) {
        Ok(k) => k,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("加密密钥失败: {}", e)))
        }
    };

    let password = match ipa_webtool_services::crypto::decrypt(
        &cred.password_encrypted, &cred.iv, &cred.auth_tag, &enc_key,
    ) {
        Ok(p) => p,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("解密密码失败".to_string()))
        }
    };
    drop(db);

    // 重新认证
    let mut store = AccountStore::new(&email);
    match store.authenticate(&password, None).await {
        Ok(result) => {
            let state = result.get("_state").and_then(|v| v.as_str()).unwrap_or("failure");
            if state == "success" {
                // 更新内存中的账号
                let mut accounts = ACCOUNTS.write().await;
                accounts.insert(token.clone(), store);
                HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                    "ok": true,
                    "email": email,
                })))
            } else {
                let err_msg = result
                    .get("customerMessage")
                    .or(result.get("failureType"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("刷新失败");
                HttpResponse::BadRequest().json(ApiResponse::<String>::error(err_msg.to_string()))
            }
        }
        Err(e) => {
            HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("刷新失败: {}", e)))
        }
    }
}

async fn admin_login(
    req: web::Json<AdminLoginRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let username = req.username.trim();
    let password = req.password.trim();

    if username.is_empty() || password.is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::error("用户名和密码不能为空".to_string()));
    }

    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("认证服务暂时不可用".to_string()))
        }
    };

    let user = match db.get_admin_user(username) {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<String>::error("用户名或密码错误".to_string()))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("查询管理员失败: {}", e)))
        }
    };

    if user.password_hash != hash_password(password) {
        return HttpResponse::Unauthorized()
            .json(ApiResponse::<String>::error("用户名或密码错误".to_string()));
    }

    let token = Uuid::new_v4().to_string();
    if let Err(e) = db.cleanup_expired_sessions() {
        log::warn!("清理过期登录态失败: {}", e);
    }

    if let Err(e) = db.create_session(&token, &user.username, &session_expires_at()) {
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("创建登录态失败: {}", e)));
    }

    HttpResponse::Ok()
        .cookie(build_session_cookie(&token))
        .json(ApiResponse::success(AuthUserPayload::from(&user)))
}

async fn logout(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    if let Some(session_cookie) = req.cookie(ADMIN_SESSION_COOKIE) {
        match data.db.lock() {
            Ok(db) => {
                if let Err(e) = db.delete_session(session_cookie.value()) {
                    log::warn!("清理登录态失败: {}", e);
                }
            }
            Err(_) => log::warn!("认证服务暂时不可用，跳过服务端 session 清理"),
        }
    }

    HttpResponse::Ok()
        .cookie(clear_session_cookie())
        .json(ApiResponse::success("已退出登录".to_string()))
}

async fn me(admin: AuthenticatedAdmin) -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success(AuthUserPayload {
        username: admin.username,
        is_default: admin.is_default,
    }))
}

async fn change_password(
    admin: AuthenticatedAdmin,
    req: web::Json<ChangePasswordRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    if req.new_password.trim().is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::error("新密码不能为空".to_string()));
    }

    if req.current_password == req.new_password {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::error("新密码不能与当前密码相同".to_string()));
    }

    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("认证服务暂时不可用".to_string()))
        }
    };

    let user = match db.get_admin_user(&admin.username) {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<String>::error("管理员账号不存在".to_string()))
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error(format!("查询管理员失败: {}", e)))
        }
    };

    if user.password_hash != hash_password(&req.current_password) {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::error("当前密码不正确".to_string()));
    }

    if let Err(e) = db.update_admin_password(&admin.username, &hash_password(&req.new_password), false) {
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("修改密码失败: {}", e)));
    }

    HttpResponse::Ok().json(ApiResponse::success(AuthUserPayload {
        username: admin.username,
        is_default: false,
    }))
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
    if req.task_name.trim().is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::error("task_name 不能为空".to_string()));
    }

    if req.items.is_empty() {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::error("items 不能为空".to_string()));
    }

    // 从全局 ACCOUNTS 中按 email 找到已认证的 AccountStore
    let accounts = ACCOUNTS.read().await;

    let mut batch_items: Vec<BatchItem<AccountStore>> = Vec::with_capacity(req.items.len());

    for item in &req.items {
        let account = accounts
            .values()
            .find(|acc| acc.account_email == item.account_email)
            .cloned();

        let account = match account {
            Some(a) => a,
            None => {
                return HttpResponse::BadRequest().json(ApiResponse::<String>::error(format!(
                    "账号未登录或不存在: {}",
                    item.account_email
                )));
            }
        };

        if account.auth_info.is_none() {
            return HttpResponse::BadRequest().json(ApiResponse::<String>::error(format!(
                "账号尚未完成认证: {}",
                item.account_email
            )));
        }

        batch_items.push(BatchItem {
            store: account,
            app_id: item.app_id.clone(),
            app_name: item.app_name.clone(),
            // 这里的 version 实际是 appVerId（external_identifier），用于 download_product 的 app_ver_id 参数
            version: item.version.clone(),
            account_email: item.account_email.clone(),
        });
    }

    drop(accounts);

    match data
        .download_manager
        .start_batch_download::<AccountStore>(&req.task_name, batch_items)
        .await
    {
        Ok(batch_id) => HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
            "batchId": batch_id,
            "taskName": req.task_name,
            "totalCount": req.items.len(),
        }))),
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("创建批量任务失败: {}", e))),
    }
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
        download_manager: download_manager.clone(),
    });

    let bind_address = "0.0.0.0:8080";
    log::info!("Starting server at {}", bind_address);

    HttpServer::new(move || {
        App::new()
            .app_data(web::JsonConfig::default().limit(4096))
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    // 公开路由：管理员认证
                    .service(
                        web::scope("/auth")
                            .route("/login", web::post().to(admin_login))
                            .route("/logout", web::post().to(logout))
                            .route("/me", web::get().to(me))
                            .route("/change-password", web::post().to(change_password)),
                    )
                    // 需要管理员认证的路由
                    .service(
                        web::scope("")
                            .wrap(from_fn(require_auth))
                            .route("/health", web::get().to(health))
                            .route("/login", web::post().to(apple_login))
                            .route("/accounts", web::get().to(get_account_list))
                            .route("/accounts/{token}", web::delete().to(delete_account))
                            .route("/credentials", web::get().to(get_credentials_list))
                            .route("/auto-login", web::post().to(auto_login_all))
                            .route("/login/refresh", web::post().to(refresh_login))
                            .route("/versions", web::get().to(get_versions))
                            .route("/download-url", web::get().to(get_download_url))
                            .route("/download", web::post().to(download_ipa))
                            .route("/search", web::get().to(search_app))
                            .route("/manifest", web::get().to(get_manifest))
                            .route("/install", web::get().to(install))
                            .route("/batch-download", web::post().to(start_batch_download))
                            .route("/batch-tasks", web::get().to(get_batch_tasks))
                            .route("/batch-tasks/{id}", web::get().to(get_batch_task))
                            .route("/batch-tasks/{id}", web::delete().to(delete_batch_task))
                            .route("/download-records", web::get().to(get_download_records))
                            .route("/download-records/{id}", web::delete().to(delete_download_record))
                            .route("/subscriptions", web::get().to(get_subscriptions))
                            .route("/subscriptions", web::post().to(add_subscription))
                            .route("/subscriptions", web::delete().to(remove_subscription))
                            .route("/check-updates", web::get().to(check_updates)),
                    ),
            )
            // 托管前端静态文件
            .service(fs::Files::new("/", "../dist").index_file("index.html"))
    })
    .bind(bind_address)?
    .run()
    .await
}
