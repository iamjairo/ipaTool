use actix_files as fs;
use actix_web::{
    body::{EitherBody, MessageBody},
    cookie::{time::Duration as CookieDuration, Cookie, SameSite},
    dev::{ServiceRequest, ServiceResponse},
    error::{ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized},
    http::header::{ContentDisposition, DispositionParam, DispositionType},
    middleware::{from_fn, Next},
    web, App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer, Responder,
};
use bytes::Bytes;
use chrono::{Duration, Utc};
use futures_util::{
    future::{ready, Ready},
    stream, StreamExt,
};
use ipa_webtool_services::{
    download_ipa_with_account, generate_mobileconfig, generate_plist, get_license_error_message,
    AccountStore, AdminUser, BatchItem, Database, DownloadManager, DownloadParams, InstallQuery,
    JobEndEvent, JobEvent, JobLogEvent, JobProgressEvent, JobProgressPayload, JobState, JobStore,
    NewSubscription,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
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
#[allow(non_snake_case)]
struct StartDownloadDirectRequest {
    token: String,
    appid: String,
    appVerId: Option<String>,
    #[serde(default)]
    autoPurchase: bool,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct JobIdQuery {
    jobId: String,
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
    new_username: Option<String>,
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
    #[allow(dead_code)]
    session_token: String,
}

#[derive(Deserialize, Default)]
#[allow(non_snake_case)]
struct ManifestQuery {
    url: Option<String>,
    bundle_id: Option<String>,
    bundle_version: Option<String>,
    title: Option<String>,
    jobId: Option<String>,
}

// 应用状态
struct AppState {
    db: Arc<Mutex<Database>>,
    download_manager: Arc<DownloadManager>,
    job_store: JobStore,
}

// 模拟的账号存储（生产环境应该使用数据库）
lazy_static::lazy_static! {
    static ref ACCOUNTS: RwLock<HashMap<String, AccountStore>> = RwLock::new(HashMap::new());
    // MFA 第一轮失败后暂存 AccountStore（保留 GUID），等待用户提交验证码后复用
    static ref PENDING_MFA: RwLock<HashMap<String, AccountStore>> = RwLock::new(HashMap::new());
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
    HttpResponse::Unauthorized().json(ApiResponse::<String>::error(
        "未登录或登录已过期".to_string(),
    ))
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
            .into_response(
                HttpResponse::Unauthorized().json(ApiResponse::<String>::error(error_message)),
            )
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

fn build_base_url(req: &HttpRequest) -> String {
    let info = req.connection_info();
    format!("{}://{}", info.scheme(), info.host())
}

fn build_job_manifest_url(req: &HttpRequest, job_id: &str) -> String {
    format!(
        "{}/api/manifest?jobId={}",
        build_base_url(req),
        urlencoding::encode(job_id)
    )
}

fn build_job_install_url(req: &HttpRequest, job_id: &str) -> String {
    let manifest_url = build_job_manifest_url(req, job_id);
    format!(
        "{}/api/install?manifest={}",
        build_base_url(req),
        urlencoding::encode(&manifest_url)
    )
}

fn encode_sse<T: Serialize>(event_name: &str, payload: &T) -> Result<Bytes, Error> {
    let payload = serde_json::to_string(payload).map_err(ErrorInternalServerError)?;
    Ok(Bytes::from(format!(
        "event: {}\ndata: {}\n\n",
        event_name, payload
    )))
}

fn encode_job_event(event: JobEvent) -> Result<Bytes, Error> {
    match event {
        JobEvent::Progress(payload) => encode_sse("progress", &payload),
        JobEvent::Log(payload) => encode_sse("log", &payload),
        JobEvent::End(payload) => encode_sse("end", &payload),
    }
}

fn snapshot_progress_event(snapshot: &JobState) -> JobProgressEvent {
    JobProgressEvent {
        status: Some(snapshot.status.clone()),
        progress: Some(JobProgressPayload {
            stage: snapshot.stage.clone(),
            percent: snapshot.progress,
            downloaded: None,
            total: None,
            message: None,
        }),
        error: snapshot.error.clone(),
    }
}

async fn start_download_direct(
    req: web::Json<StartDownloadDirectRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let accounts = ACCOUNTS.read().await;
    let account_store = match accounts.get(&req.token) {
        Some(account) => account.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<String>::error("无效的 token".to_string()))
        }
    };
    drop(accounts);

    if !req.autoPurchase {
        match account_store
            .download_product(&req.appid, req.appVerId.as_deref())
            .await
        {
            Ok(result) => {
                let state = result
                    .get("_state")
                    .and_then(|value| value.as_str())
                    .unwrap_or("failure");

                if state != "success" {
                    let error_message = result
                        .get("customerMessage")
                        .or(result.get("failureType"))
                        .or(result.get("message"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("下载失败")
                        .to_string();

                    let is_license_error = error_message.to_lowercase().contains("license")
                        || error_message.to_lowercase().contains("not found")
                        || error_message.contains("未购买")
                        || error_message.contains("未找到");

                    if is_license_error {
                        return HttpResponse::BadRequest().json(serde_json::json!({
                            "ok": false,
                            "needsPurchase": true,
                            "error": get_license_error_message(&result),
                        }));
                    }

                    return HttpResponse::BadRequest()
                        .json(ApiResponse::<String>::error(error_message));
                }
            }
            Err(error) => {
                return HttpResponse::InternalServerError().json(ApiResponse::<String>::error(
                    format!("创建任务失败: {}", error),
                ))
            }
        }
    }

    let job_id = Uuid::new_v4().to_string();
    let job = data.job_store.create_job(job_id.clone()).await;
    job.append_log(format!("[job] 已创建任务 {}", job_id)).await;

    let appid = req.appid.clone();
    let app_ver_id = req.appVerId.clone();
    let auto_purchase = req.autoPurchase;
    let account_email = account_store.account_email.clone();
    let job_for_task = job.clone();
    let job_id_for_task = job_id.clone();

    tokio::spawn(async move {
        let job_dir = format!("../downloads/jobs/{}", job_id_for_task);
        if let Err(error) = tokio::fs::create_dir_all(&job_dir).await {
            let message = format!("创建任务目录失败: {}", error);
            job_for_task
                .append_log(format!("[error] {}", message))
                .await;
            job_for_task.mark_failed(message).await;
            return;
        }

        job_for_task.set_running().await;
        job_for_task
            .append_log("[job] 开始下载任务".to_string())
            .await;

        let progress_job = job_for_task.clone();
        let progress_callback =
            std::sync::Arc::new(move |progress: ipa_webtool_services::DownloadProgress| {
                let progress_job = progress_job.clone();
                tokio::spawn(async move {
                    progress_job.append_log(progress.message.clone()).await;
                    progress_job.update_from_progress(&progress).await;
                });
            });

        let params = DownloadParams {
            store: &account_store,
            email: &account_email,
            appid: &appid,
            app_ver_id: app_ver_id.as_deref(),
            download_path: &job_dir,
            auto_purchase,
            token: None,
            progress_callback: Some(progress_callback),
        };

        match download_ipa_with_account(params).await {
            Ok(result) if result.ok => {
                if let Some(file_path) = result.file {
                    job_for_task
                        .append_log(format!("[ready] 文件已就绪：{}", file_path))
                        .await;
                    job_for_task
                        .mark_ready(file_path, result.metadata, None)
                        .await;
                } else {
                    let message = "下载完成，但未找到产物文件".to_string();
                    job_for_task
                        .append_log(format!("[error] {}", message))
                        .await;
                    job_for_task.mark_failed(message).await;
                }
            }
            Ok(result) => {
                let message = result.error.unwrap_or_else(|| "下载失败".to_string());
                job_for_task
                    .append_log(format!("[error] {}", message))
                    .await;
                job_for_task.mark_failed(message).await;
            }
            Err(error) => {
                let message = error.to_string();
                job_for_task
                    .append_log(format!("[error] {}", message))
                    .await;
                job_for_task.mark_failed(message).await;
            }
        }
    });

    HttpResponse::Ok().json(serde_json::json!({
        "ok": true,
        "jobId": job_id,
    }))
}

async fn progress_sse(
    query: web::Query<JobIdQuery>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let job = data
        .job_store
        .get(&query.jobId)
        .await
        .ok_or_else(|| ErrorNotFound("任务不存在"))?;

    let snapshot = job.snapshot().await;
    let mut initial_events: Vec<Result<Bytes, Error>> = Vec::new();

    for line in &snapshot.logs {
        initial_events.push(encode_sse("log", &JobLogEvent { line: line.clone() }));
    }

    initial_events.push(encode_sse("progress", &snapshot_progress_event(&snapshot)));

    if snapshot.status == "ready" || snapshot.status == "failed" {
        initial_events.push(encode_sse(
            "end",
            &JobEndEvent {
                status: snapshot.status.clone(),
                error: snapshot.error.clone(),
            },
        ));

        return Ok(HttpResponse::Ok()
            .insert_header(("Content-Type", "text/event-stream"))
            .insert_header(("Cache-Control", "no-cache"))
            .insert_header(("X-Accel-Buffering", "no"))
            .streaming(stream::iter(initial_events)));
    }

    let receiver = job.subscribe();
    let live_stream = stream::unfold(receiver, |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(event) => return Some((encode_job_event(event), receiver)),
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    });

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(stream::iter(initial_events).chain(live_stream)))
}

async fn download_file(
    query: web::Query<JobIdQuery>,
    data: web::Data<AppState>,
) -> Result<fs::NamedFile, Error> {
    let job = data
        .job_store
        .get(&query.jobId)
        .await
        .ok_or_else(|| ErrorNotFound("任务不存在"))?;
    let snapshot = job.snapshot().await;

    if snapshot.status != "ready" {
        return Err(ErrorNotFound("任务尚未就绪"));
    }

    let file_path = snapshot
        .file_path
        .clone()
        .ok_or_else(|| ErrorNotFound("下载文件不存在"))?;
    let path = PathBuf::from(&file_path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download.ipa")
        .to_string();

    let file = fs::NamedFile::open_async(path)
        .await
        .map_err(ErrorNotFound)?
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(file_name)],
        });

    Ok(file)
}

