import { Table } from '@tiptap/extension-table';
import type { Node } from '@tiptap/core';
import type { NodeView, ViewMutationRecord } from 'prosemirror-view';

/**
 * 自定义 TableView，在 wrapper div 上应用 align 对齐样式。
 * Tiptap 内置 TableView 的 update() 只更新列宽不更新 wrapper，
 * 所以我们包装一层，在每次 update 时同步 align → margin。
 */
function getAlignStyle(align: string): string {
  switch (align) {
    case 'center':
      return 'margin:0.5em auto;display:block;overflow-x:auto;';
    case 'right':
      return 'margin:0.5em 0;margin-left:auto;display:block;overflow-x:auto;';
    default:
      return 'overflow-x:auto;margin:0.5em 0;';
  }
}

class AlignedTableView implements NodeView {
  dom: HTMLDivElement;
  table: HTMLTableElement;
  colgroup: HTMLTableColElement;
  contentDOM: HTMLTableSectionElement;
  private node: Node;
  private cellMinWidth: number;

  constructor(node: Node, cellMinWidth: number, _view: unknown, HTMLAttributes: Record<string, unknown> = {}) {
    this.node = node;
    this.cellMinWidth = cellMinWidth;

    // 创建 wrapper div
    this.dom = document.createElement('div');
    this.dom.className = 'tableWrapper';
    this.applyAlignStyle(node.attrs.align);

    // 创建 table
    this.table = this.dom.appendChild(document.createElement('table'));
    this.table.style.borderCollapse = 'collapse';
    this.table.style.tableLayout = 'fixed';
    this.table.style.width = 'fit-content';
    this.table.style.minWidth = '200px';

    for (const [key, value] of Object.entries(HTMLAttributes)) {
      if (value !== undefined && value !== null && key !== 'style') {
        this.table.setAttribute(key, String(value));
      }
    }

    // 创建 colgroup 和 tbody
    this.colgroup = this.table.appendChild(document.createElement('colgroup'));
    this.updateColumns(node);
    this.contentDOM = this.table.appendChild(document.createElement('tbody'));
  }

  private applyAlignStyle(align: string) {
    this.dom.setAttribute('style', getAlignStyle(align || 'left'));
  }

  private updateColumns(node: Node) {
    const cellMinWidth = this.cellMinWidth;
    let totalWidth = 0;
    let nextDOM = this.colgroup.firstChild as HTMLTableColElement | null;
    const row = node.firstChild;

    if (row !== null) {
      for (let i = 0, col = 0; i < row.childCount; i += 1) {
        const { colspan, colwidth } = row.child(i).attrs;
        for (let j = 0; j < colspan; j += 1, col += 1) {
          const hasWidth = colwidth && colwidth[j];
          const cssWidth = hasWidth ? `${hasWidth}px` : '';
          totalWidth += hasWidth || cellMinWidth;

          if (!nextDOM) {
            const colEl = document.createElement('col');
            colEl.style.width = cssWidth || `${cellMinWidth}px`;
            this.colgroup.appendChild(colEl);
          } else {
            if (nextDOM.style.width !== cssWidth) {
              nextDOM.style.width = cssWidth || `${cellMinWidth}px`;
            }
            nextDOM = nextDOM.nextSibling as HTMLTableColElement | null;
          }
        }
      }
    }
  }

  update(node: Node): boolean {
    if (node.type !== this.node.type) return false;
    this.node = node;
    this.updateColumns(node);
    // 关键：同步 align 样式到 wrapper
    this.applyAlignStyle(node.attrs.align);
    return true;
  }

  ignoreMutation(mutation: ViewMutationRecord): boolean {
    return (mutation.target === this.dom && mutation.type === 'attributes') || false;
  }

  destroy() {
    // cleanup
  }
}

/**
 * 扩展 Tiptap Table，添加 align 属性控制表格水平位置。
 * 使用自定义 AlignedTableView 确保编辑器中实时显示对齐效果。
 */
export const AlignedTable = Table.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      align: {
        default: 'left',
        parseHTML: (element: HTMLElement) => {
          const wrapper = element.closest('.tableWrapper') || element;
          const style = wrapper.getAttribute('style') || '';
          if (style.includes('margin-left:auto') && style.includes('margin-right:auto')) return 'center';
          if (style.includes('margin-left:auto')) return 'right';
          return 'left';
        },
        renderHTML: (attrs: Record<string, unknown>) => {
          return {};
        },
      },
    };
  },

  addOptions() {
    return {
      ...this.parent?.(),
      View: AlignedTableView,
    };
  },

  renderHTML({ node, HTMLAttributes }: { node: Node; HTMLAttributes: Record<string, unknown> }) {
    const parentResult = (Table.prototype as any).renderHTML.call(this, { node, HTMLAttributes });
    const align = (node.attrs.align as string) || 'left';
    return ['div', { class: 'tableWrapper', style: getAlignStyle(align) }, parentResult];
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
