# Commit Checklist for fix/download-flow-followups

## Backend (server/)

### database.rs
- [x] Added `get_latest_account_region_by_email(&self, email) -> Result<Option<String>>`
  - Selects the most recent account region for a given email, ORDER BY id DESC
- [x] Added `update_account_region(&self, token: &str, region: &str) -> Result<()>`
  - Updates account region by token
  - Uses prepared statement with CURRENT_TIMESTAMP for updated_at

### main.rs
- [x] Added `normalize_region_code(region: &str) -> Option<String>`
  - Normalizes region codes (uppercase, 2-3 chars)
- [x] Added `UpdateAccountRegionRequest` struct with region field
- [x] Added `update_account_region()` handler
  - PUT /accounts/{token}/region endpoint
  - Validates region code, calls db.update_account_region
- [x] Modified `get_account_list()` to read DB for all account regions
  - Creates HashMap<String, String> from db.get_all_accounts()
  - Returns persisted region instead of hardcoded "US"
- [x] Modified `apple_login()` to preserve existing region
  - Uses db.get_latest_account_region_by_email before creating account
  - Returns region in response JSON
- [x] Modified `auto_login_all()` to preserve existing region
  - Same region preservation logic as apple_login
  - Returns region in success response objects
- [x] Added `sanitize_upload_filename(name: &str) -> String`
  - Removes path separators, keeps base filename
  - Replaces unsafe characters with underscores
- [x] Added `upload_ipa()` multipart handler
  - Max size: 2GB
  - Stores in downloads/uploads/{job_id}/{filename}
  - Creates download_record with account_email="手动上传"
  - Returns jobId, fileName, fileSize, filePath, installUrl=null
- [x] Modified `start_download_direct()` to use account_region from DB
  - Queries db.get_account_by_token to get account region
  - Passes account_region to DownloadRecord
- [x] Added route `/accounts/{token}/region` (PUT)
- [x] Added route `/upload-ipa` (POST)
- [x] Fixed Chinese string concatenation errors in error messages

## Frontend (src/components/)

### AccountManager.vue
- [x] Added `regionOptions` constant with all supported regions
- [x] Modified account display to show region editor
  - Added `account-region-row` div wrapper
  - Added `el-select` dropdown for region selection
- [x] Added `updateAccountRegion(index, region)` function
  - Calls PUT /accounts/{token}/region
  - Shows success/error messages
  - Handles optimistic UI updates with rollback on error
- [x] Modified `autoLoginAll()` to reload accounts after region update

### DownloadManager.vue
- [x] Removed entire "Upload IPA Section" template block (lines ~398-513)
- [x] Removed `uploadUrl`, `uploading`, `uploadProgress`, `uploadResult` refs
- [x] Removed all upload-related functions:
  - beforeUpload
  - handleUploadProgress
  - handleUploadSuccess
  - handleUploadError
  - installUploadedIpa
- [x] Changed `goToAccountTab()` to route to 'settings' instead of 'account'
- [x] Removed `UploadFilled` from icon imports

### IpaManager.vue
- [x] Added upload entry card with title
- [x] Added `el-upload` component with drag-drop support
- [x] Added upload progress indicator
- [x] Added `uploadResult` ref for tracking upload status
- [x] Added upload-related functions:
  - beforeUpload: validates .ipa extension and 2GB limit
  - handleUploadProgress: updates progress bar
  - handleUploadSuccess: sets result, shows message, refreshes list
  - handleUploadError: shows error message
- [x] Added `UploadFilled` icon import
- [x] Added `uploadUrl` constant pointing to /api/upload-ipa
- [x] Added `uploading`, `uploadProgress` refs

## Dependencies (server/Cargo.toml)
- [x] Added `actix-multipart = "0.6"`
- [x] Verified all other dependencies unchanged

## Build Status
- [x] Backend compiles successfully (dev profile, unoptimized + debuginfo)
  - Build time: 1m 53s
  - No compilation errors

## Testing Notes
- Region preservation should work for existing accounts in DB
- New logins default to US if no prior region exists
- Upload stores files in downloads/uploads/{job_id}/
- Upload UI is now in IPA tab under "设置" page
