# IDEA Mark - IntelliJ IDEA 书签管理工具

<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="IDEA Mark Logo" width="128" height="128">
</p>

<p align="center">
  一款轻量级桌面应用，帮助你自动读取并管理 IntelliJ IDEA 的书签数据
</p>

---

## ✨ 功能特性

- 📖 **书签读取** - 自动解析 IDEA 工作空间中的书签数据
- 📊 **书签统计** - 可视化展示书签分布和使用情况
- 💾 **数据备份** - 支持书签数据的导入导出
- 🔍 **快速搜索** - 快速定位和筛选书签内容
- 🖥️ **跨平台支持** - 支持 Windows、macOS、Linux

---

## 🛠️ 技术栈

| 类型 | 技术 |
|------|------|
| 前端框架 | Vue 3 |
| UI 组件库 | Element Plus |
| 构建工具 | Vite 5 |
| 桌面框架 | Tauri 2 |
| 后端语言 | Rust |
| 本地数据库 | SQLite |

---

## 📦 环境准备

在开始之前，请确保你的电脑已安装以下软件：

### 1. Node.js（必需）

下载地址：https://nodejs.org/

推荐安装 **LTS 版本**（长期支持版），安装完成后打开终端验证：

```bash
node -v    # 应显示版本号，如 v18.x.x
npm -v     # 应显示版本号，如 9.x.x
```

### 2. Rust（必需）

下载地址：https://www.rust-lang.org/tools/install

**Windows 用户：**
1. 下载并运行 `rustup-init.exe`
2. 按提示完成安装（选择默认选项即可）
3. 安装完成后重启终端

**验证安装：**
```bash
rustc --version    # 应显示版本号，如 rustc 1.75.0
cargo --version    # 应显示版本号，如 cargo 1.75.0
```

### 3. Visual Studio C++ 构建工具（仅 Windows）

Tauri 在 Windows 上需要 C++ 构建工具：

1. 下载 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. 运行安装程序，勾选 **"使用 C++ 的桌面开发"**
3. 完成安装

---

## 🚀 快速开始

### 第一步：克隆项目

```bash
git clone <你的仓库地址>
cd ideaMark
```

### 第二步：安装依赖

```bash
npm install
```

### 第三步：启动开发模式

```bash
npm run tauri dev
```

首次启动会编译 Rust 代码，可能需要 **3-5 分钟**，请耐心等待。

启动成功后会自动打开应用窗口。

---

## 📦 打包发布

生成可分发的安装包：

```bash
npm run tauri build
```

打包完成后，安装包位于 `src-tauri/target/release/bundle/` 目录下：

- **Windows**: `.msi` 或 `.exe` 安装包
- **macOS**: `.dmg` 安装包
- **Linux**: `.deb` 或 `.AppImage`

---

## 📁 项目结构

```
ideaMark/
├── src/                    # 前端源码（Vue）
│   ├── components/         # Vue 组件
│   │   ├── BookmarkDashboard.vue   # 主面板
│   │   ├── BookmarkList.vue        # 书签列表
│   │   ├── BookmarkStats.vue       # 统计图表
│   │   └── ...
│   ├── App.vue             # 根组件
│   └── main.js             # 入口文件
├── src-tauri/              # 后端源码（Rust）
│   ├── src/                # Rust 源码
│   ├── Cargo.toml          # Rust 依赖配置
│   └── tauri.conf.json     # Tauri 配置
├── package.json            # 前端依赖配置
└── vite.config.js          # Vite 配置
```

---

## ❓ 常见问题

### Q: 首次运行 `npm run tauri dev` 很慢？

A: 正常现象。首次需要编译 Rust 依赖，后续启动会快很多。

### Q: 提示找不到 Rust 或 Cargo？

A: 请确保 Rust 安装正确，并重启终端让环境变量生效。

### Q: Windows 上编译报错？

A: 请确保已安装 Visual Studio C++ 构建工具。

---

## 📄 开源协议

MIT License

---

## 👨‍💻 作者

小爨
