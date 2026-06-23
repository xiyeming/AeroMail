<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue';

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

const srcdoc = computed(() => {
  const csp = buildCsp();
  return `<!DOCTYPE html>
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
      max-width: 100%;
    }
    /* Respect email-defined table cell styling; don't force borders. */
    th, td {
      border: initial;
      padding: initial;
      text-align: initial;
      background: initial;
    }
    /* Prevent email content from breaking layout while preserving flow. */
    body > * {
      max-width: 100%;
    }
  </style>
</head>
<body>${props.html}</body>
</html>`;
});

let observer: MutationObserver | null = null;

function disconnectObserver() {
  if (observer) {
    observer.disconnect();
    observer = null;
  }
}

function adjustHeight() {
  disconnectObserver();
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
});

onMounted(adjustHeight);
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
