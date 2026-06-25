import { Extension } from '@tiptap/core';

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    indent: {
      indent: () => ReturnType;
      outdent: () => ReturnType;
    };
  }
}

const INDENT_ATTRIBUTE = 'data-indent';
const INDENT_MAX_LEVEL = 8;
const INDENT_SIZE = 2; // em per level

export const Indent = Extension.create({
  name: 'indent',

  addOptions() {
    return {
      types: ['paragraph', 'heading'],
      indentLevel: 0,
    };
  },

  addGlobalAttributes() {
    return [
      {
        types: this.options.types,
        attributes: {
          [INDENT_ATTRIBUTE]: {
            default: 0,
            parseHTML: (element) => {
              const level = parseInt(element.getAttribute(INDENT_ATTRIBUTE) || '0', 10);
              return level || 0;
            },
            renderHTML: (attributes) => {
              const level = attributes[INDENT_ATTRIBUTE];
              if (!level || level <= 0) return {};
              return {
                [INDENT_ATTRIBUTE]: level,
                style: `padding-left: ${level * INDENT_SIZE}em`,
              };
            },
          },
        },
      },
    ];
  },

  addCommands() {
    return {
      indent:
        () =>
        ({ tr, state, dispatch }) => {
          const { selection } = state;
          const pos = selection.$from;

          for (let depth = pos.depth; depth >= 0; depth--) {
            const node = pos.node(depth);
            if (this.options.types.includes(node.type.name)) {
              const currentLevel = node.attrs[INDENT_ATTRIBUTE] || 0;
              if (currentLevel < INDENT_MAX_LEVEL) {
                if (dispatch) {
                  tr.setNodeMarkup(pos.before(depth), undefined, {
                    ...node.attrs,
                    [INDENT_ATTRIBUTE]: currentLevel + 1,
                  });
                  dispatch(tr);
                }
                return true;
              }
            }
          }
          return false;
        },
      outdent:
        () =>
        ({ tr, state, dispatch }) => {
          const { selection } = state;
          const pos = selection.$from;

          for (let depth = pos.depth; depth >= 0; depth--) {
            const node = pos.node(depth);
            if (this.options.types.includes(node.type.name)) {
              const currentLevel = node.attrs[INDENT_ATTRIBUTE] || 0;
              if (currentLevel > 0) {
                if (dispatch) {
                  tr.setNodeMarkup(pos.before(depth), undefined, {
                    ...node.attrs,
                    [INDENT_ATTRIBUTE]: currentLevel - 1,
                  });
                  dispatch(tr);
                }
                return true;
              }
            }
          }
          return false;
        },
    };
  },

  addKeyboardShortcuts() {
    return {
      Tab: () => this.editor.commands.indent(),
      'Shift-Tab': () => this.editor.commands.outdent(),
    };
  },
});
