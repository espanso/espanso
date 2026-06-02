# espanso 管理面板 GUI — 设计文档

> **状态:** 待审核  
> **日期:** 2026-06-03  
> **分支:** `gui`  
> **上游仓库:** `https://github.com/espanso/espanso.git`  
> **Fork 仓库:** `https://github.com/10yihang/espanso.git`

---

## 目录

1. [动机与目标](#1-动机与目标)
2. [当前 GUI 能力边界](#2-当前-gui-能力边界)
3. [整体架构](#3-整体架构)
4. [技术选型](#4-技术选型)
5. [新 Crate 结构](#5-新-crate-结构)
6. [模块一：匹配管理器](#6-模块一匹配管理器)
7. [模块二：包管理器](#7-模块二包管理器)
8. [模块三：设置面板](#8-模块三设置面板)
9. [模块四：触发测试器](#9-模块四触发测试器)
10. [模块五：统计仪表盘](#10-模块五统计仪表盘)
11. [IPC 协议扩展](#11-ipc-协议扩展)
12. [国际化 (i18n)](#12-国际化-i18n)
13. [暗色模式](#13-暗色模式)
14. [跨平台策略](#14-跨平台策略)
15. [性能与内存策略](#15-性能与内存策略)
16. [错误边界与撤销](#16-错误边界与撤销)
17. [全局键盘快捷键](#17-全局键盘快捷键)
18. [导入/导出设计](#18-导入导出设计)
19. [开发估时与里程碑](#19-开发估时与里程碑)
20. [风险与缓解](#20-风险与缓解)

---

## 1. 动机与目标

### 问题

espanso 当前缺少可视化管理界面。用户必须手动编辑 YAML 配置文件才能添加、修改或删除匹配。这导致：

- **上手门槛高**：新用户需要学习 YAML 语法和 espanso 的配置模型
- **操作效率低**：每次修改都要打开文本编辑器，找到对应文件，修改后等重载
- **发现性差**：无法浏览自己配置了哪些匹配，只能靠记忆或翻文件
- **包管理困难**：记住 CLI 命令（`espanso package install ...`）不直观
- **调试困难**：改完配置后无法快速验证匹配是否正确生效

### 目标

构建一个**全功能管理面板**（独立桌面窗口），让用户无需触碰任何 YAML 即可完成：

1. 匹配的增删改查（含图片预览）
2. 包的浏览、安装、卸载、更新
3. 可视化设置编辑
4. 触发词测试与调试
5. 使用统计查看

所有功能必须满足：**内存占用极低**（窗口关闭后零增量）、**全平台兼容**、**国际化**、**支持暗色模式**。

---

## 2. 当前 GUI 能力边界

### 已有功能（不替换）

| 功能 | 技术 | 说明 |
|------|------|------|
| 托盘图标 + 右键菜单 | 平台 C FFI / notify-rust | 启用/禁用、搜索、打开配置、退出 |
| 搜索栏 | wxWidgets (C++ FFI) | Alt+Space 弹出，搜索已有匹配并展开 |
| 交互式表单 | wxWidgets (C++ FFI) | 由 `form` 类型匹配触发 |
| 首次运行向导 | wxWidgets (C++ FFI) | 仅安装时出现一次 |
| 配置错误诊断 | wxWidgets (C++ FFI) | YAML 语法错误时弹出 |
| 桌面通知 | 平台原生 API | 启用/禁用/错误通知 |

### 缺失能力（本设计要补充）

| 功能 | 当前状态 |
|------|---------|
| 可视化添加/编辑/删除匹配 | ❌ 必须手写 YAML |
| 匹配列表浏览 | ❌ 只能翻文件 |
| GUI 包管理器 | ❌ 只有 CLI |
| 可视化设置面板 | ❌ 必须查文档，手改 YAML |
| 触发词测试/调试 | ❌ 改完只能实际打字测试 |
| 统计图表界面 | ❌ 有数据但无前端 |
| 图片匹配预览 | ❌ 只知道文件名 |
| 导入/导出 | ❌ 手动复制粘贴文件 |

---

## 3. 整体架构

### 3.1 交互模式：托盘 + 桌面窗口混合

- **托盘右键**：快速操作（启用/禁用、搜索、打开管理面板、退出）
- **双击托盘图标 / 点击「打开管理面板」**：启动完整管理窗口
- **全局快捷键 Alt+Space**：保持现有搜索栏行为不变

### 3.2 进程模型：独立进程 + IPC 通信

```
┌──────────────────────────────────────────────────────┐
│                     CLI 入口                          │
│                                                      │
│  espanso gui          → 打开管理面板                    │
│  espanso gui match    → 直接进入匹配管理                │
│  espanso gui package  → 直接进入包管理                  │
└──────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────┐
│                 espanso-gui 进程                       │
│                                                      │
│  ┌────────────────────────────────────────────────┐  │
│  │              主窗口 (egui + winit)              │  │
│  │  ┌───────┐  ┌──────────────────────────────┐   │  │
│  │  │ 导航  │  │         内容区                 │   │  │
│  │  │ 匹配  │  │                              │   │  │
│  │  │ 包   │  │  根据不同模块渲染不同内容       │   │  │
│  │  │ 设置  │  │                              │   │  │
│  │  │ 测试  │  │                              │   │  │
│  │  │ 统计  │  │                              │   │  │
│  │  └───────┘  └──────────────────────────────┘   │  │
│  └────────────────────────────────────────────────┘  │
│                         │                            │
│                直接读/写 YAML 文件                     │
│                通过 IPC 通知 Worker                    │
└──────────────────────────┼───────────────────────────┘
                           │ IPC (Unix socket / Win named pipe)
                           ▼
┌──────────────────────────────────────────────────────┐
│                 espanso Worker 进程                    │
│                                                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │
│  │ 引擎线程  │  │ UI 线程   │  │ IPC 服务线程      │   │
│  │ (不修改)  │  │ (不修改)  │  │ + 新增事件处理    │   │
│  └──────────┘  └──────────┘  └──────────────────┘   │
└──────────────────────────────────────────────────────┘
```

**关键设计决策：**

- GUI 进程直接读/写 YAML 文件（通过 `espanso-config` crate），不经过 Worker
- 修改配置后，通过 IPC 发送 `ConfigChanged` 通知 Worker 重新加载
- GUI 进程崩溃不影响文本扩展，Worker 崩溃不影响配置编辑
- GUI 独立进程意味着**窗口关闭即释放所有资源**，零内存残留

### 3.3 内存策略

| 状态 | 内存占用 | 说明 |
|------|---------|------|
| 常驻 | ~15-25MB | Tray Icon + Worker 引擎（当前已有，不增加） |
| 面板打开 | +2~5MB | egui 即时模式，无 Widget 树常驻 |
| 打开 + 缩略图 | +5~10MB | 缩略图缓存 LRU，最大 200 张 |
| 面板关闭 | 回到常驻基线 | `std::process::exit(0)`，彻底释放 |

---

## 4. 技术选型

### 4.1 最终选择：egui

| 维度 | egui | wxWidgets (C++ FFI) | WebView (HTML/CSS/JS) |
|------|------|---------------------|----------------------|
| 额外内存 | **+2~5MB** | 共享已有，+0MB | +50~120MB |
| 开发语言 | **纯 Rust** | C++ + 大量 FFI | HTML/CSS/JS |
| 图片预览 | **原生纹理支持** | 需手写图片控件 | 天然支持 |
| 编译时间 | **快** | 极慢（编译 wxWidgets） | 中等 |
| 跨平台一致性 | **完全一致** | 原生风格但不统一 | 完全一致 |
| 长期潜力 | **可逐步替代 wxWidgets** | 维护负担加重 | 内存不可控 |

**选择 egui 的核心逻辑：**

1. 即时模式架构 → 无 Widget 树常驻 → 内存极低
2. 纯 Rust → 零 FFI → 可直接引用 espanso 数据结构
3. 不破坏现有代码 → 新建独立 crate，feature flag 控制

### 4.2 依赖栈

```
espanso-gui
├── egui            # 即时模式 GUI 框架
├── eframe          # egui 框架集成（winit + wgpu/glow）
├── winit           # 窗口创建，跨平台抽象
├── wgpu / glow     # GPU 渲染后端
├── rfd             # 系统原生文件对话框
├── arboard         # 跨平台剪贴板（图片粘贴）
├── image           # 图片解码/缩略图生成
├── resvg           # SVG 缩略图渲染（可选）
├── dark-light      # 系统主题检测
├── rust-i18n       # 国际化框架（编译时嵌入翻译）
├── serde / serde_json / serde_norway  # 配置序列化
├── rusqlite        # 读取 stats 数据库（已有依赖）
└── crossbeam       # 线程通信（已有依赖）
```

---

## 5. 新 Crate 结构

```
espanso-gui/
├── Cargo.toml
├── build.rs               # Windows 资源文件 (manifest)
├── locales/
│   ├── en.yml             # 英语（默认 fallback）
│   ├── zh-CN.yml          # 简体中文
│   ├── ja.yml             # 日本語
│   ├── ko.yml             # 한국어
│   ├── de.yml             # Deutsch
│   └── fr.yml             # Français
└── src/
    ├── main.rs            # 入口：解析 CLI args，创建窗口
    ├── lib.rs             # 库入口
    ├── app.rs             # App 状态机（模块切换、全局状态）
    ├── theme.rs           # 浅色/深色/跟随系统主题管理
    ├── i18n.rs            # 国际化初始化
    ├── ipc.rs             # IPC 客户端（连接 Worker）
    ├── widgets/           # 可复用的 UI 组件
    │   ├── mod.rs
    │   ├── search_bar.rs   # 搜索栏组件
    │   ├── match_list.rs   # 匹配列表组件
    │   ├── match_editor.rs # 匹配编辑器弹窗
    │   ├── confirm_dialog.rs
    │   └── thumbnail.rs   # 缩略图预览组件
    ├── modules/           # 五大功能模块
    │   ├── mod.rs
    │   ├── match_manager.rs   # 模块 1：匹配管理
    │   ├── package_manager.rs # 模块 2：包管理
    │   ├── settings.rs        # 模块 3：设置面板
    │   ├── trigger_tester.rs  # 模块 4：触发测试
    │   └── stats_dashboard.rs # 模块 5：统计仪表盘
    └── backend/           # 数据处理层
        ├── mod.rs
        ├── config_io.rs    # 读/写 YAML 配置（封装 espanso-config）
        ├── package_io.rs   # 包操作（封装 espanso-package）
        ├── match_store.rs  # 匹配数据的内存缓存
        └── stats_io.rs     # 统计数据查询
```

---

## 6. 模块一：匹配管理器

### 6.1 功能清单

- **列表浏览**：以表格/列表形式展示所有匹配
- **搜索筛选**：按触发词模糊搜索 + 按类型筛选（文本/图片/Markdown/脚本/正则）
- **添加匹配**：弹窗表单输入触发词、替换内容、高级选项
- **编辑匹配**：点击匹配行 → 编辑弹窗
- **删除匹配**：单删 / 批量多选删除（移到回收站而非永久删除）
- **图片预览**：图片类型匹配显示缩略图，点击放大查看详情
- **图片添加**：拖拽图片文件 / 点击选择 / 剪贴板粘贴 → 自动复制到 `match/images/`
- **导入/导出**：拖拽 .yml 文件导入 / 选中匹配导出为 .yml 文件

### 6.2 布局

```
┌──────────────────────────────────────────────┐
│ 🔍 搜索匹配...      [全部类型 ▾]              │
├──────────────────────────────────────────────┤
│  [+ 新增] [导入] [导出选中] [删除选中]         │
├──────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────┐ │
│ │ 🖼 :meme   → doge.png         图片  ✏️ 🗑 │ │
│ │ 📝 :addr   → 北京市朝阳区...   文本  ✏️ 🗑 │ │
│ │ ⚡ :date   → {{mydate}}       脚本  ✏️ 🗑 │ │
│ │ 📧 :mail   → ...@example.com  文本  ✏️ 🗑 │ │
│ │ ...                                      │ │
│ └──────────────────────────────────────────┘ │
│ 共 47 个匹配 · 3 种类型                       │
└──────────────────────────────────────────────┘
```

### 6.3 编辑弹窗

```
┌──────────────────────────────────────┐
│ 编辑匹配                     ✕ 关闭  │
├──────────────────────────────────────┤
│ 类型：[文本] [图片] [Markdown] [正则]  │
│                                      │
│ 触发词：  [______________________]    │
│ 替换内容：                            │
│ ┌────────────────────────────────┐   │
│ │ Hi There!                      │   │
│ │                                │   │
│ └────────────────────────────────┘   │
│                                      │
│ ▶ 高级选项                           │
│   ☐ 单词边界   ☐ 大小写传播           │
│   注入模式：[自动 ▾]                   │
│   标签：[__________]                  │
│                                      │
│           [取消]    [保存]            │
└──────────────────────────────────────┘
```

### 6.4 图片预览

- **缩略图**：64×48 缓存到 `.espanso/cache/thumbnails/`，LRU 最大 200 张
- **大图预览弹窗**：点击缩略图 → 弹出大图 + 文件名 + 尺寸 + 格式 + 所在文件位置
- **图片丢失**：占位符 "🖼 图片丢失" + 提示更新路径

### 6.5 保存流程

```
用户保存匹配
│
├─ 构建 Match 结构体 → 序列化为 YAML（保持格式兼容）
├─ 写入对应 .yml 文件
├─ 通知 MatchStore 重新加载
├─ 发送 IPC::ConfigChanged 通知 Worker 重载
└─ 撤销栈记录本次操作
```

### 6.6 数据模型（使用现有 espanso-config 类型）

直接复用 `espanso_config::Match`、`espanso_config::MatchCause`、`espanso_config::MatchEffect` 等类型，确保 GUI 编辑的内容与 YAML 解析保持完全一致。

---

## 7. 模块二：包管理器

### 7.1 功能清单

- **已安装列表**：列出所有已安装包 + 版本 + 描述 + 一键卸载
- **Hub 市场浏览**：搜索/浏览 espanso Hub 上的包 + 一键安装
- **更新提醒**：对比已安装版本 vs Hub 最新版本，显示可更新数量
- **安装详情**：点开包名 → 查看描述、作者、下载量、匹配预览 → 安装

### 7.2 数据流

```
├─ 已安装列表：espanso_package::Archiver::list() → GUI 表格
├─ Hub 市场：HTTP GET hub.espanso.org/package_index.json
│            → 本地缓存 1 小时（复用 espanso-package 已有缓存）
├─ 安装：espanso_package::Provider::download()
│       → Archiver::save() → 刷新 MatchStore → IPC 通知 Worker
├─ 卸载：Archiver::delete() → 刷新 MatchStore → IPC 通知 Worker
├─ 更新检查：对比本地版本 vs Hub 最新版本（natord 排序）
└─ 安装详情：下载并解析 _manifest.yml → 展示匹配预览
```

### 7.3 安装/更新使用独立线程

下载和安装操作在独立线程执行，防止 UI 卡住。显示进度条 + 「取消」按钮。

---

## 8. 模块三：设置面板

### 8.1 功能清单

可视化编辑 `default.yml` 中的所有配置项，按分类组织为 Tabs：

| Tab | 包含配置项 |
|-----|----------|
| **通用** | `enable`, `show_icon`, `show_notifications`, `auto_restart`, `toggle_key`, `stats.enabled` |
| **注入** | `backend`, `clipboard_threshold`, `inject_delay`, `key_delay`, `pre_paste_delay`, `paste_shortcut_event_delay`, `paste_shortcut`, `preserve_clipboard`, `restore_clipboard_delay`, `force_mode`, `disable_x11_fast_inject` |
| **快捷键** | `search_trigger`, `search_shortcut`, `toggle_key` |
| **高级** | `word_separators`, `backspace_limit`, `undo_backspace`, `apply_patch`, `emulate_alt_codes`, `keyboard_layout`, `evdev_modifier_delay`, 以及所有平台特定选项 |

### 8.2 控件类型映射

| YAML 类型 | 控件 |
|-----------|------|
| `bool` | Toggle 开关 |
| `i32` / `u32` | 数字输入框 + 微调控件 |
| `String`（枚举值） | 下拉选择框 |
| `String`（快捷键） | 快捷键录制器（按下按键自动填入） |
| `Vec<String>` | 标签/关键字录入器 |
| `String`（任意文本） | 文本输入框 |

### 8.3 保存流程

```
用户修改设置 → 点击保存
│
├─ 验证所有字段合法性
├─ 读取当前 default.yml → 合并修改 → 写回（保留未修改字段原样）
├─ 发送 IPC::ConfigChanged 通知 Worker
└─ Worker 平滑重载（不中断当前展开）
```

---

## 9. 模块四：触发测试器

### 9.1 功能清单

- **模拟输入**：在输入框中输入触发词，点击测试按钮
- **匹配诊断**：显示匹配了哪个规则（文件名 + 行号）、匹配类型（Trie/Regex）
- **渲染预览**：实时显示渲染后的输出文本
- **变量调试**：展开显示变量解析过程（日期、脚本、表单等）
- **应用上下文模拟**：可手动输入窗口标题/类名，测试应用特定匹配

### 9.2 实现策略

```
用户输入 ":hello" → 点击测试
│
├─ 复用 espanso_match::rolling::RollingMatcher 和 RegexMatcher
├─ 创建 MatchStore → 喂入测试输入 → 跑 Matcher → 得 MatchResult
├─ 复用 espanso_render::Renderer → 渲染模板 → 得输出文本
├─ 脚本类型（shell/script）：通过 IPC 委托 Worker 执行并返回结果
└─ 显示完整链路：触发词 → 匹配规则 → 变量展开 → 最终输出
```

### 9.3 变量调试视图

```
─────────────────────────────────────
触发词：:today
─────────────────────────────────────
匹配规则：:today (match/base.yml#L15)
匹配类型：Rolling (Trie)
─────────────────────────────────────
变量展开：
  {{mydate}}  → date 扩展
    format    = %Y-%m-%d
    tz        = Asia/Shanghai
    → "2026-06-03"
─────────────────────────────────────
渲染输出：
  "2026-06-03"
─────────────────────────────────────
总耗时：< 1ms
```

---

## 10. 模块五：统计仪表盘

### 10.1 功能清单

- **使用热力图**：按天/周/月显示展开频率（柱状图）
- **排行榜**：前 N 个最常用匹配
- **汇总卡片**：总展开次数、活跃匹配数、未使用匹配数
- **时间选择**：过去 7 天 / 本月 / 全部
- **引导开关**：若 stats 未启用，显示引导按钮

### 10.2 数据流

```
Worker 中已有 espanso_stats.db (SQLite via rusqlite)
│
├─ GUI 通过 IPC 请求 stats（指定时间范围、Top N）
├─ Worker 执行 SQL 查询 → 返回聚合数据
├─ GUI 用 egui 自定义绘制柱状图/排行
└─ 若 stats.enabled = false → 显示「⏸ 未启用」引导开关
```

### 10.3 图表实现

egui 没有内置图表组件。使用 egui 的 `Painter` API 手动绘制柱状图：
- 矩形 + 渐变填充
- 顶部数字标注
- 底部标签（最多显示 10 条，超出横向滚动）

---

## 11. IPC 协议扩展

### 11.1 现有事件（不修改）

```rust
pub enum IPCEvent {
    Exit,
    ExitAllProcesses,
    EnableRequest,
    DisableRequest,
    ToggleRequest,
    OpenSearchBar,
    OpenConfigFolder,
    RequestMatchExpansion(RequestMatchExpansionPayload),
}
```

### 11.2 新增事件类型

```rust
pub enum IPCEvent {
    // ... 现有事件 ...

    // === 新增：GUI 管理面板相关 ===

    /// 通知 Worker 重新加载配置（匹配变更后）
    ConfigChanged,

    /// 请求 Worker 执行一次匹配 + 渲染（触发测试器）
    /// Worker 返回 RenderedResult
    RequestTestExpansion(TestExpansionPayload),

    /// 请求 stats 数据
    RequestStats(StatsQueryPayload),

    /// Worker 返回 stats 数据
    StatsResponse(StatsResponsePayload),

    /// 请求 Worker 状态（运行中/已禁用/安全输入等）
    RequestWorkerStatus,

    /// Worker 返回状态
    WorkerStatusResponse(WorkerStatusPayload),
}

pub struct TestExpansionPayload {
    pub trigger: String,
    /// 模拟的应用上下文（可选）
    pub app_title: Option<String>,
    pub app_class: Option<String>,
    pub app_exec: Option<String>,
}

pub struct RenderedResult {
    pub matched: bool,
    pub match_id: Option<i32>,
    pub match_trigger: Option<String>,
    pub match_file: Option<String>,
    pub match_line: Option<u32>,
    pub match_type: String,       // "trie" | "regex"
    pub rendered_body: String,    // 渲染后的文本
    pub variable_trace: Vec<VariableTraceEntry>,
    pub elapsed_us: u64,
}

pub struct VariableTraceEntry {
    pub var_name: String,
    pub var_type: String,
    pub input_params: HashMap<String, String>,
    pub output_value: String,
    pub error: Option<String>,
}

pub struct StatsQueryPayload {
    pub period: String,    // "7d" | "30d" | "all"
    pub top_n: usize,      // 默认 10
}

pub struct StatsResponsePayload {
    pub total_expansions: u64,
    pub active_matches: u64,
    pub unused_matches: u64,
    pub top_triggers: Vec<TriggerStat>,
}

pub struct TriggerStat {
    pub trigger: String,
    pub count: u64,
    pub last_used: Option<String>, // ISO 8601
}

pub struct WorkerStatusPayload {
    pub running: bool,
    pub enabled: bool,
    pub secure_input_active: bool,
    pub pid: u32,
}
```

---

## 12. 国际化 (i18n)

### 12.1 方案

使用 `rust-i18n` crate：编译时将 YAML 翻译文件嵌入二进制，运行时按系统 locale 自动选择。零额外内存开销。

### 12.2 支持语言（MVP）

| 语言 | 文件 | 优先级 |
|------|------|--------|
| English | `en.yml` | 默认 fallback |
| 简体中文 | `zh-CN.yml` | P0 |
| 日本語 | `ja.yml` | P1 |
| 한국어 | `ko.yml` | P1 |
| Deutsch | `de.yml` | P2 |
| Français | `fr.yml` | P2 |

### 12.3 翻译条数估算

| 类别 | 条数 |
|------|------|
| 菜单/导航 | ~15 |
| 匹配管理 | ~50 |
| 包管理 | ~20 |
| 设置面板 | ~60 |
| 触发测试器 | ~20 |
| 统计仪表盘 | ~15 |
| 通用错误/提示 | ~40 |
| **总计** | **~220 条** |

### 12.4 使用方式

```yaml
# locales/zh-CN.yml
_match_manager:
  title: "匹配管理"
  search_placeholder: "搜索匹配..."
  add_button: "+ 新增"
  delete_button: "删除选中"
  type_filter_all: "全部类型"
  # ...
```

```rust
// Rust 代码中
use rust_i18n::t;
label = t!("match_manager.title");
```

翻译文件缺失时自动 fallback 到 `en.yml`。

---

## 13. 暗色模式

### 13.1 方案

egui 原生支持 `Visuals::dark()` / `Visuals::light()`，无需额外实现。

使用 `dark-light` crate 检测系统主题偏好。

### 13.2 三种模式

| 模式 | 行为 |
|------|------|
| **跟随系统**（默认） | 自动检测 OS 主题设置，实时切换 |
| **浅色** | 始终浅色 |
| **深色** | 始终深色 |

### 13.3 实现

```rust
// theme.rs
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

pub fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
    let visuals = match mode {
        ThemeMode::System => {
            if dark_light::detect() == dark_light::Mode::Dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            }
        }
        ThemeMode::Light => egui::Visuals::light(),
        ThemeMode::Dark => egui::Visuals::dark(),
    };
    ctx.set_visuals(visuals);
}
```

### 13.4 持久化

主题偏好存储到 `~/.config/espanso/config/gui_preferences.json`。

---

## 14. 跨平台策略

### 14.1 兼容性矩阵

| 技术层 | macOS | Windows | Linux X11 | Linux Wayland |
|--------|-------|---------|-----------|---------------|
| **窗口创建** | winit ✅ | winit ✅ | winit ✅ | winit ⚠️ 部分合成器有已知问题 |
| **GPU 渲染** | Metal ✅ | DX11/DX12 ✅ | Vulkan/GL ✅ | Vulkan/GL ✅ |
| **文件对话框** | rfd (原生) ✅ | rfd (原生) ✅ | rfd (GTK/Zenity) ✅ | rfd (GTK/Zenity) ✅ |
| **剪贴板图片** | arboard ✅ | arboard ✅ | arboard ✅ | arboard ✅ |
| **系统托盘** | 不动现有 C FFI | 不动现有 C FFI | 不动现有 notify-rust | 不动现有 notify-rust |
| **IPC** | Unix socket ✅ | 命名管道 ✅ | Unix socket ✅ | Unix socket ✅ |
| **缩略图** | image crate ✅ | image crate ✅ | image crate ✅ | image crate ✅ |
| **IME（中文输入）** | winit Ime ⚠️ | winit Ime ⚠️ | winit Ime ⚠️ | winit Ime ⚠️ |

### 14.2 Wayland 特殊处理

- GPU 后端优先使用 OpenGL fallback，避免 Vulkan 合成器（如 Sway/Hyprland）兼容问题
- 窗口装饰使用 winit 的 CSD (Client-Side Decoration)，因为不是所有 Wayland 合成器都支持 SSD
- 测试目标：GNOME (mutter)、KDE (kwin)、Sway、Hyprland

### 14.3 IME 处理

winit 提供 Ime 事件，egui 需要手动处理：

```
winit Ime::Preedit → 显示组合中文本
winit Ime::Commit → 提交确认文本到输入框
```

在中文/日文/韩文环境中，搜索框和文本编辑区域必须正确处理 IME 输入。

---

## 15. 性能与内存策略

| 机制 | 说明 |
|------|------|
| **懒加载窗口** | 只在用户点击「管理面板」时创建 egui context 和窗口 |
| **缩略图缓存** | 按需生成 64×48 缩略图，LRU 淘汰，最大 200 张（~5MB） |
| **Hub 索引缓存** | 复用 espanso-package 的 1 小时缓存，不在 GUI 中重复缓存 |
| **Match 数据** | 窗口打开时一次性加载 MatchStore 到内存，关闭即释放 |
| **关闭 = 彻底释放** | `std::process::exit(0)`，不保留后台线程 |
| **大包安装不阻塞 UI** | 下载/安装用独立线程 + 进度显示 + 可取消 |

---

## 16. 错误边界与撤销

| 场景 | 处理策略 |
|------|---------|
| YAML 语法错误（保存时） | ❌ 阻止保存 + 高亮错误行 + 显示具体错误 |
| 触发词冲突 | ⚠️ 警告弹窗「:hello 已存在，是否覆盖？」 |
| 保存后 Worker 重载失败 | 显示诊断窗口 + 保留修改（不丢失用户编辑） |
| 误删除匹配 | 移到 `.espanso/trash/` 目录，支持恢复 |
| 批量导入格式错误 | 逐行展示结果：✅ 23 成功 ⚠️ 2 跳过（含原因） |
| 图片文件被移动/删除 | 缩略图显示「图片丢失」占位符 + 提示更新路径 |
| IPC 连接断开 | 显示「Worker 未连接」横幅，允许离线编辑配置 |

---

## 17. 全局键盘快捷键

| 快捷键 | 功能 | 上下文 |
|--------|------|--------|
| `Ctrl+N` | 新建匹配 | 全局（任意模块） |
| `Ctrl+S` | 保存当前编辑 | 编辑模式下 |
| `Ctrl+F` | 聚焦搜索框 | 列表视图中 |
| `Ctrl+Z` | 撤销上一步操作 | 编辑/删除操作后 |
| `Delete` | 删除选中匹配 | 匹配列表中有选中项 |
| `Escape` | 关闭弹窗 / 取消编辑 | 弹窗/编辑模式 |
| `Ctrl+1` ~ `Ctrl+5` | 切换到模块 1~5 | 全局 |
| `Ctrl+Shift+T` | 切换主题（浅色/深色/系统） | 全局 |

---

## 18. 导入/导出设计

### 18.1 导入

| 方式 | 描述 |
|------|------|
| 拖拽文件 | 拖 .yml/.yaml 文件到匹配列表区域 |
| 文件对话框 | 点击「导入」按钮 → 选择文件 |
| 文本粘贴 | 在编辑器中粘贴 YAML 片段，自动解析 |
| 剪贴板图片 | Ctrl+V 粘贴图片到匹配编辑区 |

### 18.2 导出

| 方式 | 描述 |
|------|------|
| 选中导出 | 多选匹配 → 「导出选中」→ 保存为 .yml 文件 |
| 全量备份 | 打包所有 config/ + match/ 为 .zip 文件 |

---

## 19. 开发估时与里程碑

### 19.1 里程碑划分

#### Milestone 1：骨架 + 匹配管理器（P0，最核心）

| 任务 | 估时 |
|------|------|
| `espanso-gui` crate 骨架（Cargo.toml、build.rs、main.rs） | 0.5 天 |
| egui + winit 窗口创建 + 事件循环 | 1 天 |
| 左侧导航 + 模块切换框架 | 0.5 天 |
| 暗色模式系统 | 0.5 天 |
| 国际化框架搭建 + 英语/中文翻译 | 1 天 |
| 匹配列表视图 | 2 天 |
| 匹配编辑弹窗（添加/编辑） | 2 天 |
| 匹配删除 + 回收站 | 1 天 |
| 图片缩略图预览 | 1.5 天 |
| YAML 读写集成（config_io.rs） | 1 天 |
| IPC 通信集成（通知 Worker 重载） | 0.5 天 |
| 导入/导出 | 1 天 |
| **里程碑小计** | **~12.5 天** |

#### Milestone 2：包管理器 + 设置面板（P1）

| 任务 | 估时 |
|------|------|
| 已安装包列表 | 1 天 |
| Hub 市场浏览 | 1.5 天 |
| 安装详情弹窗 + 安装/卸载 | 1 天 |
| 更新检查 + 提醒 | 0.5 天 |
| 设置面板：通用 Tab | 1.5 天 |
| 设置面板：注入 Tab | 1.5 天 |
| 设置面板：快捷键 + 高级 Tab | 1 天 |
| YAML 设置读写 | 1 天 |
| **里程碑小计** | **~9 天** |

#### Milestone 3：触发测试器 + 统计仪表盘（P2）

| 任务 | 估时 |
|------|------|
| 触发测试器：匹配引擎集成 | 2 天 |
| 触发测试器：变量调试视图 | 1 天 |
| 触发测试器：脚本 IPC 委托 | 1 天 |
| 统计仪表盘：IPC stats 协议 | 0.5 天 |
| 统计仪表盘：柱状图 + 排行榜 | 1 天 |
| 统计仪表盘：汇总卡片 + 开关引导 | 0.5 天 |
| **里程碑小计** | **~6 天** |

#### Milestone 4：全平台测试 + 修边（P3）

| 任务 | 估时 |
|------|------|
| macOS 全功能测试 | 1 天 |
| Windows 全功能测试 | 1 天 |
| Linux X11 全功能测试 | 1 天 |
| Linux Wayland 兼容测试 | 1 天 |
| IME 输入法测试 | 0.5 天 |
| 性能/内存基准测试 | 0.5 天 |
| 剩余翻译补全 | 1 天 |
| Bug 修复缓冲 | 2 天 |
| **里程碑小计** | **~8 天** |

### 19.2 总计

| 阶段 | 估时 |
|------|------|
| M1：骨架 + 匹配管理器 | 12.5 天 |
| M2：包管理器 + 设置面板 | 9 天 |
| M3：触发测试器 + 统计 | 6 天 |
| M4：测试 + 修边 | 8 天 |
| **总计** | **~35.5 天** |

---

## 20. 风险与缓解

| 风险 | 严重度 | 缓解措施 |
|------|--------|---------|
| YAML 序列化回写丢失注释/格式 | 🟡 中 | 使用 `serde_norway` 保留格式；保存前 diff 对比，只写变化的文件 |
| Wayland + egui 窗口稳定性 | 🟡 中低 | GPU 后端默认 OpenGL fallback，避免 Vulkan 合成器问题 |
| 中文/日文 IME 在 egui 输入 | 🟡 中 | winit Ime 事件手动处理 preedit/commit，在四平台上分别测试 |
| SVG 缩略图生成 | 🟢 低 | `image` crate 不支持 SVG → 引入 `resvg` 轻量渲染 |
| 大包下载阻塞 UI | 🟢 低 | 下载/安装用独立线程 + 进度条 + 取消按钮 |
| egui 在低配机器上的 GPU 兼容性 | 🟡 中低 | 提供 glow (OpenGL ES 3.0) 作为 wgpu 的 fallback |
| Worker 进程 IPC 连接不稳定 | 🟢 低 | 离线编辑模式：即使 Worker 没运行也能修改配置，启动后重载 |

---

## 附录 A：不修改的现有代码

为确保变更范围可控，以下已有代码**在本设计中不做任何修改**：

- `espanso-modulo`（wxWidgets C++ FFI）
- `espanso-engine`（引擎核心管线）
- `espanso-detect`（按键检测）
- `espanso-inject`（文本注入）
- `espanso-clipboard`（剪贴板操作）
- `espanso-config`（配置解析 —— 仅作为依赖调用）
- `espanso-match`（匹配引擎 —— 仅作为依赖调用）
- `espanso-render`（渲染引擎 —— 仅作为依赖调用）
- `espanso-info`（系统信息）
- `espanso-package`（包管理 —— 仅作为依赖调用）
- `espanso-kvs`（键值存储）
- `espanso-mac-utils`（macOS 工具）

**需要修改的已有代码：**

- `espanso-ipc`：新增 6 个 IPC 事件类型
- `espanso/src/main.rs`：注册 `gui` 子命令
- `espanso/Cargo.toml`：新增对 `espanso-gui` 的可选依赖（feature flag）
- Worker IPC 服务：处理新增的 IPC 事件类型

---

## 附录 B：待定事项

以下内容在本次设计中不做决定，留到实现阶段根据实际情况调整：

1. **具体颜色方案**：深色/浅色模式的具体色板
2. **字体选择**：是否捆绑等宽字体用于代码/配置预览
3. **窗口默认大小和最小尺寸**
4. **自动更新检查**：是否在 GUI 中集成版本更新提醒
5. **Snippet 分享功能**：是否添加在线分享匹配片段的能力
