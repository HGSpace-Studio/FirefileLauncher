<div align="center">
  <img src="ic_launcher.png" width="120" height="120" alt="Firefly Android Launcher 图标">
  <h1>Firefly Android Launcher</h1>
  <p>一款专为 Android 打造的 Minecraft: Java Edition 启动器</p>
  <p>
    <a href="https://github.com/HGSpace-Studio/Firefly-Launcher">GitHub</a>
  </p>
</div>

---

## 关于本项目

Firefly Android Launcher 基于 [PojavLauncher](https://github.com/PojavLauncherTeam/PojavLauncher) 由 **HGSpace Studio** 维护。

本项目致力于在 Android 设备上提供流畅的 Minecraft: Java Edition 游戏体验，针对移动端进行了深度优化和界面定制。

---

## 版本信息

- **当前版本**: snowdrop
- **包名**: `work.HGSpace.FireflyAndroid`
- **命名空间**: `net.kdt.pojavlaunch.firefly`

---

## 功能特性

| 特性 | 说明 |
|------|------|
| 登录方式 | 支持第三方皮肤站登录（authlib-injector / nide8auth）及离线登录 |
| Apple 玻璃风格 UI | 暗色主题搭配半透明玻璃卡片背景与浅蓝色强调色 |
| 竖屏启动器 | 启动器主界面保持竖屏，进入游戏后自动切换为横屏 |
| 内置 Java 运行环境 | 首次启动自动解压内置 JRE 8、17、21、25 |
| 整合包导入 | 支持本地 ZIP 与 .mrpack 格式整合包导入 |
| 默认键位 | 预装 Bedrock 风格触控键位布局 |
| 内置整合包 | 预置灰白整合优化包，开箱即用 |

---

## 登录支持

| 方式 | 状态 |
|------|------|
| 离线 / 本地 | 支持 |
| 皮肤站（authlib-injector） | 支持 |
| 皮肤站（nide8auth） | 支持 |
| 微软登录 | 支持 |
| Mojang 登录 | **已移除** |

---

## 如何编译

### 环境要求

- Android Studio 或 Gradle 8.13+
- Android SDK API 21+
- NDK 25.0.3
- JDK 17

### 编译步骤

1. 克隆仓库
```bash
git clone https://github.com/HGSpace-Studio/Firefly-Launcher/tree/Firefly-Android
cd Firefly-Launcher
```

2. 编译 Debug 版本
```bash
./gradlew assembleDebug
```

3. 编译 Release 版本
```bash
./gradlew assembleRelease
```

编译完成后，APK 文件位于：
- Debug: `app/build/outputs/apk/debug/`
- Release: `app/build/outputs/apk/release/`

---

## 项目结构

```
app/src/main/
├── assets/           # 内置 JRE、默认键位、整合包
├── java/             # 源代码
│   └── net/kdt/pojavlaunch/firefly/
├── jni/              # 原生代码（C/C++）
└── res/              # Android 资源文件
```

---

## 开源协议

本项目采用 **GNU General Public License v3.0 (GPL-3.0)** 开源协议。

完整协议内容请参阅 [LICENSE](LICENSE)。

---

## 致谢

感谢以下项目和个人对 Firefly Android Launcher 的支持：

- [PojavLauncherTeam](https://github.com/PojavLauncherTeam) - 原版 PojavLauncher 的开发者
- **HGSpace Studio** - Firefly Android Launcher 的开发与维护团队

---

本启动器为社区修改版本，与 Mojang Studios 及 Microsoft 无任何关联。