async fn get_job_info(
    req: HttpRequest,
    query: web::Query<JobIdQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    let job = match data.job_store.get(&query.jobId).await {
        Some(job) => job,
        None => {
            return HttpResponse::NotFound()
                .json(ApiResponse::<String>::error("任务不存在".to_string()))
        }
    };

    let snapshot = job.snapshot().await;
    let install_url = if snapshot.status == "ready" {
        Some(build_job_install_url(&req, &query.jobId))
    } else {
        snapshot.install_url.clone()
    };

    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "jobId": snapshot.job_id,
        "status": snapshot.status,
        "stage": snapshot.stage,
        "progress": snapshot.progress,
        "installUrl": install_url,
        "error": snapshot.error,
        "metadata": snapshot.metadata,
        "filePath": snapshot.file_path,
    })))
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
    let has_mfa = req.mfa.is_some();
    log::info!(
        "Apple login attempt: email={}, has_mfa={}, mfa_len={}",
        req.email,
        has_mfa,
        req.mfa.as_ref().map(|s| s.len()).unwrap_or(0)
    );

    // 如果有 MFA code，优先复用第一轮暂存的 AccountStore（保留 GUID）
    // 否则创建新的
    let mut account_store = if has_mfa {
        let mut pending = PENDING_MFA.write().await;
        match pending.remove(&req.email) {
            Some(store) => {
                log::info!("Reusing pending MFA session for {}", req.email);
                store
            }
            None => {
                log::warn!("No pending MFA session found for {}, creating fresh (GUID mismatch risk)", req.email);
                AccountStore::new(&req.email)
            }
        }
    } else {
        AccountStore::new(&req.email)
    };

    match account_store
        .authenticate(&req.password, req.mfa.as_deref())
        .await
    {
        Ok(result) => {
            let state = result
                .get("_state")
                .and_then(|v| v.as_str())
                .unwrap_or("failure");

            log::info!(
                "Apple auth result: state={}, keys={:?}",
                state,
                result.keys().take(10).collect::<Vec<_>>()
            );

            if state == "success" {
                // 清理可能残留的 pending MFA 条目
                {
                    let mut pending = PENDING_MFA.write().await;
                    pending.remove(&req.email);
                }

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
                        if let Ok(enc_key) =
                            ipa_webtool_services::crypto::ensure_encryption_key(&db)
                        {
                            if let Ok((ct, iv, tag)) =
                                ipa_webtool_services::crypto::encrypt(&req.password, &enc_key)
                            {
                                let key_id = db
                                    .get_current_encryption_key()
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

                log::info!("Apple login success: email={}, dsid={}", req.email, dsid);

                HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                    "token": token,
                    "email": req.email,
                    "dsid": dsid,
                    "displayName": result.get("displayName"),
                })))
            } else {
                // 检查是否是 MFA 要求
                let failure_type = result
                    .get("failureType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let error_msg = result
                    .get("customerMessage")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| {
                        if failure_type.is_empty() {
                            "登录失败，Apple 未返回具体错误信息"
                        } else {
                            failure_type
                        }
                    });

                // Enhanced MFA detection — check both failureType and customerMessage
                let needs_mfa = error_msg.to_lowercase().contains("verification code")
                    || error_msg.to_lowercase().contains("two-factor")
                    || error_msg.to_lowercase().contains("mfa")
                    || error_msg.to_lowercase().contains("2fa")
                    || error_msg.to_lowercase().contains("two-step")
                    || error_msg.to_lowercase().contains("trusted device")
                    || error_msg.contains("二次验证")
                    || error_msg.contains("验证码")
                    || failure_type.contains("MFA")
                    || failure_type.contains("-219")  // Apple MFA error code
                    || failure_type.contains("verificationCode");

                // Translate Apple's cryptic customerMessage keys to readable text
                let user_facing_msg = match error_msg {
                    "MZFinance.BadLogin.Configurator_message" => "账号或密码错误，请检查后重试",
                    "MZFinance.BadLogin.Configurator.message" => "账号或密码错误，请检查后重试",
                    m if m.starts_with("MZFinance.BadLogin") => "账号或密码错误，请检查后重试",
                    m if m.contains("account.locked") || m.contains("account disabled") => "账号已被锁定或停用",
                    m if m.contains("rate.limit") || m.contains("too many") => "登录尝试过于频繁，请稍后再试",
                    _ => error_msg,
                };

                log::warn!(
                    "Apple auth failure: failureType='{}', msg='{}', needs_mfa={}, has_mfa={}",
                    failure_type, error_msg, needs_mfa, has_mfa
                );

                if needs_mfa && !has_mfa {
                    // 第一轮：暂存 AccountStore 保留 GUID，等用户提交验证码
                    {
                        let mut pending = PENDING_MFA.write().await;
                        pending.insert(req.email.clone(), account_store);
                    }

                    log::info!("Saved pending MFA session for {}", req.email);

                    return HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                        "status": "need_mfa",
                        "message": "此账号需要二次验证，请在验证码输入框输入 6 位验证码后再次点击登录",
                    })));
                }

                // MFA code provided but still got MFA-related error — code may be wrong/expired
                if needs_mfa && has_mfa {
                    // Re-save AccountStore for retry
                    {
                        let mut pending = PENDING_MFA.write().await;
                        pending.insert(req.email.clone(), account_store);
                    }

                    log::warn!("MFA code rejected for {}", req.email);

                    return HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                        "status": "mfa_failed",
                        "message": "验证码无效或已过期，请检查后重新输入",
                    })));
                }

                // Generic auth failure — still save session for potential MFA retry
                // Apple sometimes returns BadLogin even when MFA is needed,
                // so always preserve the AccountStore (GUID) for a second attempt
                if !has_mfa {
                    let mut pending = PENDING_MFA.write().await;
                    pending.insert(req.email.clone(), account_store);
                    log::info!("Saved session for {} after generic failure (GUID preserved for MFA retry)", req.email);
                }

                log::error!("Apple auth failed for {}: {}", req.email, user_facing_msg);
                HttpResponse::Ok().json(ApiResponse::<String>::error(user_facing_msg.to_string()))
            }
        }
        Err(e) => {
            let err_msg = e.to_string();
            log::error!("Apple auth exception for {}: {}", req.email, err_msg);

            // 如果是 JSON 解析错误，说明 Apple 返回了非 JSON 响应
            if err_msg.contains("error decoding response body")
                || err_msg.contains("expected value")
            {
                HttpResponse::Ok().json(ApiResponse::<String>::error(
                    "登录请求被 Apple 拒绝，请检查网络、账号密码，或者尝试使用应用专用密码登录".to_string()
                ))
            } else if err_msg.contains("timed out") || err_msg.contains("deadline") {
                HttpResponse::Ok().json(ApiResponse::<String>::error(
                    "连接 Apple 超时，请检查网络环境".to_string()
                ))
            } else {
                HttpResponse::Ok().json(ApiResponse::<String>::error(format!(
                    "登录失败: {}", err_msg
                )))
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
            let tokens_set: std::collections::HashSet<_> = accounts.keys().cloned().collect();
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
async fn delete_account(token: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let token = token.into_inner();

    // 从内存删除
    let mut accounts = ACCOUNTS.write().await;
    let removed_account = accounts.remove(&token);
    let email = removed_account.as_ref().map(|a| a.account_email.clone());

    // 从 DB 删除
    if let Ok(db) = data.db.lock() {
        let _ = db.delete_account(&token);
        // 同时删除该 email 的凭证
        if let Some(email) = email {
            let _ = db.delete_credentials(&email);
        }
    }

    if removed_account.is_some() {
        HttpResponse::Ok().json(ApiResponse::success("已删除"))
    } else {
        HttpResponse::Ok().json(ApiResponse::success("已删除（仅数据库记录）"))
    }
}

// 获取已保存的凭证邮箱列表（不返回密码）
async fn get_credentials_list(data: web::Data<AppState>) -> impl Responder {
    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<String>::error("数据库不可用".to_string()))
        }
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
    let (credentials, enc_key) = {
        let db = match data.db.lock() {
            Ok(db) => db,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error("数据库不可用".to_string()))
            }
        };

        let credentials = match db.get_all_credentials() {
            Ok(c) => c,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error(format!("获取凭证失败: {}", e)))
            }
        };

        let enc_key = match ipa_webtool_services::crypto::ensure_encryption_key(&db) {
            Ok(k) => k,
            Err(e) => {
                return HttpResponse::InternalServerError().json(ApiResponse::<String>::error(
                    format!("加密密钥初始化失败: {}", e),
                ))
            }
        };

        (credentials, enc_key)
    };

    let mut success = Vec::new();
    let mut need_code = Vec::new();
    let mut failed = Vec::new();

    let accounts = ACCOUNTS.read().await;
    // 收集已登录的邮箱列表
    let logged_in_emails: std::collections::HashSet<String> =
        accounts.values().map(|a| a.account_email.clone()).collect();
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
                let state = result
                    .get("_state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("failure");
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

                    if err_msg.contains("verification code")
                        || err_msg.contains("two-factor")
                        || err_msg.contains("MFA")
                    {
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
        None => {
            return HttpResponse::BadRequest()
                .json(ApiResponse::<String>::error("缺少 token".to_string()))
        }
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

    let password = {
        let db = match data.db.lock() {
            Ok(db) => db,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error("数据库不可用".to_string()))
            }
        };

        let cred = match db.get_credentials(&email) {
            Ok(Some(c)) => c,
            _ => {
                return HttpResponse::BadRequest().json(ApiResponse::<String>::error(
                    "未找到保存的密码，无法自动刷新。请重新登录。".to_string(),
                ))
            }
        };

        let enc_key = match ipa_webtool_services::crypto::ensure_encryption_key(&db) {
            Ok(k) => k,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error(format!("加密密钥失败: {}", e)))
            }
        };

        match ipa_webtool_services::crypto::decrypt(
            &cred.password_encrypted,
            &cred.iv,
            &cred.auth_tag,
            &enc_key,
        ) {
            Ok(password) => password,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error("解密密码失败".to_string()))
            }
        }
    };

    // 重新认证
    let mut store = AccountStore::new(&email);
    match store.authenticate(&password, None).await {
        Ok(result) => {
            let state = result
                .get("_state")
                .and_then(|v| v.as_str())
                .unwrap_or("failure");
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
        Err(e) => HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("刷新失败: {}", e))),
    }
}

