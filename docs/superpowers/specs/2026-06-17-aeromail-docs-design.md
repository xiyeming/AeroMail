# AeroMail 文档优化设计 Spec

- **项目**: RutMail / AeroMail
- **日期**: 2026-06-17
- **目标**: 优化 `docs/` 下需求文档，使 UI 设计系统与技术实现方案相互吻合，并补充 Windows / macOS / Linux(Wayland) 跨平台支持。
- **关联输入文档**:
  - `docs/技术需求.md`
  - `docs/UI方案.md`

---

## 1. 设计决策

### 1.1 文档组织方式

采用「产品需求 + UI 设计系统 + 技术实现方案」三份独立文档，职责边界如下：

| 文档 | 职责 | 目标读者 |
|---|---|---|
| `docs/PRD.md` | 产品定位、功能范围、验收标准 | 产品、项目经理、开发 |
| `docs/UI设计系统.md` | 视觉语言、布局、组件、交互、动效 | 设计师、前端开发 |
| `docs/技术实现方案.md` | 架构、模块、数据模型、接口、跨平台实现 | 后端/前端开发、运维 |

### 1.2 技术栈确认

- **后端**: Rust + Tauri v2 + Tokio
- **前端**: Vue 3 + TypeScript + Vite + Tailwind CSS + Shadcn UI
- **数据**: SQLite (rusqlite) + Tantivy
- **协议**: async-imap + lettre + mailparse

### 1.3 核心设计原则

1. **UI 与技术一一映射**: 在 `技术实现方案.md` 中增设「UI 设计系统落地映射」章节，明确每个 UI 决策对应的技术实现。
2. **跨平台完整覆盖**: 不仅保留并细化 Linux/Wayland，还要补充 Windows 与 macOS 的专项实现。
3. **内容去重**: 删除两份旧文档中重复或矛盾的描述，统一术语与数据。
4. **可验证**: PRD 中的验收标准使用 Given-When-Then 或编号清单，避免空话。

---

## 2. 输出文档结构

### 2.1 `docs/PRD.md`

1. 文档信息
2. 产品定位
3. 目标用户与使用场景
4. 核心业务功能
   - 多账户连接（预设国内外主流厂商、OAuth2、高级配置：TLS/端口/Socket）
   - 本地同步引擎
   - HTML 邮件渲染
   - 千万级全文检索
   - 全功能写信编辑器
   - 多窗口独立操作
5. 功能清单与优先级（P0/P1/P2）
6. 非功能性需求
7. 验收标准
8. 术语表

### 2.2 `docs/UI设计系统.md`

1. 设计定位
2. 整体布局体系
   - 标准三栏布局
   - 双 2K / Ultra Wide 优化
3. 视觉层级系统（3 层深度）
4. 颜色系统（Dark 主推 + Light 备选）
5. 字体系统（跨平台字体栈）
6. 组件规范
   - Sidebar
   - 邮件列表 Compact Card
   - 邮件详情页 Header
   - Compose Workspace
   - Command Palette（⌘K / Ctrl+K）
   - Status Bar（状态栏）
   - Toast 提示
7. 交互规范
   - 多窗口模式
   - Reading Mode
   - Hover Actions / 拖拽上传
8. 动效规范
9. 平台适配

### 2.3 `docs/技术实现方案.md`

1. 技术栈选型
2. 系统架构（拓扑图 + 前后端职责 + IPC）
3. 核心模块设计
   - 账户管理（预设厂商、OAuth2、高级配置）
   - 同步引擎
   - 邮件解析与存储
   - 全文检索
   - SMTP 发信与草稿
   - 通知与系统托盘
4. 数据模型
5. 接口设计
6. **UI 设计系统落地映射**
   - 主题切换 → CSS 变量 + Tauri 主题 API
   - 多窗口 → Tauri 多窗口配置
   - 毛玻璃 → Windows Mica / macOS vibrancy / Linux 透明窗口
   - 字体系统 → 跨平台字体栈
   - Command Palette → 前端组件 + 全局快捷键
   - Status Bar / Toast → 全局状态 + Tauri 通知插件
7. 跨平台专项
   - Windows
   - macOS
   - Linux(Wayland)
8. 安全设计
9. 性能与优化
10. 开发与部署

---

## 3. 关键补充点

### 3.1 多账户高级配置

- 支持国内外主流邮件厂商预设（Gmail、Outlook、QQ 邮箱、163 邮箱、企业微信邮箱等）。
- 高级配置项：IMAP/SMTP 服务器地址、端口、加密方式（SSL/TLS/STARTTLS/none）、连接超时、Socket 类型。

### 3.2 跨平台支持

| 平台 | 关键技术 |
|---|---|
| Windows | Mica/Acrylic 材质、原生通知、MSIX/EXE 安装包 |
| macOS | vibrancy、原生菜单栏、通知中心、DMG / 签名 |
| Linux(Wayland) | 原生 Wayland、Fractional Scaling、系统托盘、AppImage/deb |

### 3.3 UI 与技术映射示例

| UI 决策 | 技术实现 |
|---|---|
| 暗黑/亮色主题 | CSS 变量 + `prefers-color-scheme` + Tauri 主题插件 |
| 三栏布局 | Vue 3 组件化：Sidebar / MailList / Viewer |
| 多窗口 | Tauri `WebviewWindow` + 窗口类型标签（main/reader/compose/search）|
| 全局搜索 ⌘K | Vue Command Palette 组件 + Tauri 全局快捷键 |
| 邮件 HTML 沙箱 | `<iframe sandbox="allow-same-origin">` + `srcdoc` |
| 千万级搜索 | Tantivy 本地索引 + 增量更新 |

---

## 4. 自检清单

- [ ] 三份文档之间术语一致、数据一致。
- [ ] PRD 中的每个功能都有对应的 UI 规范或技术实现说明。
- [ ] UI 设计系统中的每个关键决策都在技术实现方案中有落地映射。
- [ ] Windows / macOS / Linux(Wayland) 跨平台支持完整。
- [ ] Mermaid 图表语法正确，中文标签加引号。
- [ ] 验收标准可测试，无空话。

---

## 5. 后续步骤

本 Spec 经用户批准后，调用 `writing-plans` 生成详细实施计划，然后按顺序生成/改写三份文档。
