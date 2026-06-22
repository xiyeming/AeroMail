# AeroMail Phase 4: 富文本写信 + SMTP 发信设计文档

> 状态：已评审通过  
> 日期：2026-06-20  
> 范围：新建邮件、草稿自动保存、IMAP Drafts 同步、回复/转发、附件、SMTP 发信

---

## 1. 背景与目标

AeroMail 已完成邮件阅读、搜索、翻译、AI 助手等能力。Phase 4 补充**写信**能力，使用户能够在应用内撰写、保存、发送富文本邮件，并支持回复/转发与普通附件/内嵌图片。

成功标准：
- 用户可通过富文本编辑器撰写邮件，输出干净 HTML。
- 本地草稿自动保存，支持断网恢复。
- 草稿可同步到 IMAP Drafts 文件夹。
- 支持回复、回复全部、转发，自动引用原文并填充收件人/主题。
- 支持普通文件附件与内嵌图片。
- 通过账号配置的 SMTP 服务器成功发送邮件。

---

## 2. 关键决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 富文本库 | Tiptap | Vue 3 生态首选，基于 ProseMirror，可定制、输出干净 HTML |
| 后端架构 | 独立 `services/compose/` | 边界清晰，可离线编辑，SMTP OAuth2 可独立实现，便于测试 |
| MIME/SMTP 库 | `lettre` | 支持 SMTP + OAuth2 XOAUTH2、异步、multipart 构建 |
| 附件存储 | 文件系统 | 避免大附件撑爆 SQLite；元数据存 drafts 表 |
| IMAP 草稿同步 | 自动 debounce | 本地保存后延迟 5 秒触发后台 IMAP APPEND |
| 发信后草稿 | 删除 | 发送成功后清理本地草稿与附件文件 |
| 数据模型 | 独立 `models/compose.rs` | 与 Mail 模型解耦，字段更贴合写信场景 |

---

## 3. 数据模型与类型

### 3.1 Rust 模型

新建 `src-tauri/src/models/compose.rs`：

```rust
pub struct ComposeDraft {
    pub id: String,
    pub account_id: String,
    pub reply_context: Option<ReplyContext>,
    pub subject: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub body_html: String,
    pub body_text: String,
    pub attachments: Vec<AttachmentDraft>,
    pub saved_at: i64,
    pub synced_at: Option<i64>,
    pub remote_uid: Option<u32>,
}

pub struct ReplyContext {
    pub original_mail_id: String,
    pub kind: ReplyKind,
}

pub enum ReplyKind { Reply, ReplyAll, Forward }

pub struct AttachmentDraft {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub local_path: Option<String>,
    pub content_id: Option<String>,
    pub is_inline: bool,
    pub preview_url: Option<String>,
}

pub struct ComposeDraftSummary {
    pub id: String,
    pub account_id: String,
    pub subject: String,
    pub to: Vec<String>,
    pub saved_at: i64,
    pub has_attachments: bool,
}

pub struct SendMailRequest {
    pub draft_id: String,
}
```

### 3.2 数据库扩展

扩展 `drafts` 表：

```sql
ALTER TABLE drafts ADD COLUMN bcc_addresses TEXT;
ALTER TABLE drafts ADD COLUMN reply_context_json TEXT;
ALTER TABLE drafts ADD COLUMN synced_at INTEGER;
ALTER TABLE drafts ADD COLUMN remote_uid INTEGER;
```

- `to_addresses`/`cc_addresses`/`bcc_addresses`：JSON 数组字符串。
- `attachments_json`：附件元数据数组。
- `reply_context_json`：`{ original_mail_id, kind }`。

### 3.3 附件文件存储

- 路径：`app_data_dir/drafts/<draft_id>/<attachment_id>/<filename>`
- 草稿删除、发送成功或用户清空附件时清理目录。
- 大文件不进入 SQLite，避免数据库膨胀。

---

## 4. 后端服务与命令

### 4.1 服务模块

新建 `src-tauri/src/services/compose/`：

