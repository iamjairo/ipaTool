pub mod apple_auth;
pub mod database;
pub mod ipa_handler;
pub mod key_manager;
pub mod signature;
pub mod ota_install;
pub mod download_manager;

pub use apple_auth::{AccountStore, AuthInfo, Store};
pub use database::{Database, Subscription, BatchDownloadTask, BatchDownloadItem};
pub use ipa_handler::{
    download_ipa_with_account, get_license_error_message, DownloadMetadata, DownloadProgress,
    DownloadResult,
};
pub use key_manager::KeyManager;
pub use signature::{read_zip, SignatureClient};
pub use ota_install::{generate_plist, generate_mobileconfig, InstallQuery};
pub use download_manager::{DownloadManager, AppUpdate};
