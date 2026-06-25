export function escapeHtml(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function sanitizeHref(url: string): string {
  if (/^(https?|mailto):/i.test(url)) {
    return url;
  }
  return '#';
}

export function renderMarkdown(content: string): string {
  let html = escapeHtml(content);

  // Fenced code blocks
  html = html.replace(
    /```([\s\S]*?)```/g,
    (_, code: string) =>
      `<pre class="rounded-md bg-base px-2.5 py-2 text-xs overflow-x-auto"><code>${code.trim()}</code></pre>`
  );

  // Inline code
  html = html.replace(/`([^`]+)`/g, '<code class="rounded bg-base px-1 py-0.5 text-xs">$1</code>');

  // Bold
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  // Italic
  html = html.replace(/(?<!\*)\*(?!\*)([^*]+)(?<!\*)\*(?!\*)/g, '<em>$1</em>');

  // Links
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, text: string, url: string) => {
    const safe = sanitizeHref(url);
    return `<a href="${safe}" class="underline hover:opacity-80" target="_blank" rel="noopener noreferrer">${text}</a>`;
  });

  // Blockquotes
  html = html.replace(
    /^&gt;\s*(.+)$/gm,
    '<blockquote class="border-l-2 border-border pl-2 text-secondary">$1</blockquote>'
  );

  // Lists and paragraphs
  const lines = html.split('\n');
  const out: string[] = [];
  let inList: 'ul' | 'ol' | null = null;

  for (const line of lines) {
    const ulMatch = line.match(/^(\s*)[-*]\s+(.+)$/);
    const olMatch = line.match(/^(\s*)\d+\.\s+(.+)$/);

    if (ulMatch) {
      if (inList !== 'ul') {
        if (inList === 'ol') out.push('</ol>');
        out.push('<ul class="list-disc pl-4 space-y-0.5">');
        inList = 'ul';
      }
      out.push(`<li>${ulMatch[2]}</li>`);
    } else if (olMatch) {
      if (inList !== 'ol') {
        if (inList === 'ul') out.push('</ul>');
        out.push('<ol class="list-decimal pl-4 space-y-0.5">');
        inList = 'ol';
      }
      out.push(`<li>${olMatch[2]}</li>`);
    } else {
      if (inList) {
        out.push(inList === 'ul' ? '</ul>' : '</ol>');
        inList = null;
      }
      if (line.trim() === '') {
        out.push('');
      } else {
        out.push(`<p class="min-h-[1em]">${line}</p>`);
      }
    }
  }
  if (inList) {
    out.push(inList === 'ul' ? '</ul>' : '</ol>');
  }

  return out
    .join('\n')
    .replace(
      /<p class="min-h-\[1em\]"><\/p>(\n<p class="min-h-\[1em\]"><\/p>)+/g,
      '<p class="min-h-[1em]"></p>'
    );
}

export function plainToHtml(text: string): string {
  return text
    .split('\n')
    .map((line) => `<p class="min-h-[1em]">${escapeHtml(line) || '<br />'}</p>`)
    .join('\n');
}

export function htmlToPlain(html: string): string {
  // Replace block tags with newlines and strip remaining tags.
  let text = html
    .replace(/<\/(p|div|h[1-6]|li|blockquote|pre)>/gi, '\n')
    .replace(/<br\s*\/?>/gi, '\n');

  // Strip remaining tags
  text = text.replace(/<[^>]+>/g, '');

  // Decode common entities
  const textarea = typeof document !== 'undefined' ? document.createElement('textarea') : null;
  if (textarea) {
    textarea.innerHTML = text;
    text = textarea.value;
  } else {
    text = text
      .replace(/&amp;/g, '&')
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&quot;/g, '"')
      .replace(/&#039;/g, "'")
      .replace(/&nbsp;/g, ' ');
  }

  return text
    .split('\n')
    .map((line) => line.trim())
    .join('\n')
    .trim();
}

/**
 * Converts a subset of HTML into Markdown.
 * Supports: headings, bold/italic, links, inline code, fenced code blocks,
 * unordered/ordered lists, blockquotes, paragraphs and line breaks.
 */
export function htmlToMarkdown(html: string): string {
  if (typeof document === 'undefined') {
    return htmlToPlain(html);
  }
  const wrapper = document.createElement('div');
  wrapper.innerHTML = html;
  return convertNode(wrapper).trim();
}

function convertNode(node: Node): string {
  if (node.nodeType === Node.TEXT_NODE) {
    return node.textContent ?? '';
  }
  if (node.nodeType !== Node.ELEMENT_NODE) {
    return '';
  }

  const element = node as HTMLElement;
  const tag = element.tagName.toLowerCase();
  const children = Array.from(element.childNodes).map(convertNode).join('');

  switch (tag) {
    case 'h1':
      return `# ${children.trim()}\n\n`;
    case 'h2':
      return `## ${children.trim()}\n\n`;
    case 'h3':
      return `### ${children.trim()}\n\n`;
    case 'h4':
    case 'h5':
    case 'h6':
      return `#### ${children.trim()}\n\n`;
    case 'strong':
    case 'b':
      return `**${children}**`;
    case 'em':
    case 'i':
      return `*${children}*`;
    case 'a': {
      const href = element.getAttribute('href') ?? '';
      return `[${children}](${href})`;
    }
    case 'code':
      return `\`${children}\``;
    case 'pre': {
      const code = element.querySelector('code');
      const codeText = code ? (code.textContent ?? children) : children;
      return `\`\`\`\n${codeText.trim()}\n\`\`\`\n\n`;
    }
    case 'blockquote':
      return `> ${children.trim().replace(/\n/g, '\n> ')}\n\n`;
    case 'p':
      return `${children.trim()}\n\n`;
    case 'br':
      return '\n';
    case 'ul':
      return (
        Array.from(element.children)
          .map((li) => `- ${convertNode(li).trim()}`)
          .join('\n') + '\n\n'
      );
    case 'ol':
      return (
        Array.from(element.children)
          .map((li, index) => `${index + 1}. ${convertNode(li).trim()}`)
          .join('\n') + '\n\n'
      );
    case 'li':
      return children;
    case 'div':
      return `${children}\n\n`;
    default:
      return children;
  }
}
