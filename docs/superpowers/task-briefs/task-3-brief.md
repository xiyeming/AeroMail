# Task 3: 创建 `docs/技术实现方案.md`

**Files:**
- Create: `docs/技术实现方案.md`
- Source: `docs/技术需求.md` 中的架构、代码示例、Linux/Wayland 专项

**Interfaces:**
- Consumes: `docs/PRD.md` 的功能需求、`docs/UI设计系统.md` 的设计决策、`docs/技术需求.md` 的技术实现
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

## Global Constraints

- 项目代号：AeroMail
- 技术栈：Rust + Tauri v2 + Tokio + Vue 3 + TypeScript + Vite + Tailwind CSS + Shadcn UI
- 数据：SQLite (rusqlite) + Tantivy
- 协议：async-imap + lettre + mailparse
- 输出目录：`docs/`
- 所有 Mermaid 中文标签必须加引号
- 验收标准必须可测试，避免空话
- 本项目当前不是 git 仓库，无需 `git commit`，完成任务后直接保存文件
