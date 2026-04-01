use base64::Engine;
use plist::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Cursor, Read, Seek, Write};
use std::path::Path;
use zip::ZipArchive;

const MH_MAGIC: u32 = 0xfeedface;
const MH_CIGAM: u32 = 0xcefaedfe;
const MH_MAGIC_64: u32 = 0xfeedfacf;
const MH_CIGAM_64: u32 = 0xcffaedfe;
const FAT_MAGIC: u32 = 0xcafebabe;
const FAT_CIGAM: u32 = 0xbebafeca;
const FAT_MAGIC_64: u32 = 0xcafebabf;
const FAT_CIGAM_64: u32 = 0xbfbafeca;
const LC_ENCRYPTION_INFO: u32 = 0x21;
const LC_ENCRYPTION_INFO_64: u32 = 0x2c;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMetadata {
    pub bundle_display_name: Option<String>,
    pub bundle_short_version_string: Option<String>,
    pub bundle_id: Option<String>,
    pub artwork_url: Option<String>,
    pub artist_name: Option<String>,
    pub apple_id: Option<String>,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sinf {
    pub id: String,
    pub sinf: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestTargets {
    #[serde(rename = "sinfPaths")]
    pub sinf_paths: Vec<String>,
    #[serde(rename = "sinfReplicationPaths")]
    pub sinf_replication_paths: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpaInspection {
    pub has_sc_info_manifest: bool,
    pub has_embedded_mobileprovision: bool,
    pub declared_sinf_paths: Vec<String>,
    pub present_sinf_paths: Vec<String>,
    pub missing_sinf_paths: Vec<String>,
    pub encrypted_binaries: Vec<String>,
    pub direct_install_ok: bool,
    pub blocked_reason: Option<String>,
    pub recommended_action: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Default)]
pub struct SignatureApplyResult {
    pub applied_paths: Vec<String>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SignatureClient {
    archive: Vec<u8>,
    filename: String,
    metadata: SignatureMetadata,
    signatures: Vec<Sinf>,
    email: String,
}

fn normalize_sinf_id(value: &serde_json::Value, fallback_index: usize) -> String {
    value
        .as_i64()
        .map(|v| v.to_string())
        .or_else(|| value.as_u64().map(|v| v.to_string()))
        .or_else(|| value.as_str().map(|v| v.to_string()))
        .unwrap_or_else(|| fallback_index.to_string())
}

fn parse_sinf_entry(value: &serde_json::Value, fallback_index: usize) -> Option<Sinf> {
    let sinf = value.get("sinf")?.as_str()?.trim().to_string();
    if sinf.is_empty() {
        return None;
    }
    Some(Sinf {
        id: normalize_sinf_id(
            value.get("id").unwrap_or(&serde_json::Value::Null),
            fallback_index,
        ),
        sinf,
    })
}

fn ordered_unique_paths(paths: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut ordered = Vec::new();
    for path in paths {
        if seen.insert(path.clone()) {
            ordered.push(path);
        }
    }
    ordered
}

fn find_app_bundle_name<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    for i in 0..zip.len() {
        let zip_name = zip.by_index(i)?.name().to_string();
        if zip_name.starts_with("Payload/") && zip_name.ends_with(".app/") {
            let bundle = zip_name
                .strip_prefix("Payload/")
                .and_then(|s| s.strip_suffix('/'))
                .unwrap_or(&zip_name)
                .to_string();
            return Ok(bundle);
        }
    }
    Err("Could not find app bundle".into())
}

fn read_zip_entry_bytes<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    path: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut file = zip.by_name(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

fn read_zip_plist<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    path: &str,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let content = read_zip_entry_bytes(zip, path)?;
    Ok(plist::from_bytes(&content)?)
}

fn manifest_targets_from_value(manifest: &Value) -> ManifestTargets {
    let mut targets = ManifestTargets::default();
    if let Value::Dictionary(dict) = manifest {
        targets.sinf_paths = dict
            .get("SinfPaths")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_string().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        targets.sinf_replication_paths = dict
            .get("SinfReplicationPaths")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_string().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
    }
    targets
}

fn read_manifest_targets<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    app_bundle_name: &str,
) -> Result<Option<ManifestTargets>, Box<dyn std::error::Error + Send + Sync>> {
    let manifest_path = format!("Payload/{}/SC_Info/Manifest.plist", app_bundle_name);
    let manifest = match read_zip_plist(zip, &manifest_path) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    Ok(Some(manifest_targets_from_value(&manifest)))
}

fn decode_signatures(
    signatures: &[Sinf],
) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
    signatures
        .iter()
        .map(|signature| Ok(base64::engine::general_purpose::STANDARD.decode(&signature.sinf)?))
        .collect()
}