| 文件 | 职责 |
|------|------|
| `mod.rs` | 模块导出与 `ComposeService` 入口 |
| `draft.rs` | 本地草稿 CRUD、附件文件管理、目录清理 |
| `mime_builder.rs` | 使用 `lettre` 构建 MIME：multipart/alternative + multipart/mixed，处理内嵌图片与普通附件 |
| `smtp_sender.rs` | SMTP 连接（TLS/STARTTLS/None），密码与 OAuth2 XOAUTH2 认证 |
| `imap_draft_sync.rs` | 后台 IMAP APPEND 到 Drafts；存在 `remote_uid` 时先删除旧邮件再上传新邮件 |

### 4.2 Tauri 命令

新建 `src-tauri/src/commands/compose.rs`：

```rust
save_draft(draft: ComposeDraft) -> Result<ComposeDraft, ErrorPayload>
get_drafts(account_id: Option<String>) -> Result<Vec<ComposeDraftSummary>, ErrorPayload>
get_draft(draft_id: String) -> Result<ComposeDraft, ErrorPayload>
delete_draft(draft_id: String) -> Result<(), ErrorPayload>
send_mail(req: SendMailRequest) -> Result<(), ErrorPayload>
prepare_reply(mail_id: String, kind: ReplyKind) -> Result<ComposeDraft, ErrorPayload>
sync_draft_to_imap(draft_id: String) -> Result<(), ErrorPayload>
```

### 4.3 发信流程

1. 根据 `draft_id` 读取完整草稿与附件。
2. `mime_builder` 构造 MIME 消息：
   - `multipart/mixed` 根节点
   - `multipart/alternative` 包含 `text/plain`（从 HTML 提取或同步维护）与 `text/html`
   - 普通附件作为 `application/*` part
   - 内嵌图片作为 `image/*` part，Content-ID 匹配 HTML 中的 `cid:` 引用
3. `smtp_sender` 建立 SMTP 连接，按账号 TLS 模式与认证方式发送。
4. 发送成功后：
   - 删除本地草稿记录
   - 删除附件文件目录
   - （可选后续阶段）APPEND 到 IMAP Sent 文件夹

### 4.4 IMAP 草稿同步流程

- 本地 `save_draft` 完成后，通过 debounce（5 秒）触发 `ImapDraftSync` 任务。
- 任务从 SQLite 读取最新草稿内容，构建 MIME。
- 若 `remote_uid` 存在：
  1. SELECT Drafts 文件夹
  2. `STORE <uid> +FLAGS (\\Deleted)`
  3. `EXPUNGE`
- 执行 `APPEND Drafts {mime_bytes}`，记录返回的 UID 到 `remote_uid` 与 `synced_at`。

### 4.5 回复/转发草稿生成

`prepare_reply` 逻辑：

- 读取原邮件 `MailDetail`。
- 默认账号：接收原邮件的账号。
- 主题前缀：`Re:` / `Fwd:`。
- 收件人：
  - Reply：原邮件 `From`
  - ReplyAll：原邮件 `From` + `To`（排除自己）+ `Cc`
  - Forward：空，由用户填写
- 正文：在编辑器底部插入引用块（原邮件主题、发件人、日期、正文文本）。

---

## 5. 前端组件与流程

### 5.1 新增组件

| 组件/文件 | 职责 |
|-----------|------|
| `src/views/ComposeView.vue` | 写信页面容器 |
| `src/components/compose/ComposeEditor.vue` | Tiptap 编辑器 + 工具栏 |
| `src/components/compose/ComposeHeader.vue` | 账号选择、主题、To/Cc/Bcc 切换 |
| `src/components/compose/RecipientInput.vue` | 邮箱地址标签输入，支持粘贴多地址 |
| `src/components/compose/ComposeAttachmentList.vue` | 普通附件与内嵌图片列表 |
| `src/stores/compose.ts` | 当前草稿状态、自动保存、发送流程 |
| `src/types/compose.ts` | TypeScript 类型 |

### 5.2 路由

```typescript
/compose              // 新建邮件
/compose/:draftId     // 继续编辑草稿
/reply/:mailId        // 回复
/reply-all/:mailId    // 回复全部
/forward/:mailId      // 转发
```

### 5.3 关键交互

