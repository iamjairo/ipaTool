pub mod apple_auth;
pub mod crypto;
pub mod database;
pub mod download_manager;
pub mod ipa_handler;
pub mod key_manager;
pub mod ota_install;
pub mod signature;
pub mod web_jobs;

pub use apple_auth::{AccountStore, AuthInfo, Store};
pub use database::{
    Account, AdminUser, BatchDownloadItem, BatchDownloadTask, Credentials, Database, DownloadRecord,
    EncryptionKey, NewSubscription, SessionRecord, Subscription,
};

pub use download_manager::{AppUpdate, BatchItem, DownloadManager};
pub use ipa_handler::{
    download_ipa_with_account, get_license_error_message, DownloadMetadata, DownloadParams,
    DownloadProgress, DownloadResult,
};
pub use key_manager::KeyManager;
pub use ota_install::{generate_mobileconfig, generate_plist, InstallQuery};
pub use signature::{read_zip, SignatureClient};

pub use web_jobs::{
    JobEndEvent, JobEvent, JobHandle, JobLogEvent, JobProgressEvent, JobProgressPayload, JobState,
    JobStore,
};
