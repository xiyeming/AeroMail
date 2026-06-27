<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted, watchEffect } from 'vue';

const props = defineProps<{
  html: string;
  className?: string;
  allowedDomains?: string[];
}>();

const emit = defineEmits<{
  'security-violation': [{ domain: string; blockedUri: string }];
  'remote-domains': [domains: string[]];
  'link-click': [url: string];
  load: [];
  selection: [{ text: string; clientX: number; clientY: number }];
}>();

const iframeRef = ref<HTMLIFrameElement | null>(null);

function buildCsp(): string {
  const allowed = (props.allowedDomains || []).map((d) => d.trim()).filter(Boolean);

  if (allowed.includes('*')) {
    return [
      "default-src 'none'",
      "img-src 'self' data: cid: http: https:",
      "style-src 'unsafe-inline' 'self' http: https:",
      "font-src 'self' http: https:",
      "media-src 'self' http: https:",
      "script-src 'none'",
    ].join('; ');
  }

  // Allow exact host, its direct subdomains, and (for deep hosts) sibling
  // subdomains under the parent domain.
  const hosts = allowed.flatMap((domain) => {
    const labels = domain.split('.');
    const entries = [domain, `*.${domain}`];
    if (labels.length >= 3) {
      entries.push(`*.${labels.slice(1).join('.')}`);
    }
    return entries;
  });

  const imgSrc = ["'self'", 'data:', 'cid:', ...hosts].join(' ');
  const styleSrc = ["'unsafe-inline'", "'self'", ...hosts].join(' ');
  const fontSrc = ["'self'", ...hosts].join(' ');
  const mediaSrc = ["'self'", ...hosts].join(' ');
  return [
    "default-src 'none'",
    `img-src ${imgSrc}`,
    `style-src ${styleSrc}`,
    `font-src ${fontSrc}`,
    `media-src ${mediaSrc}`,
    "script-src 'none'",
  ].join('; ');
}

const srcdoc = computed(() => {
  const csp = buildCsp();
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="${csp}">
  <style>
    :root { color-scheme: light; }
    body { margin: 0; padding: 0; }
    img { max-width: 100%; height: auto; }
    /* 居中邮件内容 */
    .email-wrapper { margin: 0 auto; }
  </style>
</head>
<body><div class="email-wrapper">${props.html}</div></body>
</html>`;
});

let observer: MutationObserver | null = null;
let attachInterval: ReturnType<typeof setInterval> | null = null;

function clearAttachInterval() {
  if (attachInterval) {
    clearInterval(attachInterval);
    attachInterval = null;
  }
}

function disconnectObserver() {
  if (observer) {
    observer.disconnect();
    observer = null;
  }
}

function detachViolationListener() {
  clearAttachInterval();
  detachSelectionListener();
  detachLinkClickListener();
  if (!iframeRef.value?.contentDocument) return;
  const doc = iframeRef.value.contentDocument;
  doc.removeEventListener('securitypolicyviolation', handleViolation);
  doc.removeEventListener('error', handleResourceError, true);
}

let selectionMouseupHandler: ((this: Window, ev: MouseEvent) => void) | null = null;

function detachSelectionListener() {
  try {
    const win = iframeRef.value?.contentWindow;
    if (win && selectionMouseupHandler) {
      win.removeEventListener('mouseup', selectionMouseupHandler);
    }
  } catch {
    // The iframe may have been navigated away; ignore cleanup errors.
  }
  selectionMouseupHandler = null;
}

function attachSelectionListener(): boolean {
  const win = iframeRef.value?.contentWindow;
  if (!win) return false;
  detachSelectionListener();
  selectionMouseupHandler = (e: MouseEvent) => {
    const text = win.getSelection()?.toString()?.trim();
    if (!text) return;
    emit('selection', { text, clientX: e.clientX, clientY: e.clientY });
  };
  try {
    win.addEventListener('mouseup', selectionMouseupHandler);
    return true;
  } catch {
    selectionMouseupHandler = null;
    return false;
  }
}

function ensureSelectionListener() {
  if (attachSelectionListener()) return;
  const interval = setInterval(() => {
    if (attachSelectionListener() || !iframeRef.value) {
      clearInterval(interval);
    }
  }, 100);
  setTimeout(() => clearInterval(interval), 3000);
}

function attachViolationListener() {
  if (!iframeRef.value?.contentDocument) return false;
  const doc = iframeRef.value.contentDocument;
  doc.removeEventListener('securitypolicyviolation', handleViolation);
  doc.addEventListener('securitypolicyviolation', handleViolation);
  doc.removeEventListener('error', handleResourceError, true);
  doc.addEventListener('error', handleResourceError, true);
  return true;
}

function ensureViolationListener() {
  clearAttachInterval();
  if (attachViolationListener()) return;
  // Retry for up to 3 seconds in case the contentDocument is not yet ready.
  attachInterval = setInterval(() => {
    if (attachViolationListener() || !iframeRef.value) {
      clearAttachInterval();
    }
  }, 100);
  setTimeout(() => clearAttachInterval(), 3000);
}

function handleViolation(event: SecurityPolicyViolationEvent) {
  const uri = event.blockedURI;
  if (!uri) return;
  try {
    const normalized = uri.startsWith('//')
      ? `https:${uri}`
      : /^https?:\/\//i.test(uri)
        ? uri
        : `https://${uri}`;
    const domain = new URL(normalized).hostname.toLowerCase();
    emit('security-violation', { domain, blockedUri: uri });
  } catch {
    // ignore malformed URIs
  }
}

