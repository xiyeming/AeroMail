# Task 3 Fix Brief

**Files:**
- Modify: `docs/技术实现方案.md`

**Interfaces:**
- Consumes: Task 3 reviewer findings
- Produces: Updated `docs/技术实现方案.md` with all Critical and Important issues fixed

## Critical Issues (Must Fix)

### 1. `docs/技术实现方案.md:1297` — `computed(async () => ...)` 非法

当前代码：
```typescript
const results = computed(async () => {
  if (!query.value) return [];
  return await invoke<SearchResult[]>('search_local_mails', {
    queryStr: query.value,
    filters: {},
  });
});
```

Vue 3 的 `computed` 不支持异步函数。请改为：
```typescript
const results = ref<SearchResult[]>([]);

watch(query, async (val) => {
  if (!val) {
    results.value = [];
    return;
  }
  results.value = await invoke<SearchResult[]>('search_local_mails', {
    queryStr: val,
    filters: {},
  });
}, { immediate: true });
```

### 2. `docs/技术实现方案.md:1775` — AES-GCM 固定 nonce 严重安全缺陷

当前代码使用 `Nonce::from_slice(b"unique nonce")`，这破坏了 AES-GCM 的安全性。请改为使用随机 nonce 并与密文一起存储：
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::aead::rand_core::RngCore;

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let mut ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| e.to_string())?;
    let mut result = nonce_bytes.to_vec();
    result.append(&mut ciphertext);
    Ok(result)
}
```

## Important Issues (Should Fix)

### 3. `docs/技术实现方案.md:1326` — Tauri v2 全局快捷键 API 错误

当前代码：
```typescript
register('CmdOrControl+K', () => {
  const palette = document.querySelector('command-palette');
  palette?.open();
});
```

请改为：
```typescript
import { register } from '@tauri-apps/plugin-global-shortcut';

register('CmdOrControl+K', (event) => {
  if (event.state === 'Pressed') {
    // 通过事件总线或全局状态触发 Command Palette
    window.dispatchEvent(new CustomEvent('aeromail:open-command-palette'));
  }
});
```

### 4. `docs/技术实现方案.md:370` — 正则表达式语法错误

当前代码中 `
` 应为空格：
```rust
let intercepted = html.replace(
    r#"<img([^>]*) src=["'](http[^"']*)["']"#,
    r#"<img$1 data-src="$2" src="placeholder.png" class="lazy-mail-img""#
);
```

### 5. `docs/技术实现方案.md:1166` — `NSVisualEffectMaterial::Sidebar` 未导入

在 Rust 代码示例中补充导入：
```rust
#[cfg(target_os = "macos")]
use window_vibrancy::NSVisualEffectMaterial;
```

### 6. `docs/技术实现方案.md:1585` — Tauri v2 `MenuItem::new` 参数错误

当前代码：
```rust
file_menu.append(&MenuItem::new(app, "new_mail", "New Mail", true, Some("CmdOrControl+N"))?)?;
```

Tauri v2 的 MenuItem API 不同。请改为：
```rust
let new_mail = MenuItem::with_id(app, "new_mail", "New Mail", true, Some("CmdOrControl+N"))?;
file_menu.append(&new_mail)?;
```

### 7. 补充 Reading Mode 实现说明

在「UI 设计系统落地映射」章节新增 6.10 节「Reading Mode」，包含：
- 快捷键 `Ctrl/Cmd + Shift + R`
- Vue 组件中 `isReadingMode` 状态控制 Sidebar 和 MailList 的显示/隐藏
- Mail Viewer 宽度扩展至 100%
- 退出方式：Esc 键、再次点击阅读模式按钮

示例代码：
```vue
<template>
  <div class="flex h-screen">
    <AppSidebar v-show="!isReadingMode" />
    <MailList v-show="!isReadingMode" />
    <MailViewer :class="isReadingMode ? 'flex-1' : 'w-[480px]'" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
const isReadingMode = ref(false);

function toggleReadingMode() {
  isReadingMode.value = !isReadingMode.value;
}
</script>
```

### 8. 补充 Linux Wayland 强制配置

在 7.3.1 节中增加 Tauri 构建和运行时的完整说明：

```bash
# 运行时强制 Wayland
export GDK_BACKEND=wayland
export WEBKIT_DISABLE_COMPOSITING_MODE=0

# 构建时确保 WebKitGTK 使用 Wayland
cargo tauri build
```

说明：Tauri v2 在 Linux 上依赖 WebKitGTK，上述环境变量确保 Webview 走原生 Wayland 而非 XWayland。

## Minor Issues (Fix if Time Allows)

### 9. 补充 PRD 中缺失的功能实现说明

在「核心模块设计」或对应章节中补充以下 P0/P1 功能的实现说明：
- `READ-03` 纯文本/HTML 切换：在 `MailViewer.vue` 中添加 `viewMode: 'html' | 'text'` 状态
- `COMP-02` Markdown 实时转换：集成 `marked` 或 `markdown-it`，在 Compose Workspace 中添加 Rich Text / Markdown 模式切换
- `COMP-07` 联系人自动补全：从 `from_address`/`to_addresses` 中提取地址，构建本地联系人索引
- `SYNC-07` 同步策略配置：在 `AccountConfig` 中添加 `sync_interval_secs: u64` 和 `excluded_folders: Vec<String>`
- `ACC-07` 账户导入/导出：Tauri Command `export_accounts` / `import_accounts`，JSON 格式，密码字段脱敏

### 10. 补充 Tailwind 配置中的语义颜色

在 6.2 节的 `tailwind.config.ts` 中补充：
```typescript
colors: {
  background: 'var(--background)',
  panel: 'var(--panel)',
  card: 'var(--card)',
  border: 'var(--border)',
  primary: 'var(--primary)',
  text: 'var(--text)',
  'text-secondary': 'var(--text-secondary)',
  muted: 'var(--muted)',
  success: 'var(--success)',
  warning: 'var(--warning)',
  danger: 'var(--danger)',
  info: 'var(--info)',
  overlay: 'var(--overlay)',
  glass: 'var(--glass)',
}
```

### 11. 修正 Linux Fractional Scaling 说明

在 7.3.5 节中删除或修正无效的 CSS `@supports (-webkit-font-smoothing: subpixel-antialiased)` 代码，改为说明：
> Wayland 的 Fractional Scaling 由 compositor 和 WebKitGTK 合成器自动处理，前端无需干预。应用只需确保 `GDK_BACKEND=wayland` 并启用 GPU 合成即可。

## Output

修改完成后，将更新后的 `docs/技术实现方案.md` 保存。无需 git commit。
