# Firefile Launcher

跨平台 Minecraft 启动器，使用 Tauri（Rust + Vue 3）构建。

## 技术栈

- **前端**: Vue 3 + TypeScript、Vite、vue-i18n、Lucide 图标
- **后端**: Tauri v2、Rust（tokio 异步运行时）
- **Minecraft 核心**: minecraft-java-rs-core
- **数据源**: BMCLAPI（版本清单/加载器）、Modrinth API（资源）、Mojang（新闻）

## 开发

```sh
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev

# 重置开发环境（删除 ~/.minecraft 后启动）
npm run fil-reset

# 构建
npm run tauri build
```

## 项目结构

```
src/
├── components/
│   ├── view/
│   │   ├── HomePage.vue          # 首页（Mojang 新闻）
│   │   ├── ResourcesCenter.vue   # 资源中心（Modrinth 浏览）
│   │   ├── new_mci/              # 新建实例
│   │   └── onboarding/           # 首次设置向导（OOBE）
│   ├── settings_interface.vue    # 设置界面
│   └── Sidebar.vue              # 侧边导航栏
├── lang/                         # i18n 语言文件（6 种语言）
├── utils/
│   └── cache.ts                  # localStorage 缓存（24h 过期）
└── assets/                       # 静态资源

src-tauri/
├── src/
│   ├── lib.rs                    # Tauri 应用入口、命令注册
│   └── launcher.rs               # Minecraft 启动、版本查询、Java 检测等命令
└── Cargo.toml
```

## 功能

- **游戏启动**: 支持 Fabric/Forge 模组加载器
- **版本管理**: 从 BMCLAPI 获取 Minecraft 版本、Fabric、Forge 清单
- **资源浏览**: 通过 Modrinth API 搜索模组/整合包/光影包/资源包
- **Java 管理**: 自动检测系统 Java，内存配置
- **首次设置向导**: 语言、主题、Java、账户设置
- **国际化**: 简体中文、英语、韩语、法语、俄语、越南语