fn build_injection_plan(
    signatures: &[Sinf],
    manifest_targets: &ManifestTargets,
) -> Result<SignatureApplyResult, Box<dyn std::error::Error + Send + Sync>> {
    let target_paths = ordered_unique_paths(
        manifest_targets
            .sinf_paths
            .iter()
            .cloned()
            .chain(manifest_targets.sinf_replication_paths.iter().cloned()),
    );

    if target_paths.is_empty() {
        return Ok(SignatureApplyResult {
            applied_paths: Vec::new(),
            warning: Some("包内未声明需要补齐的 .sinf 目标".to_string()),
        });
    }

    if signatures.is_empty() {
        return Ok(SignatureApplyResult {
            applied_paths: Vec::new(),
            warning: Some(format!(
                "包内声明了 {} 个 .sinf 目标，但 Apple 下载响应未返回任何真实 sinf",
                target_paths.len()
            )),
        });
    }

    if signatures.len() == 1 {
        return Ok(SignatureApplyResult {
            applied_paths: target_paths,
            warning: None,
        });
    }

    if signatures.len() == target_paths.len() {
        return Ok(SignatureApplyResult {
            applied_paths: target_paths,
            warning: None,
        });
    }

    if signatures.len() == manifest_targets.sinf_paths.len()
        && !manifest_targets.sinf_replication_paths.is_empty()
    {
        return Ok(SignatureApplyResult {
            applied_paths: Vec::new(),
            warning: Some(format!(
                "Apple 返回了 {} 个 sinf，主 SinfPaths={}，但还存在 {} 个 replication 目标；当前无法安全推断一对多映射，跳过注入",
                signatures.len(),
                manifest_targets.sinf_paths.len(),
                manifest_targets.sinf_replication_paths.len()
            )),
        });
    }

    Ok(SignatureApplyResult {
        applied_paths: Vec::new(),
        warning: Some(format!(
            "Apple 返回 sinf 数量 ({}) 与包内声明目标数量 ({}) 不匹配，跳过注入",
            signatures.len(),
            target_paths.len()
        )),
    })
}

fn replace_zip_entries(
    archive: &mut Vec<u8>,
    replacements: &[(String, Vec<u8>)],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let reader = Cursor::new(archive.clone());
    let mut zip = ZipArchive::new(reader)?;
    let replacement_paths = replacements
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<HashSet<_>>();

    // IMPORTANT: write to a fresh buffer, then replace `archive`.
    // Writing into the existing Vec via Cursor can leave trailing bytes and corrupt the zip.
    let mut out = Vec::with_capacity(archive.len() + 1024);
    let mut new_archive = zip::ZipWriter::new(Cursor::new(&mut out));
    let options: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        if replacement_paths.contains(file.name()) {
            continue;
        }
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        new_archive.start_file(file.name(), options)?;
        new_archive.write_all(&buffer)?;
    }

    for (path, data) in replacements {
        new_archive.start_file(path, options)?;
        new_archive.write_all(data)?;
    }

    let _ = new_archive.finish()?;
    *archive = out;
    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize, little_endian: bool) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(if little_endian {
        u32::from_le_bytes(slice.try_into().ok()?)
    } else {
        u32::from_be_bytes(slice.try_into().ok()?)
    })
}