function handleResourceError(event: Event) {
  const target = event.target as HTMLElement | null;
  if (!target) return;
  const tag = target.tagName?.toLowerCase();
  if (!tag || !['img', 'source', 'video', 'audio', 'iframe'].includes(tag)) return;
  const src =
    target.getAttribute('src') ||
    target.getAttribute('data-src') ||
    target.getAttribute('data-original') ||
    target.getAttribute('href');
  if (!src) return;
  try {
    const normalized = src.startsWith('//') ? `https:${src}` : src;
    const url = new URL(normalized);
    if (url.protocol === 'http:' || url.protocol === 'https:') {
      emit('security-violation', {
        domain: url.hostname.toLowerCase(),
        blockedUri: src,
      });
    }
  } catch {
    // ignore invalid URLs
  }
}

function extractRemoteDomainsFromDom(): string[] {
  if (!iframeRef.value?.contentDocument) return [];
  const doc = iframeRef.value.contentDocument;
  const domains = new Set<string>();

  const addUrl = (raw: string | null) => {
    if (!raw) return;
    try {
      const normalized = raw.startsWith('//') ? `https:${raw}` : raw;
      domains.add(new URL(normalized).hostname.toLowerCase());
    } catch {
      // ignore invalid URLs
    }
  };

  const elements = doc.querySelectorAll(
    '[src], [href], [srcset], [poster], [data-src], [data-original], [background]'
  );
  elements.forEach((el) => {
    addUrl(el.getAttribute('src'));
    addUrl(el.getAttribute('href'));
    addUrl(el.getAttribute('poster'));
    addUrl(el.getAttribute('data-src'));
    addUrl(el.getAttribute('data-original'));
    addUrl(el.getAttribute('background'));
    const srcset = el.getAttribute('srcset');
    if (srcset) {
      srcset.split(',').forEach((part) => {
        const url = part.trim().split(/\s+/)[0];
        addUrl(url);
      });
    }
  });

  // Inline style attributes and <style> blocks
  const extractCssUrls = (value: string) => {
    const urlRe = /url\(\s*(['"]?)([^"')]+)\1\s*\)/gi;
    let match: RegExpExecArray | null;
    while ((match = urlRe.exec(value)) !== null) {
      addUrl(match[2]);
    }
  };

  doc.querySelectorAll('[style]').forEach((el) => {
    const style = el.getAttribute('style');
    if (style) extractCssUrls(style);
  });

  Array.from(doc.styleSheets).forEach((sheet) => {
    try {
      const walkRules = (rules: CSSRuleList) => {
        Array.from(rules).forEach((rule) => {
          if (rule instanceof CSSStyleRule) {
            extractCssUrls(rule.cssText);
          } else if (rule instanceof CSSImportRule) {
            addUrl(rule.href);
          } else if (rule instanceof CSSMediaRule && rule.cssRules) {
            walkRules(rule.cssRules);
          }
        });
      };
      walkRules(sheet.cssRules);
    } catch {
      // Cross-origin or restricted stylesheets are ignored.
    }
  });

  return Array.from(domains).sort();
}