async fn admin_login(
    req: web::Json<AdminLoginRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let username = req.username.trim();
    let password = req.password.trim();

    if username.is_empty() || password.is_empty() {
        return HttpResponse::BadRequest().json(ApiResponse::<String>::error(
            "用户名和密码不能为空".to_string(),
        ));
    }

    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<String>::error(
                "认证服务暂时不可用".to_string(),
            ))
        }
    };

    let user = match db.get_admin_user(username) {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<String>::error("用户名或密码错误".to_string()))
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
                "查询管理员失败: {}",
                e
            )))
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
        return HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
            "创建登录态失败: {}",
            e
        )));
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
        return HttpResponse::BadRequest().json(ApiResponse::<String>::error(
            "新密码不能与当前密码相同".to_string(),
        ));
    }

    let db = match data.db.lock() {
        Ok(db) => db,
        Err(_) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<String>::error(
                "认证服务暂时不可用".to_string(),
            ))
        }
    };

    let user = match db.get_admin_user(&admin.username) {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .json(ApiResponse::<String>::error("管理员账号不存在".to_string()))
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
                "查询管理员失败: {}",
                e
            )))
        }
    };

    if user.password_hash != hash_password(&req.current_password) {
        return HttpResponse::BadRequest()
            .json(ApiResponse::<String>::error("当前密码不正确".to_string()));
    }

    if let Err(e) =
        db.update_admin_password(&admin.username, &hash_password(&req.new_password), false)
    {
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<String>::error(format!("修改密码失败: {}", e)));
    }

    let final_username = if let Some(new_name) = &req.new_username {
        let trimmed = new_name.trim();
        if !trimmed.is_empty() && trimmed != admin.username {
            if let Err(e) = db.rename_admin_user(&admin.username, trimmed) {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error(format!("修改用户名失败: {}", e)));
            }
            trimmed.to_string()
        } else {
            admin.username.clone()
        }
    } else {
        admin.username.clone()
    };

    HttpResponse::Ok().json(ApiResponse::success(AuthUserPayload {
        username: final_username,
        is_default: false,
    }))
}