fn parse_macho_load_commands(bytes: &[u8], little_endian: bool, is_64: bool) -> Option<bool> {
    let header_size = if is_64 { 32 } else { 28 };
    let ncmds = read_u32(bytes, 16, little_endian)? as usize;
    let mut offset = header_size;

    for _ in 0..ncmds {
        let cmd = read_u32(bytes, offset, little_endian)?;
        let cmdsize = read_u32(bytes, offset + 4, little_endian)? as usize;
        if cmdsize < 8 || offset.checked_add(cmdsize)? > bytes.len() {
            return None;
        }
        if matches!(cmd, LC_ENCRYPTION_INFO | LC_ENCRYPTION_INFO_64) {
            let cryptid = read_u32(bytes, offset + 16, little_endian)?;
            return Some(cryptid == 1);
        }
        offset += cmdsize;
    }

    Some(false)
}

fn macho_cryptid_one(bytes: &[u8]) -> Option<bool> {
    let magic_be = read_u32(bytes, 0, false)?;
    let magic_le = read_u32(bytes, 0, true)?;

    match magic_le {
        MH_MAGIC => return parse_macho_load_commands(bytes, true, false),
        MH_MAGIC_64 => return parse_macho_load_commands(bytes, true, true),
        FAT_CIGAM => {
            let nfat_arch = read_u32(bytes, 4, true)? as usize;
            for index in 0..nfat_arch {
                let arch_offset = 8 + index * 20;
                let slice_offset = read_u32(bytes, arch_offset + 8, true)? as usize;
                let slice_size = read_u32(bytes, arch_offset + 12, true)? as usize;
                let slice = bytes.get(slice_offset..slice_offset.checked_add(slice_size)?)?;
                if macho_cryptid_one(slice)? {
                    return Some(true);
                }
            }
            return Some(false);
        }
        FAT_CIGAM_64 => {
            let nfat_arch = read_u32(bytes, 4, true)? as usize;
            for index in 0..nfat_arch {
                let arch_offset = 8 + index * 32;
                let hi = read_u32(bytes, arch_offset + 8, true)? as u64;
                let lo = read_u32(bytes, arch_offset + 12, true)? as u64;
                let size_hi = read_u32(bytes, arch_offset + 16, true)? as u64;
                let size_lo = read_u32(bytes, arch_offset + 20, true)? as u64;
                let slice_offset = ((hi << 32) | lo) as usize;
                let slice_size = ((size_hi << 32) | size_lo) as usize;
                let slice = bytes.get(slice_offset..slice_offset.checked_add(slice_size)?)?;
                if macho_cryptid_one(slice)? {
                    return Some(true);
                }
            }
            return Some(false);
        }
        _ => {}
    }

    match magic_be {
        MH_CIGAM => parse_macho_load_commands(bytes, false, false),
        MH_CIGAM_64 => parse_macho_load_commands(bytes, false, true),
        FAT_MAGIC => {
            let nfat_arch = read_u32(bytes, 4, false)? as usize;
            for index in 0..nfat_arch {
                let arch_offset = 8 + index * 20;
                let slice_offset = read_u32(bytes, arch_offset + 8, false)? as usize;
                let slice_size = read_u32(bytes, arch_offset + 12, false)? as usize;
                let slice = bytes.get(slice_offset..slice_offset.checked_add(slice_size)?)?;
                if macho_cryptid_one(slice)? {
                    return Some(true);
                }
            }
            Some(false)
        }
        FAT_MAGIC_64 => {
            let nfat_arch = read_u32(bytes, 4, false)? as usize;
            for index in 0..nfat_arch {
                let arch_offset = 8 + index * 32;
                let hi = read_u32(bytes, arch_offset + 8, false)? as u64;
                let lo = read_u32(bytes, arch_offset + 12, false)? as u64;
                let size_hi = read_u32(bytes, arch_offset + 16, false)? as u64;
                let size_lo = read_u32(bytes, arch_offset + 20, false)? as u64;
                let slice_offset = ((hi << 32) | lo) as usize;
                let slice_size = ((size_hi << 32) | size_lo) as usize;
                let slice = bytes.get(slice_offset..slice_offset.checked_add(slice_size)?)?;
                if macho_cryptid_one(slice)? {
                    return Some(true);
                }
            }
            Some(false)
        }
        _ => None,
    }
}

