import { Table } from '@tiptap/extension-table';
import type { Node } from '@tiptap/core';

/**
 * 扩展 Tiptap Table，添加 align 属性控制表格水平位置。
 * renderWrapper 设为 true，确保 HTML 输出包含 div.tableWrapper。
 */
export const AlignedTable = Table.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      align: {
        default: 'left',
        parseHTML: (element: HTMLElement) => {
          // 从 wrapper div 或 table 元素读取 align
          const wrapper = element.closest('.tableWrapper') || element;
          const style = wrapper.getAttribute('style') || '';
          if (style.includes('margin-left:auto') && style.includes('margin-right:auto')) return 'center';
          if (style.includes('margin-left:auto')) return 'right';
          return 'left';
        },
        renderHTML: (attrs: Record<string, unknown>) => {
          const align = attrs.align as string;
          if (!align || align === 'left') return {};
          const marginMap: Record<string, string> = {
            center: 'margin:0.5em auto;',
            right: 'margin:0.5em 0;margin-left:auto;display:block;',
          };
          return { style: marginMap[align] || '' };
        },
      },
    };
  },

  renderHTML({ node, HTMLAttributes }: { node: Node; HTMLAttributes: Record<string, unknown> }) {
    // 调用父类的 renderHTML 获取 table 结构
    const parentResult = (Table.prototype as any).renderHTML.call(this, { node, HTMLAttributes });
    // parentResult 可能是 ["table", attrs, ...] 或 ["div", wrapperAttrs, ["table", ...]]
    // 由于 renderWrapper=false（默认），它返回 ["table", attrs, colgroup, ["tbody", 0]]
    // 我们用 align 属性包裹一层 wrapper
    const align = (node.attrs.align as string) || 'left';
    const wrapperStyles: Record<string, string> = {
      left: 'overflow-x:auto;margin:0.5em 0;',
      center: 'overflow-x:auto;margin:0.5em auto;display:block;',
      right: 'overflow-x:auto;margin:0.5em 0;margin-left:auto;display:block;',
    };
    return ['div', { class: 'tableWrapper', style: wrapperStyles[align] || wrapperStyles.left }, parentResult];
  },

  addCommands() {
    return {
      ...this.parent?.(),
      setTableAlign:
        (align: 'left' | 'center' | 'right') =>
        ({ commands }: { commands: any }) => {
          return commands.updateAttributes('table', { align });
        },
    };
  },
});