// 生成 plist 清单文件
async fn get_manifest(
    req: HttpRequest,
    query: web::Query<ManifestQuery>,
    data: web::Data<AppState>,
) -> impl Responder {
    let manifest_result = if let Some(job_id) = &query.jobId {
        let job = match data.job_store.get(job_id).await {
            Some(job) => job,
            None => {
                return HttpResponse::NotFound()
                    .json(ApiResponse::<String>::error("任务不存在".to_string()))
            }
        };
        let snapshot = job.snapshot().await;

        if snapshot.status != "ready" {
            return HttpResponse::BadRequest().json(ApiResponse::<String>::error(
                "任务尚未完成，无法生成 manifest".to_string(),
            ));
        }

        let metadata = match snapshot.metadata {
            Some(metadata) => metadata,
            None => {
                return HttpResponse::InternalServerError()
                    .json(ApiResponse::<String>::error("任务缺少元数据".to_string()))
            }
        };

        let download_url = format!(
            "{}/api/download-file?jobId={}",
            build_base_url(&req),
            urlencoding::encode(job_id)
        );

        generate_plist(
            download_url,
            metadata.bundle_id,
            metadata.bundle_short_version_string,
            metadata.bundle_display_name,
        )
    } else {
        let url = match &query.url {
            Some(url) => url.clone(),
            None => {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::<String>::error("url 不能为空".to_string()))
            }
        };
        let bundle_id = match &query.bundle_id {
            Some(bundle_id) => bundle_id.clone(),
            None => {
                return HttpResponse::BadRequest().json(ApiResponse::<String>::error(
                    "bundle_id 不能为空".to_string(),
                ))
            }
        };
        let bundle_version = match &query.bundle_version {
            Some(bundle_version) => bundle_version.clone(),
            None => {
                return HttpResponse::BadRequest().json(ApiResponse::<String>::error(
                    "bundle_version 不能为空".to_string(),
                ))
            }
        };
        let title = match &query.title {
            Some(title) => title.clone(),
            None => {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::<String>::error("title 不能为空".to_string()))
            }
        };

        generate_plist(url, bundle_id, bundle_version, title)
    };

    match manifest_result {
        Ok(plist) => HttpResponse::Ok()
            .content_type("application/x-plist")
            .insert_header((
                "Content-Disposition",
                "attachment; filename=\"manifest.plist\"",
            ))
            .body(plist),
        Err(error) => {
            log::error!("Failed to generate plist: {}", error);
            HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
                "生成 plist 失败: {}",
                error
            )))
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
                .insert_header((
                    "Content-Disposition",
                    "attachment; filename=\"install.mobileconfig\"",
                ))
                .body(mobileconfig)
        }
        Err(e) => {
            log::error!("Failed to generate mobileconfig: {}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
                "生成安装描述文件失败: {}",
                e
            )))
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
        return HttpResponse::BadRequest().json(ApiResponse::<String>::error(
            "task_name 不能为空".to_string(),
        ));
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
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
            "创建批量任务失败: {}",
            e
        ))),
    }
}