pub fn inspect_ipa_path(
    path: &Path,
) -> Result<IpaInspection, Box<dyn std::error::Error + Send + Sync>> {
    let mut zip = read_zip(&path.to_string_lossy())?;
    let app_bundle_name = find_app_bundle_name(&mut zip)?;
    let manifest_targets = read_manifest_targets(&mut zip, &app_bundle_name)?.unwrap_or_default();
    let declared_sinf_paths = ordered_unique_paths(
        manifest_targets
            .sinf_paths
            .iter()
            .cloned()
            .chain(manifest_targets.sinf_replication_paths.iter().cloned()),
    );

    let app_prefix = format!("Payload/{}/", app_bundle_name);
    let mut present_sinf_paths = Vec::new();
    let mut plugin_dirs = HashSet::new();
    for i in 0..zip.len() {
        let name = zip.by_index(i)?.name().to_string();
        if name.starts_with(&app_prefix) && name.ends_with(".sinf") {
            present_sinf_paths.push(name.trim_start_matches(&app_prefix).to_string());
        }
        if name.starts_with(&app_prefix) && name.contains("/PlugIns/") && name.ends_with(".appex/")
        {
            plugin_dirs.insert(name.trim_end_matches('/').to_string());
        }
    }
    present_sinf_paths = ordered_unique_paths(present_sinf_paths);

    let present_sinf_set = present_sinf_paths.iter().cloned().collect::<HashSet<_>>();
    let missing_sinf_paths = declared_sinf_paths
        .iter()
        .filter(|path| !present_sinf_set.contains(*path))
        .cloned()
        .collect::<Vec<_>>();

    let mut encrypted_binaries = Vec::new();
    let app_info_path = format!("Payload/{}/Info.plist", app_bundle_name);
    if let Ok(Value::Dictionary(info)) = read_zip_plist(&mut zip, &app_info_path) {
        if let Some(executable) = info
            .get("CFBundleExecutable")
            .and_then(|value| value.as_string())
        {
            let binary_path = format!("Payload/{}/{}", app_bundle_name, executable);
            if let Ok(binary) = read_zip_entry_bytes(&mut zip, &binary_path) {
                if macho_cryptid_one(&binary).unwrap_or(false) {
                    encrypted_binaries.push(binary_path.trim_start_matches("Payload/").to_string());
                }
            }
        }
    }

    for plugin_dir in ordered_unique_paths(plugin_dirs.into_iter()) {
        let info_path = format!("{}/Info.plist", plugin_dir);
        if let Ok(Value::Dictionary(info)) = read_zip_plist(&mut zip, &info_path) {
            if let Some(executable) = info
                .get("CFBundleExecutable")
                .and_then(|value| value.as_string())
            {
                let binary_path = format!("{}/{}", plugin_dir, executable);
                if let Ok(binary) = read_zip_entry_bytes(&mut zip, &binary_path) {
                    if macho_cryptid_one(&binary).unwrap_or(false) {
                        encrypted_binaries
                            .push(binary_path.trim_start_matches("Payload/").to_string());
                    }
                }
            }
        }
    }

    let has_sc_info_manifest = !declared_sinf_paths.is_empty();
    let has_embedded_mobileprovision = zip
        .by_name(&format!(
            "Payload/{}/embedded.mobileprovision",
            app_bundle_name
        ))
        .is_ok();

    let mut blockers = Vec::new();
    if !missing_sinf_paths.is_empty() {
        blockers.push(format!(
            "包内声明了 {} 个 .sinf 目标，但缺少 {} 个：{}",
            declared_sinf_paths.len(),
            missing_sinf_paths.len(),
            missing_sinf_paths.join(", ")
        ));
    }
    if !encrypted_binaries.is_empty() {
        blockers.push(if has_embedded_mobileprovision {
            format!(
                "检测到 {} 个 FairPlay 加密二进制，这类包通常不是可直接侧载的成品 IPA",
                encrypted_binaries.len()
            )
        } else {
            format!(
                "检测到 {} 个 FairPlay 加密二进制，且未发现 embedded.mobileprovision，这类包不能直接侧载，继续安装大概率黑屏或闪退",
                encrypted_binaries.len()
            )
        });
    } else if !has_embedded_mobileprovision {
        blockers.push(
            "包内未发现 embedded.mobileprovision，当前看起来不像已正确重签的可侧载 IPA".to_string(),
        );
    }

    let direct_install_ok = blockers.is_empty();
    let blocked_reason = (!blockers.is_empty()).then(|| blockers.join("；"));
    let recommended_action = blocked_reason
        .as_ref()
        .map(|_| "请先获取完整解密并正确重签（含全部 .appex）的 IPA，再重新上传或安装".to_string());
    let summary = match (&blocked_reason, &recommended_action) {
        (Some(reason), Some(action)) => format!("{}。{}。", reason, action),
        (Some(reason), None) => reason.clone(),
        _ if has_sc_info_manifest => "未检测到缺失的 .sinf 目标，可继续安装验证".to_string(),
        _ => "未发现明显的 FairPlay / 签名阻塞，可继续安装验证".to_string(),
    };

    Ok(IpaInspection {
        has_sc_info_manifest,
        has_embedded_mobileprovision,
        declared_sinf_paths,
        present_sinf_paths,
        missing_sinf_paths,
        encrypted_binaries,
        direct_install_ok,
        blocked_reason,
        recommended_action,
        summary,
    })
}

