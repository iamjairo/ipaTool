use plist::Value;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// OTA 安装参数
#[derive(Debug, Deserialize)]
pub struct InstallQuery {
    /// manifest URL - plist 文件的 URL
    pub manifest: String,
}

/// 生成 plist 清单文件
/// 
/// 用于 iOS OTA 安装，包含应用下载链接和元数据
pub fn generate_plist(
    url: String,
    bundle_identifier: String,
    bundle_version: String,
    title: String,
) -> Result<String, Box<dyn std::error::Error>> {
    // 创建 plist 数据结构
    let mut dict = BTreeMap::new();

    // 软件属性
    let mut software_props = BTreeMap::new();
    software_props.insert("SoftwarePackageURL".to_string(), Value::String(url));
    software_props.insert("SoftwareVersion".to_string(), Value::String(bundle_version));
    software_props.insert("URL".to_string(), Value::String(url));

    // 元数据
    let mut metadata = BTreeMap::new();
    metadata.insert("bundle-identifier".to_string(), Value::String(bundle_identifier));
    metadata.insert("bundle-version".to_string(), Value::String(bundle_version));
    metadata.insert("title".to_string(), Value::String(title));
    metadata.insert("kind".to_string(), Value::String("software".to_string()));

    software_props.insert("metadata".to_string(), Value::Dictionary(metadata));

    dict.insert("software-attributes".to_string(), Value::Dictionary(software_props));

    // 生成 plist 字符串
    let plist_value = Value::Dictionary(dict);
    let plist_string = plist::to_string_xml(&plist_value)?;

    Ok(plist_string)
}

/// 生成 iOS 安装描述文件（.mobileconfig）
///
/// .mobileconfig 文件包含一个 URL，指向 itms-services:// 协议
/// 该协议会打开 plist 文件，触发 OTA 安装
pub fn generate_mobileconfig(
    manifest_url: String,
    display_name: String,
) -> Result<String, Box<dyn std::error::Error>> {
    // 构建 itms-services URL
    let encoded_manifest_url = urlencoding::encode(&manifest_url);
    let itms_url = format!("itms-services://?action=download-manifest&url={}", encoded_manifest_url);

    // 创建 mobileconfig 的 Payload 数据结构
    let mut content_dict = BTreeMap::new();
    content_dict.insert("URL".to_string(), Value::String(itms_url));

    let mut payload_dict = BTreeMap::new();
    payload_dict.insert("Content".to_string(), Value::Dictionary(content_dict));
    payload_dict.insert("Description".to_string(), Value::String("Install app".to_string()));
    payload_dict.insert("DisplayName".to_string(), Value::String(display_name));
    payload_dict.insert("Identifier".to_string(), Value::String(format!("com.ipatool.install.{}", uuid::Uuid::new_v4())));
    payload_dict.insert("PayloadType".to_string(), Value::String("com.apple.developer.ota-install".to_string()));
    payload_dict.insert("PayloadUUID".to_string(), Value::String(uuid::Uuid::new_v4().to_string()));
    payload_dict.insert("PayloadVersion".to_string(), Value::Integer(1));

    // 外层
    let mut mobileconfig_dict = BTreeMap::new();
    mobileconfig_dict.insert("PayloadContent".to_string(), Value::Dictionary(payload_dict));
    mobileconfig_dict.insert("PayloadDescription".to_string(), Value::String("Install app via OTA".to_string()));
    mobileconfig_dict.insert("PayloadDisplayName".to_string(), Value::String(display_name));
    mobileconfig_dict.insert("PayloadIdentifier".to_string(), Value::String(format!("com.ipatool.config.{}", uuid::Uuid::new_v4())));
    mobileconfig_dict.insert("PayloadOrganization".to_string(), Value::String("ipaTool".to_string()));
    mobileconfig_dict.insert("PayloadRemovalDisallowed".to_string(), Value::Boolean(false));
    mobileconfig_dict.insert("PayloadType".to_string(), Value::String("Configuration".to_string()));
    mobileconfig_dict.insert("PayloadUUID".to_string(), Value::String(uuid::Uuid::new_v4().to_string()));
    mobileconfig_dict.insert("PayloadVersion".to_string(), Value::Integer(1));

    // 生成 mobileconfig XML 字符串
    let mobileconfig_value = Value::Dictionary(mobileconfig_dict);
    let mobileconfig_string = plist::to_string_xml(&mobileconfig_value)?;

    Ok(mobileconfig_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_plist() {
        let url = "https://example.com/app.ipa".to_string();
        let bundle_identifier = "com.example.app".to_string();
        let bundle_version = "1.0.0".to_string();
        let title = "Test App".to_string();

        let result = generate_plist(url, bundle_identifier, bundle_version, title);
        assert!(result.is_ok());

        let plist = result.unwrap();
        assert!(plist.contains("SoftwarePackageURL"));
        assert!(plist.contains("bundle-identifier"));
    }

    #[test]
    fn test_generate_mobileconfig() {
        let manifest_url = "https://example.com/manifest.plist".to_string();
        let display_name = "Test App".to_string();

        let result = generate_mobileconfig(manifest_url, display_name);
        assert!(result.is_ok());

        let mobileconfig = result.unwrap();
        assert!(mobileconfig.contains("itms-services://"));
        assert!(mobileconfig.contains("PayloadType"));
    }
}
