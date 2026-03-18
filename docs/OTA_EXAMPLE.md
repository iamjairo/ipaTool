# OTA 安装使用示例

## 完整安装流程示例

### 步骤 1: 准备 IPA 文件

确保你已经有一个可以公开访问的 IPA 文件下载 URL（必须是 HTTPS）。

例如：`https://your-domain.com/downloads/app.ipa`

### 步骤 2: 生成 plist 清单文件

调用 `/manifest` 端点生成 plist 文件：

```bash
curl "https://your-domain.com/manifest?url=https%3A%2F%2Fyour-domain.com%2Fdownloads%2Fapp.ipa&bundle_id=com.example.app&bundle_version=1.0.0&title=My%20App"
```

返回示例（manifest.plist）：
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>software-attributes</key>
    <dict>
        <key>SoftwarePackageURL</key>
        <string>https://your-domain.com/downloads/app.ipa</string>
        <key>SoftwareVersion</key>
        <string>1.0.0</string>
        <key>URL</key>
        <string>https://your-domain.com/downloads/app.ipa</string>
        <key>metadata</key>
        <dict>
            <key>bundle-identifier</key>
            <string>com.example.app</string>
            <key>bundle-version</key>
            <string>1.0.0</string>
            <key>title</key>
            <string>My App</string>
            <key>kind</key>
            <string>software</string>
        </dict>
    </dict>
</dict>
</plist>
```

### 步骤 3: 生成 mobileconfig 安装文件

调用 `/install` 端点生成 .mobileconfig 文件：

```bash
curl "https://your-domain.com/install?manifest=https%3A%2F%2Fyour-domain.com%2Fmanifest%3Furl%3Dhttps%253A%252F%252Fyour-domain.com%252Fdownloads%252Fapp.ipa%26bundle_id%3Dcom.example.app%26bundle_version%3D1.0.0%26title%3DMy%2520App"
```

返回示例（install.mobileconfig）：
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>PayloadContent</key>
    <dict>
        <key>Content</key>
        <dict>
            <key>URL</key>
            <string>itms-services://?action=download-manifest&amp;url=https%3A%2F%2Fyour-domain.com%2Fmanifest%3Furl%3Dhttps%253A%252F%252Fyour-domain.com%252Fdownloads%252Fapp.ipa%26bundle_id%3Dcom.example.app%26bundle_version%3D1.0.0%26title%3DMy%2520App</string>
        </dict>
        <key>Description</key>
        <string>Install app</string>
        <key>DisplayName</key>
        <string>manifest</string>
        <key>Identifier</key>
        <string>com.ipatool.install.12345678-1234-1234-1234-123456789012</string>
        <key>PayloadType</key>
        <string>com.apple.developer.ota-install</string>
        <key>PayloadUUID</key>
        <string>12345678-1234-1234-1234-123456789012</string>
        <key>PayloadVersion</key>
        <integer>1</integer>
    </dict>
    <key>PayloadDescription</key>
    <string>Install app via OTA</string>
    <key>PayloadDisplayName</key>
    <string>manifest</string>
    <key>PayloadIdentifier</key>
    <string>com.ipatool.config.12345678-1234-1234-1234-123456789012</string>
    <key>PayloadOrganization</key>
    <string>ipaTool</string>
    <key>PayloadRemovalDisallowed</key>
    <false/>
    <key>PayloadType</key>
    <string>Configuration</string>
    <key>PayloadUUID</key>
    <string>12345678-1234-1234-1234-123456789012</string>
    <key>PayloadVersion</key>
    <integer>1</integer>
</dict>
</plist>
```

### 步骤 4: 在 iOS Safari 中安装

1. 在 iOS 设备上打开 Safari 浏览器
2. 访问 `/install` 端点返回的 .mobileconfig 文件 URL
3. 点击"安装"按钮
4. 前往"设置" → "通用" → "VPN 与设备管理"
5. 找到对应的应用配置文件并安装

## JavaScript 集成示例

### Vue 3 示例

