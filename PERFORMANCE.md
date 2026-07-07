# AeroMail 性能分析报告

## 一、前端性能分析

### 1. 渲染性能

#### 1.1 邮件列表渲染
**位置**：`src/components/MailList.vue`

**当前实现**：
- 使用全量渲染，所有邮件条目直接渲染到 DOM
- 每页固定 50 封邮件（`PAGE_SIZE = 50`）
- 搜索过滤使用 `computed` 实时计算

**问题**：
- 当用户快速滚动或频繁搜索时，`displayedMails` 计算属性会频繁重新计算
- 大邮箱（上千封邮件）时，`filter` 操作在每次渲染周期都可能执行
- 没有虚拟滚动（virtual scrolling），大量 DOM 节点导致重排/重绘开销

**影响**：中低端设备上滚动帧率下降，搜索输入时 UI 卡顿

#### 1.2 图片处理
**位置**：`src/components/MailViewer.vue`

**当前实现**：
- 内联图片（`cid:`）通过 `inlineImageMap` 以 base64 Data URL  ̃形式缓存
- 图片加载采用并发控制（`CONCURRENT = 5`）

**问题**：
- `inlineImageMap` 在邮件切换时清空，但旧邮件的 base64 字符串可能仍被 Vue 的响应式系统追踪
- 大图片（如高分辨率照片）的 base64 编码会增加内存占用约 33%
- 没有图片懒加载或渐进式加载

**影响**：内存占用随打开邮件数量线性增长，长邮件会话中可能出现 GC 压力

#### 1.3 状态管理
**位置**：`src/stores/mail.ts`

**当前实现**：
- `mails` 是响应式 `ref<MailHeader[]>`，包含完整邮件头信息
- `folders` 同样为完整响应式数组
- 每次数据变更都通过 `mails.value = newMails` 或 `push(...)` 触发重渲染

**问题**：
- `mailStore.mails` 包含大量字符串字段（subject, fromName, fromAddress），响应式代理开销显著
- `loadMails` 中 `mails.value.push(...newMails)` 会逐个触发响应式更新（Vue 3 的 `reactive`/`ref` 对数组 push 的优化有限）
- `selectedMailIds` 在批量选择时可能累积到数百个元素，每次点击都触发数组复制

**影响**：文件夹切换、批量选择时 UI 线程阻塞

#### 1.4 事件监听
**位置**：`src/components/MailList.vue`

**当前实现**：
- `watch(() => mailStore.mails.length, ...)` 在邮件数量变化时重新设置 `IntersectionObserver`
- `watch(() => statusStore.syncingAccounts, ...)` 同步完成后刷新列表

**问题**：
- `setupInfiniteScroll` 在每次邮件长度变化时都被调用，即使 `scrollObserver` 已存在
- 同步完成后的刷新是顺序 `await`，多账户时会累积延迟

**影响**：同步完成后邮件列表刷新有可感知的延迟

### 2. 网络与 IPC

**位置**：`src/composables/useTauriInvoke.ts`

**当前实现**：
- 每个 Tauri IPC 调用都是独立的 `invoke` Promise
- 没有请求去重或取消机制

**问题**：
- 快速切换文件夹时，旧请求的响应可能晚于新请求，导致状态混乱（虽有 `currentLoadId` 防抖，但其他调用如 `loadFolders` 没有）
- 没有请求队列，高并发时前端发送请求过快可能导致后端积压

**影响**：快速导航时可能出现短暂的状态错乱

### 3. 内存泄漏风险

| 风险点 | 位置 | 说明 |
|--------|------|------|
| `inlineImageMap` | MailViewer.vue:68 | 切换邮件时清空，但旧 base64 字符串可能仍被闭包引用 |
| `extractedDomainsCache` | MailViewer.vue:431 | 模块级变量，无清理机制，随邮件切换累积 |
| `scrollObserver` | MailList.vue:74 | `disconnect` 在 `onUnmounted` 调用，但组件复用场景下可能未正确清理 |
| `contextMenu` | MailList.vue:56 | 全局点击监听未在代码中体现，需确认是否在 `onUnmounted` 移除 |

---

## 二、后端性能分析

### 1. 数据库层

#### 1.1 连接模型
**位置**：`src-tauri/src/db/pool.rs`

**当前实现**：
```rust
pub struct Database {
    connection: Mutex<Connection>,
}
```

**问题**：
- 全局单连接 SQLite，通过 `std::sync::Mutex` 序列化所有访问
- 每次操作都要 `lock()` → 执行 → `drop(conn)`
- 没有连接池，高并发时（如多账户同步 + 前端查询）会互相阻塞

**影响**：IMAP 同步时如果前端同时查询邮件列表，同步速度会下降

#### 1.2 查询效率

**问题**：
- `get_setting` 每次都要 `prepare` 语句（无缓存）
- `upsert_mail` 单条插入，大批量同步时（1000+ 邮件）事务开销大
- `count_mails_in_folder`、`get_max_uid` 等查询没有索引优化证据

