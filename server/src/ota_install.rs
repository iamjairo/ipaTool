use plist::{Dictionary, Value, XmlWriteOptions};
use serde::Deserialize;

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
    // iOS OTA manifest 需要标准结构：items -> [ { assets, metadata } ]
    let mut root = Dictionary::new();

    let mut asset = Dictionary::new();
    asset.insert("kind".into(), Value::String("software-package".into()));
    asset.insert("url".into(), Value::String(url));

    let mut metadata = Dictionary::new();
    metadata.insert("bundle-identifier".into(), Value::String(bundle_identifier));
    metadata.insert("bundle-version".into(), Value::String(bundle_version));
    metadata.insert("kind".into(), Value::String("software".into()));
    metadata.insert("title".into(), Value::String(title));

    let mut item = Dictionary::new();
    item.insert(
        "assets".into(),
        Value::Array(vec![Value::Dictionary(asset)]),
    );
    item.insert("metadata".into(), Value::Dictionary(metadata));

    root.insert("items".into(), Value::Array(vec![Value::Dictionary(item)]));

    let plist_value = Value::Dictionary(root);
    let mut plist_bytes = Vec::new();
    plist::to_writer_xml_with_options(&mut plist_bytes, &plist_value, &XmlWriteOptions::default())?;
    let plist_string = String::from_utf8(plist_bytes)?;

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
    let itms_url = format!(
        "itms-services://?action=download-manifest&url={}",
        encoded_manifest_url
    );

    // 创建 mobileconfig 的 Payload 数据结构
    let mut content_dict = Dictionary::new();
    content_dict.insert("URL".into(), Value::String(itms_url));

    let mut payload_dict = Dictionary::new();
    payload_dict.insert("Content".into(), Value::Dictionary(content_dict));
    payload_dict.insert("Description".into(), Value::String("Install app".into()));
    payload_dict.insert("DisplayName".into(), Value::String(display_name.clone()));
    payload_dict.insert(
        "Identifier".into(),
        Value::String(format!("com.ipatool.install.{}", uuid::Uuid::new_v4())),
    );
    payload_dict.insert(
        "PayloadType".into(),
        Value::String("com.apple.developer.ota-install".into()),
    );
    payload_dict.insert(
        "PayloadUUID".into(),
        Value::String(uuid::Uuid::new_v4().to_string()),
    );
    payload_dict.insert("PayloadVersion".into(), Value::Integer(1.into()));

    // 外层
    let mut mobileconfig_dict = Dictionary::new();
    mobileconfig_dict.insert("PayloadContent".into(), Value::Dictionary(payload_dict));
    mobileconfig_dict.insert(
        "PayloadDescription".into(),
        Value::String("Install app via OTA".into()),
    );
    mobileconfig_dict.insert("PayloadDisplayName".into(), Value::String(display_name));
    mobileconfig_dict.insert(
        "PayloadIdentifier".into(),
        Value::String(format!("com.ipatool.config.{}", uuid::Uuid::new_v4())),
    );
    mobileconfig_dict.insert(
        "PayloadOrganization".into(),
        Value::String("ipaTool".into()),
    );
    mobileconfig_dict.insert("PayloadRemovalDisallowed".into(), Value::Boolean(false));
    mobileconfig_dict.insert("PayloadType".into(), Value::String("Configuration".into()));
    mobileconfig_dict.insert(
        "PayloadUUID".into(),
        Value::String(uuid::Uuid::new_v4().to_string()),
    );
    mobileconfig_dict.insert("PayloadVersion".into(), Value::Integer(1.into()));

    // 生成 mobileconfig XML 字符串
    let mobileconfig_value = Value::Dictionary(mobileconfig_dict);
    let mut mobileconfig_bytes = Vec::new();
    plist::to_writer_xml_with_options(
        &mut mobileconfig_bytes,
        &mobileconfig_value,
        &XmlWriteOptions::default(),
    )?;
    let mobileconfig_string = String::from_utf8(mobileconfig_bytes)?;

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
        assert!(plist.contains("<key>items</key>"));
        assert!(plist.contains("software-package"));
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