let linkClickHandler: ((this: Document, ev: MouseEvent) => void) | null = null;

function detachLinkClickListener() {
  try {
    const doc = iframeRef.value?.contentDocument;
    if (doc && linkClickHandler) {
      doc.removeEventListener('click', linkClickHandler);
    }
  } catch {
    // iframe 可能已经导航离开；忽略清理错误
  }
  linkClickHandler = null;
}

function attachLinkClickListener(): boolean {
  const doc = iframeRef.value?.contentDocument;
  if (!doc) return false;
  detachLinkClickListener();
  linkClickHandler = (e: MouseEvent) => {
    const target = e.target as HTMLElement;
    // 向上查找最近的 <a> 标签（即使 a 有 pointer-events:none，子元素仍可点击）
    const anchor = target.closest('a');
    if (!anchor) return;
    const href = anchor.getAttribute('href');
    if (!href) return;
    // 处理 http/https 链接和 mailto 链接
    if (/^https?:\/\//i.test(href) || /^mailto:/i.test(href)) {
      e.preventDefault();
      e.stopPropagation();
      emit('link-click', href);
    }
  };
  try {
    doc.addEventListener('click', linkClickHandler);
    return true;
  } catch {
    linkClickHandler = null;
    return false;
  }
}

function ensureLinkClickListener() {
  if (attachLinkClickListener()) return;
  const interval = setInterval(() => {
    if (attachLinkClickListener() || !iframeRef.value) {
      clearInterval(interval);
    }
  }, 100);
  setTimeout(() => clearInterval(interval), 3000);
}

function emitRemoteDomains() {
  const domains = extractRemoteDomainsFromDom();
  console.log('[SandboxedHtml] remote domains from DOM:', domains);
  if (domains.length > 0) {
    emit('remote-domains', domains);
  }
}

function adjustHeight() {
  emit('load');
  disconnectObserver();
  ensureViolationListener();
  ensureSelectionListener();
  ensureLinkClickListener();
  emitRemoteDomains();
  if (!iframeRef.value?.contentDocument?.body) return;

  const body = iframeRef.value.contentDocument.body;

  const applyHeight = () => {
    if (!iframeRef.value?.contentDocument?.body) return;
    const height = iframeRef.value.contentDocument.body.scrollHeight;
    iframeRef.value.style.height = `${Math.min(height + 20, 2000)}px`;
  };

  observer = new MutationObserver(applyHeight);
  observer.observe(body, {
    childList: true,
    subtree: true,
    attributes: true,
  });

  applyHeight();
}

watch(srcdoc, () => {
  // srcdoc changes trigger a re-load; height will be re-applied via @load.
  disconnectObserver();
  detachViolationListener();
  ensureSelectionListener();
});

watchEffect(() => {
  if (iframeRef.value) {
    ensureViolationListener();
  }
});

onMounted(() => {
  ensureViolationListener();
  adjustHeight();
});

onUnmounted(() => {
  disconnectObserver();
  detachViolationListener();
  detachLinkClickListener();
});

defineExpose({ iframeRef });
</script>

<template>
  <iframe
    ref="iframeRef"
    :class="className"
    :srcdoc="srcdoc"
    sandbox="allow-same-origin"
    style="width: 100%; border: none; min-height: 100px"
    title="Email content"
    @load="adjustHeight"
  />
</template>