**影响**：全量同步 1000 封邮件时，数据库操作可能成为瓶颈

### 2. IMAP 同步引擎

#### 2.1 并发模型
**位置**：`src-tauri/src/services/sync/mod.rs`

**当前实现**：
- 每个账户一个 `SyncWorker` Tokio task
- 账户间并行同步，但共享同一个数据库连接

**问题**：
- 多账户同时同步时，数据库 Mutex 成为竞争点
- `progress_tx` 使用 `mpsc::channel(100)`，改为 `try_send` 后缓冲区满时静默丢弃进度事件

**影响**：
- 多账户同步速度不如预期线性提升
- 大邮箱同步时进度条可能卡住（事件被丢弃）

#### 2.2 回填与增量同步
**位置**：`src-tauri/src/services/sync/worker.rs:553-653`

**当前实现**：
- 增量同步先回填旧邮件，再拉取新邮件
- 使用 `should_skip_folder` 基于 `MESSAGES`、`UIDVALIDITY`、`UIDNEXT` 判断是否跳过

**问题**：
- `build_sync_uid_set` 对每个文件夹都执行 `UID SEARCH SINCE ...`，即使文件夹可能没有新邮件
- 回填逻辑（`1:min_uid-1`）在 `min_uid` 较大时会生成非常大的 UID 集合

**影响**：文件夹多、邮件多的账户同步时间较长

#### 2.3 附件处理
**位置**：`src-tauri/src/services/sync/worker.rs:740-777`

**当前实现**：
- 每封邮件的附件同步：`create_dir_all` + 循环 `std::fs::write`
- 先 `delete_attachments` 再逐条 `insert_attachment`

**问题**：
- 单封邮件的附件处理没有批量优化
- 每个附件执行一次 SQL INSERT，无事务包裹
- `std::fs::write` 同步阻塞 Tokio 线程池

**影响**：附件多的邮件同步慢，且阻塞线程池

### 3. 内存与资源

| 问题 | 位置 | 说明 |
|------|------|------|
| `extractRemoteDomains` 无缓存清理 | MailViewer.vue:431 | 模块级 `extractedDomainsCache` 无限增长 |
| IMAP 会话复用 | imap_client.rs | 每个 `sync_once` 创建新连接，没有连接复用 |
| 附件目录 | worker.rs:750 | 每封邮件创建子目录，小文件多时 inode 占用高 |

### 4. 超时与容错

**位置**：`src-tauri/src/services/imap_client.rs`

**当前实现**：
- `TcpStream::connect` 和 TLS 握手没有超时
- `AccountConfig` 中虽然有 `connect_timeout_secs`、`read_timeout_secs` 字段，但 `imap_client.rs` 从未使用

**影响**：网络不佳时，同步线程可能无限挂起

---

## 三、优先级建议

### 高优先级（用户体验影响大）

1. **前端虚拟滚动**：邮件列表超过 100 封时应启用虚拟滚动，减少 DOM 节点
2. **后端单连接瓶颈**：SQLite 连接改为读写分离或至少使用 `r2d2`/`deadpool` 连接池
3. **IMAP 超时**：`imap_client.rs` 应用 `connect_timeout`/`read_timeout`
4. **`fetch_older_mails` UIDNEXT 清空**：已修复，但需验证

### 中优先级（性能提升明显）

5. **前端搜索优化**：对 `displayedMails` 增加防抖或 Web Worker
6. **批量数据库操作**：`upsert_mail`、`insert_attachment` 使用事务批量执行
7. **附件 I/O**：将 `std::fs::write` 改为 `tokio::fs::write`
8. **进度事件可靠性**：恢复 `.send().await` 或增大缓冲区 + 背压处理

### 低优先级（长期优化）

9. **内存缓存清理**：`extractedDomainsCache`、`inlineImageMap` 增加 LRU 或大小限制
10. **连接复用**：IMAP 连接在同步多个文件夹时保持长连接
11. **前端组件懒加载**：`AiAssistantPanel`、`TodoPanel` 等非核心面板改为异步组件

---

## 四、性能基准建议

| 指标 | 当前预期 | 目标 |
|------|----------|------|
| 首屏渲染 | < 500ms | < 300ms |
| 邮件列表滚动（60 封） | 30-40 fps | 60 fps |
| 搜索响应（1000 封） | 100-200ms | < 50ms |
| IMAP 全量同步（1000 封） | 30-60s | < 20s |
| 内存占用（打开 10 封邮件） | ~200MB | < 150MB |
| 数据库操作并发 | 1 连接 | 2-4 连接 |

---

## 五、总结

前端主要瓶颈在 **DOM 规模**（无虚拟滚动）和 **响应式开销**（大数组全量响应式）。后端主要瓶颈在 **单连接 SQLite** 和 **IMAP 超时缺失**。按优先级逐步优化可显著提升体验。
