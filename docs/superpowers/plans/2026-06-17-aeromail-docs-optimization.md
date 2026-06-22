# AeroMail 文档优化实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 `docs/技术需求.md` 与 `docs/UI方案.md` 优化为「PRD + UI 设计系统 + 技术实现方案」三份相互吻合的文档，并补充 Windows / macOS / Linux(Wayland) 跨平台支持。

**Architecture:** 以现有两份文档为素材，去重、对齐术语、补齐缺失章节；在 `技术实现方案.md` 中增设「UI 设计系统落地映射」章节，确保每个 UI 决策都有明确技术实现；最终删除或归档旧文档。

**Tech Stack:** Markdown、Mermaid

## Global Constraints

- 项目代号：AeroMail
- 技术栈：Rust + Tauri v2 + Tokio + Vue 3 + TypeScript + Vite + Tailwind CSS + Shadcn UI
- 数据：SQLite (rusqlite) + Tantivy
- 协议：async-imap + lettre + mailparse
- 输出目录：`docs/`
- 所有 Mermaid 中文标签必须加引号
- 验收标准必须可测试，避免空话
- 本项目当前不是 git 仓库，无需 `git commit`，每完成一个任务直接保存文件即可

---

## Task 1: 创建 `docs/PRD.md`

**Files:**
- Create: `docs/PRD.md`
- Source: `docs/技术需求.md`（第 1~26 行核心功能与 UI/UX 需求）

**Interfaces:**
- Consumes: 现有 `docs/技术需求.md` 中的业务功能、Linux/Wayland 专项、性能目标
- Produces: 一份完整的产品需求文档，供 `UI设计系统.md` 和 `技术实现方案.md` 引用

- [ ] **Step 1: 读取源文档并提取需求**

  读取 `docs/技术需求.md`，提取：
  - 项目代号、产品关键词、目标用户
  - 核心业务功能（多账户、同步、HTML 渲染、搜索、写信、多窗口）
  - 功能边界与性能目标

- [ ] **Step 2: 编写 PRD 章节**

  按以下结构写入 `docs/PRD.md`：
  1. 文档信息
  2. 产品定位
  3. 目标用户与使用场景
  4. 核心业务功能
     - 多账户连接：预设国内外主流厂商、OAuth2、传统密码、高级配置（TLS/端口/Socket）
     - 多线程本地同步引擎
     - 100% HTML 邮件渲染
     - 千万级本地全文检索
     - 全功能写信编辑器
     - 多窗口独立操作
  5. 功能清单与优先级（P0/P1/P2，按模块分组）
  6. 非功能性需求（性能、安全、可访问性）
  7. 验收标准（逐条可测试）
  8. 术语表

- [ ] **Step 3: 自检 PRD**

  验证：
  - [ ] 所有核心业务功能已覆盖
  - [ ] 多账户高级配置（TLS、Socket）已写入
  - [ ] 验收标准具体可验证
  - [ ] 无空话（如"系统应具有良好的性能"）

---

## Task 2: 创建 `docs/UI设计系统.md`

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

---

## Task 3: 创建 `docs/技术实现方案.md`

**Files:**
- Create: `docs/技术实现方案.md`
- Source: `docs/技术需求.md` 中的架构、代码示例、Linux/Wayland 专项

**Interfaces:**
- Consumes: `docs/PRD.md` 的功能需求、`docs/UI设计系统.md` 的设计决策
- Produces: 完整的技术实现方案，包含 UI 设计系统落地映射与跨平台专项

- [ ] **Step 1: 读取源文档并提取技术方案**

  读取 `docs/技术需求.md`，提取：
  - 架构拓扑（重后端、轻前端）
  - 核心模块与选型（tokio、async-imap、lettre、mailparse、rusqlite、tantivy）
  - Tauri Command 示例
  - HTML 渲染沙箱方案
  - Linux/Wayland 专项配置

