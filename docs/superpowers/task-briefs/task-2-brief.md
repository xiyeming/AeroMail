# Task 2: 创建 `docs/UI设计系统.md`

**Files:**
- Create: `docs/UI设计系统.md`
- Source: `docs/UI方案.md` 全部内容

**Interfaces:**
- Consumes: `docs/UI方案.md` 中的设计语言、布局、颜色、字体、组件、动效
- Produces: 结构化的 UI 设计系统文档，供 `技术实现方案.md` 引用并映射

- [ ] **Step 1: 读取源文档并提取设计规范**

  读取 `docs/UI方案.md`，提取：
  - 设计语言（Fluent 2 + Linear + Arc Browser + Notion）
  - 布局体系（标准模式、双 2K 模式、Ultra Wide）
  - 视觉层级（Level 0/1/2）
  - 颜色系统（Dark Theme、Light Theme）
  - 字体系统（Linux Inter、中文 MiSans/HarmonyOS Sans）
  - 组件规范（Sidebar、邮件列表、详情页、Compose、Command Palette）
  - 动效规范（150ms/200ms/250ms）

- [ ] **Step 2: 编写 UI 设计系统章节**

  按以下结构写入 `docs/UI设计系统.md`：
  1. 设计定位
  2. 整体布局体系
  3. 视觉层级系统
  4. 颜色系统
  5. 字体系统
  6. 组件规范
     - Sidebar
     - 邮件列表 Compact Card
     - 邮件详情页 Header
     - Compose Workspace
     - Command Palette
     - **Status Bar（新增）**
     - **Toast 提示（新增）**
  7. 交互规范
     - 多窗口模式
     - Reading Mode
     - Hover Actions / 拖拽上传
  8. 动效规范
  9. 平台适配（Windows / macOS / Linux Wayland 视觉差异）

- [ ] **Step 3: 自检 UI 设计系统**

  验证：
  - [ ] 所有原 `UI方案.md` 内容已纳入
  - [ ] Status Bar 和 Toast 已补充
  - [ ] 颜色、字体有具体值或字体名
  - [ ] 布局有具体尺寸（px）
  - [ ] Mermaid 图表语法正确

## Global Constraints

- 项目代号：AeroMail
- 技术栈：Rust + Tauri v2 + Tokio + Vue 3 + TypeScript + Vite + Tailwind CSS + Shadcn UI
- 数据：SQLite (rusqlite) + Tantivy
- 协议：async-imap + lettre + mailparse
- 输出目录：`docs/`
- 所有 Mermaid 中文标签必须加引号
- 验收标准必须可测试，避免空话
- 本项目当前不是 git 仓库，无需 `git commit`，完成任务后直接保存文件