impl SignatureClient {
    pub fn new(
        song_list_0: &serde_json::Value,
        email: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = SignatureMetadata {
            bundle_display_name: song_list_0["metadata"]["bundleDisplayName"]
                .as_str()
                .map(|s| s.to_string()),
            bundle_short_version_string: song_list_0["metadata"]["bundleShortVersionString"]
                .as_str()
                .map(|s| s.to_string()),
            bundle_id: song_list_0["metadata"]["bundleId"]
                .as_str()
                .map(|s| s.to_string()),
            artwork_url: {
                let url_60 = song_list_0["metadata"]["artworkUrl60"].as_str();
                let url_512 = song_list_0["metadata"]["artworkUrl512"].as_str();
                let url_100 = song_list_0["metadata"]["artworkUrl100"].as_str();
                url_60.or(url_512).or(url_100).map(|s| s.to_string())
            },
            artist_name: song_list_0["metadata"]["artistName"]
                .as_str()
                .map(|s| s.to_string()),
            apple_id: Some(email.to_string()),
            user_name: Some(email.to_string()),
        };

        let signatures = song_list_0["sinfs"]
            .as_array()
            .map(|sinfs| {
                sinfs
                    .iter()
                    .enumerate()
                    .filter_map(|(index, sinf)| parse_sinf_entry(sinf, index))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if signatures.is_empty() {
            eprintln!(
                "[signature] WARNING: no usable sinf entries found in Apple response; keys={}, sinfs={:?}",
                song_list_0
                    .as_object()
                    .map(|o| o.keys().cloned().collect::<Vec<_>>().join(","))
                    .unwrap_or_default(),
                song_list_0["sinfs"]
            );
        }

        Ok(SignatureClient {
            archive: Vec::new(),
            filename: String::new(),
            metadata,
            signatures,
            email: email.to_string(),
        })
    }

    pub fn load_file(
        &mut self,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        self.archive = buffer;
        self.filename = path.to_string();
        Ok(())
    }

    pub fn append_metadata(&mut self) -> &mut Self {
        let mut dict = plist::Dictionary::new();
        if let Some(name) = &self.metadata.bundle_display_name {
            dict.insert(
                "bundleDisplayName".to_string(),
                plist::Value::String(name.clone()),
            );
        }
        if let Some(version) = &self.metadata.bundle_short_version_string {
            dict.insert(
                "bundleShortVersionString".to_string(),
                plist::Value::String(version.clone()),
            );
        }
        if let Some(bundle_id) = &self.metadata.bundle_id {
            dict.insert(
                "bundleId".to_string(),
                plist::Value::String(bundle_id.clone()),
            );
        }
        if let Some(artwork_url) = &self.metadata.artwork_url {
            dict.insert(
                "artworkUrl".to_string(),
                plist::Value::String(artwork_url.clone()),
            );
        }
        if let Some(artist_name) = &self.metadata.artist_name {
            dict.insert(
                "artistName".to_string(),
                plist::Value::String(artist_name.clone()),
            );
        }
        dict.insert(
            "apple-id".to_string(),
            plist::Value::String(self.email.clone()),
        );
        dict.insert(
            "userName".to_string(),
            plist::Value::String(self.email.clone()),
        );

        let metadata_plist = plist::Value::Dictionary(dict);
        let mut buf = Vec::new();
        let options = plist::XmlWriteOptions::default();
        plist::to_writer_xml_with_options(&mut buf, &metadata_plist, &options)
            .map_err(|e| format!("Failed to serialize plist: {}", e))
            .unwrap();
        let metadata_content = String::from_utf8(buf)
            .map_err(|e| format!("Invalid UTF-8: {}", e))
            .unwrap();

        let reader = Cursor::new(self.archive.clone());
        let mut zip = match ZipArchive::new(reader) {
            Ok(z) => z,
            Err(_) => {
                // Source is not a zip; create a new zip with only iTunesMetadata.
                let mut out = Vec::new();
                let mut archive = zip::ZipWriter::new(Cursor::new(&mut out));
                let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored);
                archive.start_file("iTunesMetadata.plist", options).unwrap();
                archive.write_all(metadata_content.as_bytes()).unwrap();
                let _ = archive.finish();
                self.archive = out;
                return self;
            }
        };

        // IMPORTANT: write to a fresh buffer, then replace `self.archive`.
        let mut out = Vec::with_capacity(self.archive.len() + 4096);
        let mut new_archive = zip::ZipWriter::new(Cursor::new(&mut out));
        let options: zip::write::FileOptions<'_, ()> =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            let name = file.name().to_string();
            if name == "iTunesMetadata.plist" {
                continue;
            }
            new_archive.start_file(&name, options).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            new_archive.write_all(&buffer).unwrap();
        }

        new_archive
            .start_file("iTunesMetadata.plist", options)
            .unwrap();
        new_archive.write_all(metadata_content.as_bytes()).unwrap();
        let _ = new_archive.finish();

        self.archive = out;
        self
    }

