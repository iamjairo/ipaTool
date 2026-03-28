use crate::{DownloadMetadata, DownloadProgress};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgressPayload {
    pub stage: String,
    pub percent: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloaded: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobState {
    pub job_id: String,
    pub status: String,
    pub stage: String,
    pub progress: u32,
    pub logs: Vec<String>,
    pub file_path: Option<String>,
    pub metadata: Option<DownloadMetadata>,
    pub install_url: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgressEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<JobProgressPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobLogEvent {
    pub line: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobEndEvent {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum JobEvent {
    Progress(JobProgressEvent),
    Log(JobLogEvent),
    End(JobEndEvent),
}

#[derive(Clone)]
pub struct JobHandle {
    state: Arc<RwLock<JobState>>,
    tx: broadcast::Sender<JobEvent>,
}

impl JobHandle {
    pub fn new(job_id: String) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            state: Arc::new(RwLock::new(JobState {
                job_id,
                status: "queued".to_string(),
                stage: "queued".to_string(),
                progress: 0,
                logs: Vec::new(),
                file_path: None,
                metadata: None,
                install_url: None,
                error: None,
            })),
            tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<JobEvent> {
        self.tx.subscribe()
    }

    pub async fn snapshot(&self) -> JobState {
        self.state.read().await.clone()
    }

    pub async fn append_log(&self, line: impl Into<String>) {
        let line = line.into();
        {
            let mut state = self.state.write().await;
            state.logs.push(line.clone());
        }
        let _ = self.tx.send(JobEvent::Log(JobLogEvent { line }));
    }

    pub async fn set_running(&self) {
        let progress_event = {
            let mut state = self.state.write().await;
            state.status = "running".to_string();
            JobProgressEvent {
                status: Some(state.status.clone()),
                progress: Some(JobProgressPayload {
                    stage: state.stage.clone(),
                    percent: state.progress,
                    downloaded: None,
                    total: None,
                    message: None,
                }),
                error: None,
            }
        };
        let _ = self.tx.send(JobEvent::Progress(progress_event));
    }

    pub async fn update_from_progress(&self, progress: &DownloadProgress) {
        let percent = progress.progress.unwrap_or(0.0).round().clamp(0.0, 100.0) as u32;
        let progress_event = {
            let mut state = self.state.write().await;
            state.status = "running".to_string();
            state.stage = progress.phase.clone();
            if progress.progress.is_some() {
                state.progress = percent;
            }
            JobProgressEvent {
                status: None,
                progress: Some(JobProgressPayload {
                    stage: progress.phase.clone(),
                    percent: state.progress,
                    downloaded: progress.downloaded,
                    total: progress.file_size,
                    message: Some(progress.message.clone()),
                }),
                error: None,
            }
        };
        let _ = self.tx.send(JobEvent::Progress(progress_event));
    }

    pub async fn mark_ready(
        &self,
        file_path: String,
        metadata: Option<DownloadMetadata>,
        install_url: Option<String>,
    ) {
        let progress_event = {
            let mut state = self.state.write().await;
            state.status = "ready".to_string();
            state.stage = "done".to_string();
            state.progress = 100;
            state.file_path = Some(file_path);
            state.metadata = metadata;
            state.install_url = install_url;
            state.error = None;
            JobProgressEvent {
                status: Some(state.status.clone()),
                progress: Some(JobProgressPayload {
                    stage: state.stage.clone(),
                    percent: state.progress,
                    downloaded: None,
                    total: None,
                    message: Some("任务完成".to_string()),
                }),
                error: None,
            }
        };
        let _ = self.tx.send(JobEvent::Progress(progress_event));
        let _ = self.tx.send(JobEvent::End(JobEndEvent {
            status: "ready".to_string(),
            error: None,
        }));
    }

    pub async fn mark_failed(&self, error: impl Into<String>) {
        let error = error.into();
        let progress_event = {
            let mut state = self.state.write().await;
            state.status = "failed".to_string();
            state.error = Some(error.clone());
            JobProgressEvent {
                status: Some(state.status.clone()),
                progress: Some(JobProgressPayload {
                    stage: state.stage.clone(),
                    percent: state.progress,
                    downloaded: None,
                    total: None,
                    message: None,
                }),
                error: Some(error.clone()),
            }
        };
        let _ = self.tx.send(JobEvent::Progress(progress_event));
        let _ = self.tx.send(JobEvent::End(JobEndEvent {
            status: "failed".to_string(),
            error: Some(error),
        }));
    }
}

#[derive(Clone, Default)]
pub struct JobStore {
    jobs: Arc<RwLock<HashMap<String, JobHandle>>>,
}

impl JobStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_job(&self, job_id: String) -> JobHandle {
        let job = JobHandle::new(job_id.clone());
        self.jobs.write().await.insert(job_id, job.clone());
        job
    }

    pub async fn get(&self, job_id: &str) -> Option<JobHandle> {
        self.jobs.read().await.get(job_id).cloned()
    }
}
