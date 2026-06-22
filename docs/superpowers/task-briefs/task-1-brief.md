# Task 1: 创建 `docs/PRD.md`

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
  6. 非功能性需求
  7. 验收标准（逐条可测试）
  8. 术语表

- [ ] **Step 3: 自检 PRD**

  验证：
  - [ ] 所有核心业务功能已覆盖
  - [ ] 多账户高级配置（TLS、Socket）已写入
  - [ ] 验收标准具体可验证
  - [ ] 无空话（如"系统应具有良好的性能"）

## Global Constraints

- 项目代号：AeroMail
- 技术栈：Rust + Tauri v2 + Tokio + Vue 3 + TypeScript + Vite + Tailwind CSS + Shadcn UI
- 数据：SQLite (rusqlite) + Tantivy
- 协议：async-imap + lettre + mailparse
- 输出目录：`docs/`
- 所有 Mermaid 中文标签必须加引号
- 验收标准必须可测试，避免空话
- 本项目当前不是 git 仓库，无需 `git commit`，完成任务后直接保存文件