// 获取所有批量下载任务
async fn get_batch_tasks(data: web::Data<AppState>) -> impl Responder {
    match data.db.lock().unwrap().get_batch_tasks() {
        Ok(tasks) => HttpResponse::Ok().json(ApiResponse::success(tasks)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
            "获取批量任务失败: {}",
            e
        ))),
    }
}

// 获取单个批量下载任务详情
async fn get_batch_task(path: web::Path<i64>, data: web::Data<AppState>) -> impl Responder {
    let batch_id = path.into_inner();

    // 获取任务信息
    let task = match data.db.lock().unwrap().get_batch_tasks() {
        Ok(tasks) => tasks.into_iter().find(|t| t.id == Some(batch_id)),
        Err(e) => {
            return HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
                "获取批量任务失败: {}",
                e
            )))
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
            return HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
                "获取批量任务项失败: {}",
                e
            )))
        }
    };

    HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "task": task,
        "items": items
    })))
}

// 删除批量下载任务
async fn delete_batch_task(path: web::Path<i64>, data: web::Data<AppState>) -> impl Responder {
    let batch_id = path.into_inner();

    match data.db.lock().unwrap().delete_batch_task(batch_id) {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success("批量任务已删除".to_string())),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
            "删除批量任务失败: {}",
            e
        ))),
    }
}