- [ ] **Step 2: 编写技术实现方案章节**

  按以下结构写入 `docs/技术实现方案.md`：
  1. 技术栈选型
  2. 系统架构（含 Mermaid 拓扑图）
  3. 核心模块设计
     - 账户管理（预设厂商、OAuth2、高级配置）
     - 同步引擎
     - 邮件解析与存储
     - 全文检索
     - SMTP 发信与草稿
     - 通知与系统托盘
  4. 数据模型（ER 图 + 表结构）
  5. 接口设计（Tauri Command 清单）
  6. **UI 设计系统落地映射**
     - 主题切换 → CSS 变量 + Tauri 主题 API
     - 三栏布局 → Vue 3 组件划分
     - 多窗口 → Tauri `WebviewWindow` + 窗口类型
     - 毛玻璃 → Windows Mica / macOS vibrancy / Linux 透明窗口
     - 字体系统 → 跨平台字体栈配置
     - Command Palette → Vue 组件 + Tauri 全局快捷键
     - Status Bar / Toast → 全局状态 + Tauri 通知插件
  7. 跨平台专项
     - Windows：Mica/Acrylic、MSIX/EXE、通知
     - macOS：vibrancy、原生菜单栏、DMG/签名、通知中心
     - Linux(Wayland)：原生 Wayland、系统托盘、Fractional Scaling、AppImage/deb
  8. 安全设计
  9. 性能与优化
  10. 开发与部署

- [ ] **Step 3: 自检技术实现方案**

  验证：
  - [ ] 每个 PRD 功能都有对应实现说明
  - [ ] 每个 UI 设计系统关键决策都有落地映射
  - [ ] Windows / macOS / Linux 专项完整
  - [ ] Mermaid 图表语法正确
  - [ ] 代码示例可运行（语法正确）

---

## Task 4: 删除旧文档并更新索引

**Files:**
- Delete: `docs/技术需求.md`
- Delete: `docs/UI方案.md`
- Create/Modify: `docs/README.md`（如不存在则创建，用于索引新文档）

**Interfaces:**
- Consumes: 新创建的三份文档
- Produces: 清理后的 docs 目录与索引说明

- [ ] **Step 1: 确认新文档已生成且自检通过**

  确认：
  - `docs/PRD.md` 存在且内容完整
  - `docs/UI设计系统.md` 存在且内容完整
  - `docs/技术实现方案.md` 存在且内容完整

- [ ] **Step 2: 删除旧文档**

  删除：
  - `docs/技术需求.md`
  - `docs/UI方案.md`

- [ ] **Step 3: 创建/更新 `docs/README.md`**

  写入索引：
  - 项目简介
  - 新文档清单与作用
  - 阅读顺序建议

---

## Task 5: 最终一致性检查

**Files:**
- Read: `docs/PRD.md`
- Read: `docs/UI设计系统.md`
- Read: `docs/技术实现方案.md`

- [ ] **Step 1: 术语一致性检查**

  确认以下术语在三份文档中一致：
  - 项目代号：AeroMail
  - 技术栈：Rust + Tauri v2 + Vue 3 + TypeScript
  - 多账户高级配置项名称
  - 颜色变量名
  - 窗口类型名（Main / Reader / Compose / Search）

- [ ] **Step 2: 需求-设计-技术覆盖检查**

  逐条核对 PRD 中的 P0/P1 功能：
  - [ ] 在 UI 设计系统中有对应页面/组件说明
  - [ ] 在技术实现方案中有对应模块/接口说明

- [ ] **Step 3: 跨平台支持检查**

  确认：
  - [ ] Windows 专项已覆盖 Mica/Acrylic、安装包、通知
  - [ ] macOS 专项已覆盖 vibrancy、菜单栏、签名
  - [ ] Linux(Wayland) 专项已覆盖原生渲染、Fractional Scaling、托盘、通知

- [ ] **Step 4: Mermaid 与格式检查**

  确认：
  - [ ] 所有 Mermaid 中文标签加引号
  - [ ] 无未闭合的代码块
  - [ ] 标题层级连续

---

## Spec Coverage

| Spec 要求 | 对应任务 |
|---|---|
| 拆分为 PRD / UI 设计系统 / 技术实现方案 | Task 1, 2, 3 |
| UI 设计系统落地映射 | Task 3 Step 2 第 6 章 |
| Windows / macOS / Linux(Wayland) 跨平台支持 | Task 3 Step 2 第 7 章、Task 5 Step 3 |
| 多账户预设厂商 + 高级配置（TLS/Socket） | Task 1 Step 2 第 4 章、Task 3 Step 2 第 3 章 |
| Status Bar / Toast | Task 2 Step 2 第 6 章、Task 3 Step 2 第 6 章 |
| 术语一致、数据一致 | Task 5 |

## Placeholder Scan

- 无 TBD / TODO
- 无"适当处理"等模糊描述
- 所有代码/配置示例已给出或可推导

## Type Consistency

- 项目代号统一为 AeroMail
- 技术栈统一为 Rust + Tauri v2 + Vue 3 + TypeScript + Vite + Tailwind CSS + Shadcn UI
- 窗口类型统一为 Main / Reader / Compose / Search