```vue
<script setup>
import { ref } from 'vue'

const appData = ref({
  name: 'My App',
  bundleId: 'com.example.app',
  version: '1.0.0',
  downloadUrl: 'https://your-domain.com/downloads/app.ipa'
})

const installApp = async () => {
  // 检查是否为 HTTPS 环境
  if (window.location.protocol !== 'https:') {
    alert('OTA 安装需要 HTTPS 环境')
    return
  }

  try {
    // 生成 manifest URL
    const manifestParams = new URLSearchParams({
      url: appData.value.downloadUrl,
      bundle_id: appData.value.bundleId,
      bundle_version: appData.value.version,
      title: appData.value.name
    })
    const manifestUrl = `/manifest?${manifestParams}`

    // 生成 install URL
    const installParams = new URLSearchParams({
      manifest: `${window.location.origin}${manifestUrl}`
    })
    const installUrl = `/install?${installParams}`

    // 在 Safari 中打开安装链接
    window.location.href = installUrl

  } catch (error) {
    console.error('安装失败:', error)
    alert('生成安装链接失败')
  }
}
</script>

<template>
  <div>
    <h2>{{ appData.name }}</h2>
    <p>Bundle ID: {{ appData.bundleId }}</p>
    <p>版本: {{ appData.version }}</p>

    <el-button @click="installApp" type="success" size="large">
      点击安装到设备
    </el-button>

    <p class="hint">
      请在 iOS 设备的 Safari 中打开此页面并点击安装
    </p>
  </div>
</template>

<style scoped>
.hint {
  font-size: 14px;
  color: #999;
  margin-top: 16px;
  text-align: center;
}
</style>
```

### 原生 JavaScript 示例

```javascript
function installApp(appName, bundleId, version, downloadUrl) {
  // 检查 HTTPS
  if (window.location.protocol !== 'https:') {
    alert('iOS OTA 安装需要 HTTPS 环境');
    return;
  }

  // 构建参数
  const params = new URLSearchParams({
    url: downloadUrl,
    bundle_id: bundleId,
    bundle_version: version,
    title: appName
  });

  // 生成 manifest URL
  const manifestUrl = `${window.location.origin}/manifest?${params}`;

  // 生成 install URL
  const installUrl = `${window.location.origin}/install?manifest=${encodeURIComponent(manifestUrl)}`;

  // 打开安装链接
  window.location.href = installUrl;
}

// 使用示例
installApp(
  'My App',
  'com.example.app',
  '1.0.0',
  'https://your-domain.com/downloads/app.ipa'
);
```

## 后端集成示例（Node.js）

```javascript
const express = require('express');
const axios = require('axios');

const app = express();
const PORT = 3000;

// 处理下载完成后的安装
app.post('/download-complete', async (req, res) => {
  const { fileName, downloadUrl, bundleId, version, appName } = req.body;

  try {
    // 生成 manifest URL
    const manifestParams = {
      url: downloadUrl,
      bundle_id: bundleId,
      bundle_version: version,
      title: appName
    };

    // 生成安装链接
    const installUrl = `https://your-domain.com/install?manifest=${encodeURIComponent(
      `https://your-domain.com/manifest?${new URLSearchParams(manifestParams)}`
    )}`;

    // 返回给前端
    res.json({
      success: true,
      installUrl: installUrl
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
});
```

## 错误处理

### 常见错误及解决方案

#### 1. 非 HTTPS 环境

**错误提示：**
```
iOS OTA 安装需要 HTTPS 环境
```

**解决方案：**
- 部署到 HTTPS 服务器
- 使用 Cloudflare Tunnel 或 Let's Encrypt 等免费 SSL 服务

#### 2. 证书无效

**错误提示：**
```
无法验证服务器身份
```

**解决方案：**
- 使用有效的 SSL 证书
- 避免使用自签名证书

#### 3. 文件下载失败

**错误提示：**
```
无法下载应用
```

**解决方案：**
- 确保 IPA 文件 URL 可公开访问
- 检查文件大小（iOS OTA 安装通常有限制）
- 确认文件格式正确（有效的 IPA 文件）

## 测试清单

- [ ] HTTPS 环境部署完成
- [ ] IPA 文件可公开访问
- [ ] `/manifest` 端点返回正确的 plist 格式
- [ ] `/install` 端点返回正确的 mobileconfig 格式
- [ ] iOS Safari 可打开 .mobileconfig 文件
- [ ] 系统自动触发安装流程
- [ ] 应用成功安装到设备

## 参考链接

- [Apple OTA Deployment Guide](https://developer.apple.com/library/archive/documentation/NetworkingInternet/Conceptual/iPhoneOTAConfiguration/)
- [itms-services 协议说明](https://stackoverflow.com/questions/6985524/directly-install-iphone-app-from-url)