    pub fn append_signatures(
        &mut self,
    ) -> Result<SignatureApplyResult, Box<dyn std::error::Error + Send + Sync>> {
        let reader = Cursor::new(self.archive.clone());
        let mut zip = ZipArchive::new(reader)?;
        let app_bundle_name = find_app_bundle_name(&mut zip)?;
        let Some(manifest_targets) = read_manifest_targets(&mut zip, &app_bundle_name)? else {
            return Ok(SignatureApplyResult {
                applied_paths: Vec::new(),
                warning: Some("包内未找到 SC_Info/Manifest.plist，跳过 sinf 注入".to_string()),
            });
        };

        let apply_result = build_injection_plan(&self.signatures, &manifest_targets)?;
        if apply_result.applied_paths.is_empty() {
            return Ok(apply_result);
        }

        let decoded_signatures = decode_signatures(&self.signatures)?;
        let replacements = if decoded_signatures.len() == 1 {
            apply_result
                .applied_paths
                .iter()
                .map(|path| {
                    (
                        format!("Payload/{}/{}", app_bundle_name, path),
                        decoded_signatures[0].clone(),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            apply_result
                .applied_paths
                .iter()
                .enumerate()
                .map(|(index, path)| {
                    (
                        format!("Payload/{}/{}", app_bundle_name, path),
                        decoded_signatures[index].clone(),
                    )
                })
                .collect::<Vec<_>>()
        };

        replace_zip_entries(&mut self.archive, &replacements)?;
        Ok(apply_result)
    }

    pub fn write(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.filename)?;

        file.write_all(&self.archive)?;
        Ok(())
    }
}

pub fn read_zip(
    path: &str,
) -> Result<ZipArchive<std::fs::File>, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(path)?;
    let zip = ZipArchive::new(file)?;
    Ok(zip)
}
