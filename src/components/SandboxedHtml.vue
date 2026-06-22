<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';

const props = defineProps<{
  html: string;
  className?: string;
  allowedDomains?: string[];
}>();

const iframeRef = ref<HTMLIFrameElement | null>(null);

function buildCsp(): string {
  const allowed = (props.allowedDomains || [])
    .map((d) => d.trim())
    .filter(Boolean);

  // Allow exact host and its subdomains for each trusted domain.
  const hosts = allowed.flatMap((domain) => [domain, `*.${domain}`]);

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

function renderHtml() {
  if (!iframeRef.value) return;
  const doc = iframeRef.value.contentDocument;
  if (!doc) return;

  const csp = buildCsp();

  doc.open();
  doc.write(`
    <!DOCTYPE html>
    <html>
    <head>
      <meta charset="utf-8">
      <meta http-equiv="Content-Security-Policy" content="${csp}">
      <style>
        :root {
          color-scheme: light;
        }
        body {
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
          font-size: 14px;
          line-height: 1.6;
          color: #333;
          margin: 0;
          padding: 8px;
          word-wrap: break-word;
          overflow-wrap: break-word;
          background: #fff;
        }
        img {
          max-width: 100%;
          height: auto;
        }
        a {
          color: #0066cc;
          text-decoration: none;
          pointer-events: none;
        }
        a:hover {
          text-decoration: underline;
        }
        pre, code {
          font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
          background: #f5f5f5;
          padding: 2px 4px;
          border-radius: 3px;
          font-size: 13px;
        }
        pre {
          padding: 12px;
          overflow-x: auto;
        }
        blockquote {
          border-left: 3px solid #ddd;
          margin: 0;
          padding-left: 12px;
          color: #666;
        }
        table {
          border-collapse: collapse;
          max-width: 100%;
        }
        th, td {
          border: 1px solid #ddd;
          padding: 8px;
          text-align: left;
        }
        th {
          background: #f5f5f5;
        }
        /* 防止邮件内容破坏布局 */
        body > * {
          max-width: 100%;
          overflow: hidden;
        }
      </style>
    </head>
    <body>${props.html}</body>
    </html>
  `);
  doc.close();

  // 自动调整 iframe 高度
  adjustHeight();
}

function adjustHeight() {
  if (!iframeRef.value?.contentDocument?.body) return;
  // 使用 MutationObserver 监听内容变化
  const observer = new MutationObserver(() => {
    if (!iframeRef.value?.contentDocument?.body) return;
    const height = iframeRef.value.contentDocument.body.scrollHeight;
    iframeRef.value.style.height = `${Math.min(height + 20, 2000)}px`;
  });

  observer.observe(iframeRef.value.contentDocument.body, {
    childList: true,
    subtree: true,
    attributes: true,
  });

  // 初始调整
  const height = iframeRef.value.contentDocument.body.scrollHeight;
  iframeRef.value.style.height = `${Math.min(height + 20, 2000)}px`;
}

watch(() => [props.html, props.allowedDomains], renderHtml, { deep: true });
onMounted(renderHtml);
</script>

<template>
  <iframe
    ref="iframeRef"
    :class="className"
    sandbox="allow-same-origin"
    style="width: 100%; border: none; min-height: 100px"
    title="Email content"
  />
</template>