- **Tiptap 输出**：HTML，工具栏支持 bold、italic、strike、heading、list、blockquote、link、undo/redo。
- **内嵌图片**：粘贴/拖拽图片时，上传到后端作为内联附件，返回 `cid:` 引用后插入编辑器。
- **普通附件**：文件选择或拖拽添加，保存到本地草稿目录。
- **自动保存**：内容变化 2 秒 debounce 调用 `save_draft`；5 秒 debounce 触发 `sync_draft_to_imap`。
- **发送校验**：至少存在一个有效收件人；前端先做基本邮箱格式校验。
- **发送成功**：toast 提示并返回收件箱。

### 5.4 编辑器工具栏

使用 Tiptap StarterKit + Link + Image，工具栏按钮：

- 撤销 / 重做
- 加粗 / 斜体 / 删除线
- 标题（H1/H2/正文）
- 无序列表 / 有序列表
- 引用块
- 插入/编辑链接

---

## 6. 错误处理

### 6.1 后端错误变体

新增 `AeroError`：

```rust
SmtpConnectionFailed(String)
SmtpAuthFailed(String)
InvalidRecipient(String)
DraftNotFound(String)
AttachmentNotFound(String)
InvalidAttachment(String)
MailBuilderFailed(String)
ImapAppendFailed(String)
```

通过 `to_payload()` 映射为错误码返回前端。

### 6.2 前端处理

- 发送失败：显示可重试的 toast。
- 草稿保存失败：提示用户但不丢失编辑器内容。
- 收件人格式错误：输入框内联提示。
- i18n：在 `en.json` 与 `zh-CN.json` 中新增对应 key，运行 `pnpm i18n:check` 校验。

---

## 7. 安全考量

- 附件路径严格限制在 `app_data_dir/drafts/<draft_id>/` 下，防止路径遍历。
- HTML 由 Tiptap 生成，发送前不过滤 script；但邮件客户端收到后按各自策略处理。AeroMail 发信端只保证 MIME 合规。
- SMTP 密码复用账号表中 `auth_credentials_encrypted`，与现有 IMAP 逻辑一致（当前按明文处理，后续统一加密）。

---

## 8. 验证计划

- **单元测试**：新增 `lettre` MIME 构建测试，覆盖纯文本、HTML、单附件、内嵌图片组合。
- **手动端到端**：
  - 新建邮件 → 保存草稿 → 关闭应用 → 重新打开恢复草稿
  - 添加普通附件与内嵌图片 → 发送 → 收件方查看内容与附件
  - 回复/转发 → 验证收件人、主题前缀、引用块
  - 多账号切换发信
- **代码质量**：
  - `pnpm type-check`
  - `pnpm lint`
  - `cargo clippy`
  - `pnpm i18n:check`

---

## 9. 风险与后续迭代

| 风险 | 缓解 |
|------|------|
| `lettre` 与现有 `imap` crate 的 TLS 配置差异 | 统一封装 TLS 连接器逻辑 |
| OAuth2 access_token 过期 | 本期只使用当前 token；刷新逻辑后续与 IMAP OAuth2 统一实现 |
| IMAP Drafts 同步频繁导致草稿冗余 | 通过 `remote_uid` 删除旧邮件后再 APPEND |
| 大附件内存占用 | 文件流式读取，避免一次性加载到内存 |

后续迭代可考虑：
- 自动保存到 IMAP Sent 文件夹
- 地址簿自动补全
- 模板/签名
- 定时发送

---

## 10. 相关文件

- `src-tauri/src/models/compose.rs`（新增）
- `src-tauri/src/services/compose/`（新增）
- `src-tauri/src/commands/compose.rs`（新增）
- `src-tauri/src/db/schema.rs`（扩展 drafts 表）
- `src-tauri/src/error.rs`（新增错误变体）
- `src-tauri/src/lib.rs`、`src-tauri/src/commands/mod.rs`、`src-tauri/src/services/mod.rs`（注册模块与命令）
- `src-tauri/Cargo.toml`（新增 `lettre` 依赖）
- `src/views/ComposeView.vue`（新增）
- `src/components/compose/*.vue`（新增）
- `src/stores/compose.ts`、`src/types/compose.ts`（新增）
- `src/router.ts`（新增路由）
- `src/components/MailViewer.vue`（添加回复/转发按钮）
- `src/i18n/locales/en.json`、`src/i18n/locales/zh-CN.json`（新增文案）