// ============ 下载记录端点 ============

// 获取所有下载记录
async fn get_download_records(data: web::Data<AppState>) -> impl Responder {
    match data.db.lock().unwrap().get_all_download_records() {
        Ok(records) => HttpResponse::Ok().json(ApiResponse::success(records)),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<String>::error(format!(
            "获取下载记录失败: {}",
            e
        ))),
    }
}

// 删除下载记录
async fn delete_download_record(path: web::Path<i64>, data: web::Data<AppState>) -> impl Responder {
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
    let subscription = NewSubscription {
        app_id: &req.app_id,
        app_name: &req.app_name,
        bundle_id: req.bundle_id.as_deref(),
        account_email: &req.account_email,
        account_region: req.account_region.as_deref(),
        artwork_url: req.artwork_url.as_deref(),
        artist_name: req.artist_name.as_deref(),
    };

    match data.db.lock().unwrap().add_subscription(&subscription) {
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
        }
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
        job_store: JobStore::new(),
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
                            .route("/start-download-direct", web::post().to(start_download_direct))
                            .route("/progress-sse", web::get().to(progress_sse))
                            .route("/download-file", web::get().to(download_file))
                            .route("/job-info", web::get().to(get_job_info))
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
            .service(fs::Files::new("/", "./dist").index_file("index.html"))
    })
    .bind(bind_address)?
    .run()
    .await
}
