# OTA 在线安装功能实现文档

## 概述

本文档描述了 ipatool 项目中 OTA 在线安装功能的实现细节。

## 功能描述

OTA (Over-The-Air) 安装允许用户通过 iOS Safari 浏览器直接安装应用，无需使用 iTunes 或 Xcode。

## 实现的 API 端点

### 1. `/manifest` - 生成 plist 清单文件

**请求格式：**
```
GET /manifest?url={ipa_url}&bundle_id={bundle_id}&bundle_version={version}&title={app_name}
```

**参数说明：**
- `url` - IPA 文件的下载 URL（需 HTTPS）
- `bundle_id` - 应用的 Bundle ID（例如：com.example.app）
- `bundle_version` - 应用版本号（例如：1.0.0）
- `title` - 应用显示名称（例如：Test App）

**返回：**
- XML 格式的 plist 文件（Content-Type: application/x-plist）

**示例：**
```bash
curl "http://localhost:8080/manifest?url=https://example.com/app.ipa&bundle_id=com.example.app&bundle_version=1.0.0&title=Test%20App"
```

### 2. `/install` - 生成安装描述文件

**请求格式：**
```
GET /install?manifest={manifest_url}
```

**参数说明：**
- `manifest` - plist 清单文件的 URL（需 HTTPS）

**返回：**
- .mobileconfig 配置文件（Content-Type: application/x-apple-aspen-config）

**示例：**
```bash
curl "http://localhost:8080/install?manifest=https://example.com/manifest.plist"
```

## 安装流程

1. **生成 plist 文件**
   - 后端根据 IPA 文件 URL 和应用元数据生成 plist 清单文件
   - plist 文件包含应用下载地址和元数据

2. **生成 mobileconfig 文件**
   - 后端根据 plist 文件的 URL 生成 .mobileconfig 配置文件
   - .mobileconfig 文件包含 itms-services:// 协议链接
   - 此链接会触发 iOS 系统的 OTA 安装流程

3. **用户安装**
   - 用户在 iOS Safari 中打开 /install 端点返回的 .mobileconfig 文件
   - iOS 系统解析 .mobileconfig 文件
   - 系统自动打开 itms-services:// 链接
   - 系统下载并安装应用

## 文件结构

```
server/src/
├── ota_install.rs          # OTA 安装功能核心实现
│   ├── generate_plist()     # 生成 plist 清单文件
│   ├── generate_mobileconfig()  # 生成 .mobileconfig 文件
│   └── InstallQuery        # 查询参数结构体
├── main.rs                # HTTP 服务器和路由配置
│   ├── get_manifest()      # /manifest 端点处理函数
│   └── install()          # /install 端点处理函数
└── lib.rs                 # 模块声明
    └── pub mod ota_install
```

## 代码实现细节

### generate_plist 函数

```rust
pub fn generate_plist(
    url: String,
    bundle_identifier: String,
    bundle_version: String,
    title: String,
) -> Result<String, Box<dyn std::error::Error>>
```

生成 iOS OTA 安装所需的 plist 清单文件。plist 文件包含以下字段：
- `software-attributes.SoftwarePackageURL` - IPA 文件下载 URL
- `software-attributes.SoftwareVersion` - 应用版本号
- `software-attributes.metadata` - 应用元数据
  - `bundle-identifier` - Bundle ID
  - `bundle-version` - 版本号
  - `title` - 应用名称
  - `kind` - 类型（software）

### generate_mobileconfig 函数

```rust
pub fn generate_mobileconfig(
    manifest_url: String,
    display_name: String,
) -> Result<String, Box<dyn std::error::Error>>
```

生成 iOS 安装描述文件（.mobileconfig）。文件包含：
- 外层 Payload：
  - `PayloadType` - Configuration
  - `PayloadIdentifier` - 配置文件标识符
  - `PayloadUUID` - 唯一标识符
  - `PayloadDescription` - 描述
  - `PayloadDisplayName` - 显示名称
- 内层 Content Payload：
  - `PayloadType` - com.apple.developer.ota-install
  - `URL` - itms-services:// 协议链接
  - `Description` - 描述
  - `DisplayName` - 显示名称

## 使用示例

### JavaScript 示例

```javascript
// 1. 生成 plist 清单文件的 URL
const manifestUrl = `https://your-domain.com/manifest?` +
    `url=${encodeURIComponent(ipaDownloadUrl)}&` +
    `bundle_id=${encodeURIComponent(bundleId)}&` +
    `bundle_version=${encodeURIComponent(version)}&` +
    `title=${encodeURIComponent(appName)}`;

// 2. 生成 mobileconfig 安装链接
const installUrl = `https://your-domain.com/install?manifest=${encodeURIComponent(manifestUrl)}`;

// 3. 在 Safari 中打开安装链接
window.location.href = installUrl;
```

### 前端集成示例

```vue
<template>
  <el-button @click="installApp" type="success">
    点击安装到设备
  </el-button>
</template>

<script setup>
const installApp = () => {
  // 从 API 获取安装 URL
  const installUrl = 'https://your-domain.com/install?manifest=...';

  // 检测环境
  const isHttps = window.location.protocol === 'https:';
  
  if (!isHttps) {
    alert('iOS 安装需要 HTTPS 环境');
    return;
  }

  // 打开安装链接
  window.location.href = installUrl;
};
</script>
```

## 测试

运行测试脚本：
```bash
cd /root/ipatool
chmod +x test_ota.sh
./test_ota.sh http://localhost:8080
```

测试脚本会：
1. 生成 plist 文件并验证其内容
2. 生成 mobileconfig 文件并验证其内容
3. 检查必需的字段是否存在

## 依赖

- `plist` - plist 文件生成和解析（v1.5）
- `urlencoding` - URL 编码（v2.1）
- `uuid` - 生成唯一标识符（v1.6）

## 安全注意事项

1. **HTTPS 要求**
   - iOS 系统要求 OTA 安装必须使用 HTTPS 协议
   - HTTP 环境下无法正常安装

2. **证书验证**
   - HTTPS 证书必须有效
   - 自签名证书在某些 iOS 版本上可能不被信任

3. **URL 验证**
   - 建议在生产环境中验证 manifest URL 的合法性
   - 防止开放重定向攻击

## 故障排查

### 问题：iOS 设备无法安装

**可能原因：**
- 不是 HTTPS 环境
- 证书无效或过期
- iOS 版本限制

**解决方案：**
- 部署到 HTTPS 服务器
- 使用有效的 SSL 证书
- 更新 iOS 系统版本

### 问题：mobileconfig 文件无法打开

**可能原因：**
- plist 格式错误
- Content-Type 不正确

**解决方案：**
- 验证 plist 格式
- 确保 Content-Type 为 `application/x-apple-aspen-config`

## 参考资料

- [iOS OTA Deployment](https://developer.apple.com/library/archive/documentation/NetworkingInternet/Conceptual/iPhoneOTAConfiguration/Introduction/Introduction.html)
- [itms-services Protocol](https://stackoverflow.com/questions/6985524/directly-install-iphone-app-from-url)
- [plist 格式文档](https://developer.apple.com/documentation/bundleresources/property_lists)
