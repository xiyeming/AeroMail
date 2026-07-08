<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';

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

function buildCsp(scriptNonce: string): string {
  const allowed = (props.allowedDomains || []).map((d) => d.trim()).filter(Boolean);

  const scriptSrc = scriptNonce
    ? `'nonce-${scriptNonce}'`
    : "'none'";

  if (allowed.includes('*')) {
    return [
      "default-src 'none'",
      "img-src 'self' data: cid: http: https:",
      "style-src 'unsafe-inline' 'self' http: https:",
      "font-src 'self' http: https:",
      "media-src 'self' http: https:",
      `script-src ${scriptSrc}`,
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

  const imgSrc = ["'self'", 'data:', 'cid:', 'http:', 'https:', ...hosts].join(' ');
  const styleSrc = ["'unsafe-inline'", "'self'", 'http:', 'https:', ...hosts].join(' ');
  const fontSrc = ["'self'", ...hosts].join(' ');
  const mediaSrc = ["'self'", ...hosts].join(' ');
  return [
    "default-src 'none'",
    `img-src ${imgSrc}`,
    `style-src ${styleSrc}`,
    `font-src ${fontSrc}`,
    `media-src ${mediaSrc}`,
    `script-src ${scriptSrc}`,
  ].join('; ');
}

function sanitizeHtml(html: string): string {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');

  // 移除可能嵌套脚本或加载外部资源的标签
  doc.querySelectorAll('script, noscript, iframe, object, embed').forEach((el) => el.remove());

  // 移除危险 link rel
  doc.querySelectorAll('link').forEach((el) => {
    const rel = el.getAttribute('rel')?.toLowerCase() ?? '';
    if (rel.includes('import') || rel.includes('modulepreload') || rel.includes('prefetch')) {
      el.remove();
      return;
    }
    const href = el.getAttribute('href')?.trim().toLowerCase() ?? '';
    if (
      href.startsWith('javascript:') ||
      href.startsWith('data:text/html') ||
      href.startsWith('vbscript:')
    ) {
      el.remove();
    }
  });

  // 移除 meta refresh 等可能导致跳转的 meta
  doc.querySelectorAll('meta[http-equiv]').forEach((el) => {
    const equiv = el.getAttribute('http-equiv')?.toLowerCase();
    if (equiv === 'refresh' || equiv === 'content-security-policy') {
      el.remove();
    }
  });

  // 移除内联事件处理器与危险伪协议
  const dangerousAttrs = new Set([
    'onabort', 'onblur', 'oncancel', 'oncanplay', 'oncanplaythrough', 'onchange', 'onclick',
    'onclose', 'oncontextmenu', 'oncuechange', 'ondblclick', 'ondrag', 'ondragend', 'ondragenter',
    'ondragleave', 'ondragover', 'ondragstart', 'ondrop', 'ondurationchange', 'onemptied',
    'onended', 'onerror', 'onfocus', 'onformdata', 'oninput', 'oninvalid', 'onkeydown',
    'onkeypress', 'onkeyup', 'onload', 'onloadeddata', 'onloadedmetadata', 'onloadstart',
    'onmousedown', 'onmouseenter', 'onmouseleave', 'onmousemove', 'onmouseout', 'onmouseover',
    'onmouseup', 'onmousewheel', 'onpause', 'onplay', 'onplaying', 'onprogress', 'onratechange',
    'onreset', 'onresize', 'onscroll', 'onsecuritypolicyviolation', 'onseeked', 'onseeking',
    'onselect', 'onslotchange', 'onstalled', 'onsubmit', 'onsuspend', 'ontimeupdate', 'ontoggle',
    'onvolumechange', 'onwaiting', 'onwheel',
  ]);

  function walk(node: Element) {
    // 同时处理 svg:script 等跨命名空间的 script 标签
    if (node.tagName.toLowerCase() === 'script') {
      node.remove();
      return;
    }

    for (const attr of Array.from(node.attributes)) {
      const name = attr.name.toLowerCase();
      if (dangerousAttrs.has(name) || name.startsWith('on')) {
        node.removeAttribute(attr.name);
      }
      if (
        name === 'href' ||
        name === 'src' ||
        name === 'action' ||
        name === 'formaction' ||
        name === 'background' ||
        name === 'dynsrc' ||
        name === 'lowsrc'
      ) {
        const value = attr.value.trim().toLowerCase();
        if (
          value.startsWith('javascript:') ||
          value.startsWith('data:text/html') ||
          value.startsWith('vbscript:') ||
          value.startsWith('mocha:') ||
          value.startsWith('livescript:')
        ) {
          node.removeAttribute(attr.name);
        }
      }
    }
    Array.from(node.children).forEach(walk);
  }

  [doc.documentElement, doc.head, doc.body].forEach((root) => {
    if (root) walk(root);
  });

  // 把文本节点中的裸 URL（如 https://pis.baiwang.com/...）转换成可点击的 <a> 链接。
  // 邮件客户端经常把链接以纯文本形式展示，导致用户无法点击。
  linkifyTextNodes(doc.body, doc);

  return doc.body.innerHTML;
}

function linkifyTextNodes(root: Node, doc: Document) {
  const urlRegex = /(https?:\/\/[^\s<>"{}|\\^`[\]]+)/gi;
  const skipTags = new Set(['a', 'area', 'script', 'style', 'pre', 'code']);

  function walk(node: Node) {
    if (node.nodeType === Node.TEXT_NODE) {
      const text = node.textContent ?? '';
      if (!urlRegex.test(text)) return;
      urlRegex.lastIndex = 0;

      const fragment = doc.createDocumentFragment();
      let lastIndex = 0;
      let match: RegExpExecArray | null;
      while ((match = urlRegex.exec(text)) !== null) {
        const [fullMatch] = match;
        const offset = match.index;
        if (offset > lastIndex) {
          fragment.appendChild(doc.createTextNode(text.slice(lastIndex, offset)));
        }
        const anchor = doc.createElement('a');
        anchor.href = fullMatch;
        anchor.textContent = fullMatch;
        anchor.style.color = 'var(--accent)';
        anchor.style.textDecoration = 'underline';
        fragment.appendChild(anchor);
        lastIndex = offset + fullMatch.length;
      }
      if (lastIndex < text.length) {
        fragment.appendChild(doc.createTextNode(text.slice(lastIndex)));
      }
      node.parentNode?.replaceChild(fragment, node);
      return;
    }

    if (node.nodeType !== Node.ELEMENT_NODE) return;
    const el = node as Element;
    if (skipTags.has(el.tagName.toLowerCase())) return;
    Array.from(node.childNodes).forEach(walk);
  }

  Array.from(root.childNodes).forEach(walk);
}

const srcdoc = computed(() => {
  // WebKit/Safari 在 sandbox="allow-same-origin"（不含 allow-scripts）的 iframe 中
  // 会阻止父页面脚本向 iframe 内元素附加事件监听器（Bug 218086）。
  // 我们通过允许脚本执行但配合 CSP nonce 仅允许我们自己的可信脚本运行，
  // 由 iframe 内部脚本把链接点击通过 postMessage 转发给父页面。
  const nonce =
    typeof crypto !== 'undefined' && 'randomUUID' in crypto
      ? crypto.randomUUID()
      : `${Math.random().toString(36).slice(2)}${Date.now().toString(36)}`;
  const csp = buildCsp(nonce);
  console.debug('[SandboxedHtml] allowedDomains:', props.allowedDomains);
  console.debug('[SandboxedHtml] CSP:', csp);
  const safeHtml = sanitizeHtml(props.html);
  // 注意：闭合标签使用字符串拼接，避免 Vue SFC 解析器将 </body></html> 误认为模板结束标签
  const closeBody = '<' + '/body>';
  const closeHtml = '<' + '/html>';
  const trustedScript = `<script nonce="${nonce}">\n(function(){\n  document.addEventListener('click', function(e) {\n    var a = e.target.closest('a') || e.target.closest('area');\n    if (a && a.href) {\n      e.preventDefault();\n      e.stopPropagation();\n      if (window.parent && window.parent !== window) {\n        window.parent.postMessage({ type: 'aeromail:link-click', url: a.href }, '*');\n      }\n    }\n  }, true);\n})();\n<` + '/script>';
  return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="${csp}">
  <style>
    :root { color-scheme: light; }
    body { margin: 0; padding: 0; }
    img { max-width: 100%; height: auto; }
    .email-wrapper { margin: 0 auto; max-width: 100%; box-sizing: border-box; }
  </style>
</head>
<body><div class="email-wrapper">${safeHtml}</div>
${trustedScript}
${closeBody}
${closeHtml}`;
});

let observer: MutationObserver | null = null;
let detectBlockedTimer: ReturnType<typeof setTimeout> | null = null;
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
  if (detectBlockedTimer) {
    clearTimeout(detectBlockedTimer);
    detectBlockedTimer = null;
  }
}

/** Debounced detectBlockedImages — called from MutationObserver & image load/error handlers. */
function scheduleDetectBlocked() {
  if (detectBlockedTimer) clearTimeout(detectBlockedTimer);
  detectBlockedTimer = setTimeout(detectBlockedImages, 300);
}

function detachViolationListener() {
  clearAttachInterval();
  detachSelectionListener();
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
    const url = new URL(normalized);
    if (url.protocol !== 'http:' && url.protocol !== 'https:') return;
    emit('security-violation', { domain: url.hostname.toLowerCase(), blockedUri: uri });
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
      const url = new URL(normalized);
      // 只统计真正的远程 http/https 资源；data: / blob: / cid: 等不应显示在安全横幅中
      if (url.protocol === 'http:' || url.protocol === 'https:') {
        domains.add(url.hostname.toLowerCase());
      }
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

/**
 * 检测因 CSP 被阻止而未能加载的图片，并 emit security-violation。
 * 部分浏览器的 securitypolicyviolation 事件在 srcdoc iframe 中触发不可靠，
 * 通过检查 img.complete && naturalWidth === 0 可以补获被阻止的资源。
 */
function detectBlockedImages() {
  if (!iframeRef.value?.contentDocument) return;
  const doc = iframeRef.value.contentDocument;
  const images = doc.querySelectorAll('img');

  images.forEach((img) => {
    const src =
      img.getAttribute('src') ||
      img.getAttribute('data-src') ||
      img.getAttribute('data-original');
    if (!src) return;

    const isBlocked = img.complete && img.naturalWidth === 0 && img.naturalHeight === 0;
    if (!isBlocked) return;

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
  });
}

function emitRemoteDomains() {
  const domains = extractRemoteDomainsFromDom();
  if (domains.length > 0) {
    emit('remote-domains', domains);
  }
}

function applyHeight() {
  if (!iframeRef.value?.contentDocument?.body) return;
  const height = iframeRef.value.contentDocument.body.scrollHeight;
  iframeRef.value.style.height = `${Math.min(height + 20, 2000)}px`;
  scheduleDetectBlocked();
}

/**
 * 给 iframe 内尚未加载完成的图片绑定 load/error 事件，
 * 在图片加载完成后重新计算高度并触发被阻止图片检测。
 * MutationObserver 无法捕获图片加载引起的布局变化（DOM 结构未变），
 * 必须通过图片自身的事件来弥补。
 */
function watchImages() {
  if (!iframeRef.value?.contentDocument) return;
  const images = iframeRef.value.contentDocument.querySelectorAll('img');
  images.forEach((img) => {
    if (img.complete) return;
    img.addEventListener(
      'load',
      () => {
        applyHeight();
      },
      { once: true },
    );
    img.addEventListener(
      'error',
      () => {
        applyHeight();
      },
      { once: true },
    );
  });
}

function adjustHeight() {
  emit('load');
  disconnectObserver();
  ensureViolationListener();
  ensureSelectionListener();
  emitRemoteDomains();
  // 延迟检测被 CSP 阻止的图片，给浏览器留出加载/失败判定时间
  setTimeout(detectBlockedImages, 500);
  if (!iframeRef.value?.contentDocument?.body) return;

  const body = iframeRef.value.contentDocument.body;

  observer = new MutationObserver(applyHeight);
  observer.observe(body, {
    childList: true,
    subtree: true,
    attributes: true,
  });

  applyHeight();
  watchImages();
}

watch(srcdoc, () => {
  // srcdoc changes trigger a re-load; height will be re-applied via @load.
  disconnectObserver();
  detachViolationListener();
  ensureViolationListener();
  ensureSelectionListener();
});

function handleWindowMessage(event: MessageEvent) {
  if (event.data?.type === 'aeromail:link-click' && event.data.url) {
    emit('link-click', event.data.url);
  }
}

onMounted(() => {
  ensureViolationListener();
  adjustHeight();
  window.addEventListener('message', handleWindowMessage);
});

onUnmounted(() => {
  disconnectObserver();
  detachViolationListener();
  window.removeEventListener('message', handleWindowMessage);
});

defineExpose({ iframeRef });
</script>

<template>
  <iframe
    ref="iframeRef"
    :class="className"
    :srcdoc="srcdoc"
    sandbox="allow-same-origin allow-scripts"
    style="width: 100%; border: none; min-height: 100px"
    title="Email content"
    @load="adjustHeight"
  />
</template>
