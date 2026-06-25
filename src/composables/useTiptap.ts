import { computed, type Ref } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Underline from '@tiptap/extension-underline';
import { TextStyle } from '@tiptap/extension-text-style';
import FontFamily from '@tiptap/extension-font-family';
import Color from '@tiptap/extension-color';
import Highlight from '@tiptap/extension-highlight';
import TextAlign from '@tiptap/extension-text-align';
import Subscript from '@tiptap/extension-subscript';
import Superscript from '@tiptap/extension-superscript';
import Link from '@tiptap/extension-link';
import Image from '@tiptap/extension-image';
import { Table } from '@tiptap/extension-table';
import TableRow from '@tiptap/extension-table-row';
import TableHeader from '@tiptap/extension-table-header';
import TableCell from '@tiptap/extension-table-cell';
import Placeholder from '@tiptap/extension-placeholder';
import { FontSize } from './useFontSizeExtension';
import { Indent } from './useIndentExtension';

export interface UseTiptapOptions {
  content: Ref<string>;
  placeholder?: string;
  onUpdate?: (html: string, text: string) => void;
  onImagePasted?: (file: File) => void;
}

export function useTiptap(options: UseTiptapOptions) {
  const editor = useEditor({
    content: options.content.value,
    extensions: [
      StarterKit.configure({
        heading: { levels: [1, 2, 3] },
      }),
      Underline,
      TextStyle,
      FontFamily,
      FontSize,
      Color,
      Highlight.configure({ multicolor: true }),
      TextAlign.configure({ types: ['heading', 'paragraph'] }),
      Subscript,
      Superscript,
      Link.configure({ openOnClick: false }),
      Image.configure({ allowBase64: true }),
      Table.configure({
        resizable: true,
        HTMLAttributes: {
          class: 'table-wrapper',
        },
      }),
      TableRow,
      TableHeader,
      TableCell,
      Placeholder.configure({
        placeholder: options.placeholder ?? 'Write something...',
      }),
      Indent,
    ],
    editorProps: {
      handlePaste: (view, event) => {
        const items = event.clipboardData?.items;
        if (!items) return false;
        for (const item of Array.from(items)) {
          if (item.type.startsWith('image/')) {
            const file = item.getAsFile();
            if (file && options.onImagePasted) {
              // 阻止浏览器默认粘贴行为，由自定义图片处理接管
              event.preventDefault();
              options.onImagePasted(file);
              return true;
            }
          }
        }
        return false;
      },
      handleDrop: (view, event) => {
        const files = event.dataTransfer?.files;
        if (!files) return false;
        for (const file of Array.from(files)) {
          if (file.type.startsWith('image/') && options.onImagePasted) {
            // 阻止浏览器默认拖拽行为，由自定义图片处理接管
            event.preventDefault();
            options.onImagePasted(file);
            return true;
          }
        }
        return false;
      },
    },
    onUpdate: ({ editor: ed }) => {
      const html = ed.getHTML();
      const text = ed.getText();
      options.onUpdate?.(html, text);
    },
  });

  function isActive(name: string, attrs?: Record<string, unknown>) {
    return editor.value?.isActive(name, attrs) ?? false;
  }

  function getAttributes(name: string) {
    return (editor.value?.getAttributes(name) as Record<string, unknown>) ?? {};
  }

  function getSelection() {
    if (!editor.value) return { from: 0, to: 0, empty: true };
    const { from, to, empty } = editor.value.state.selection;
    return { from, to, empty };
  }

  function getSelectedText() {
    if (!editor.value) return '';
    const { from, to } = editor.value.state.selection;
    return editor.value.state.doc.textBetween(from, to, ' ');
  }

  function setContent(html: string) {
    editor.value?.commands.setContent(html, { emitUpdate: false });
  }

  function insertContent(html: string) {
    editor.value?.chain().focus().insertContent(html).run();
  }

  const toolbarActions = computed(() => {
    if (!editor.value) return {};
    const ed = editor.value;
    return {
      bold: () => ed.chain().focus().toggleBold().run(),
      italic: () => ed.chain().focus().toggleItalic().run(),
      underline: () => ed.chain().focus().toggleUnderline().run(),
      strike: () => ed.chain().focus().toggleStrike().run(),
      code: () => ed.chain().focus().toggleCode().run(),
      subscript: () => ed.chain().focus().toggleSubscript().run(),
      superscript: () => ed.chain().focus().toggleSuperscript().run(),
      h1: () => ed.chain().focus().toggleHeading({ level: 1 }).run(),
      h2: () => ed.chain().focus().toggleHeading({ level: 2 }).run(),
      paragraph: () => ed.chain().focus().setParagraph().run(),
      bulletList: () => ed.chain().focus().toggleBulletList().run(),
      orderedList: () => ed.chain().focus().toggleOrderedList().run(),
      blockquote: () => ed.chain().focus().toggleBlockquote().run(),
      alignLeft: () => ed.chain().focus().setTextAlign('left').run(),
      alignCenter: () => ed.chain().focus().setTextAlign('center').run(),
      alignRight: () => ed.chain().focus().setTextAlign('right').run(),
      alignJustify: () => ed.chain().focus().setTextAlign('justify').run(),
      indent: () => ed.chain().focus().sinkListItem('listItem').run(),
      outdent: () => ed.chain().focus().liftListItem('listItem').run(),
      link: (url: string) => ed.chain().focus().setLink({ href: url }).run(),
      unsetLink: () => ed.chain().focus().unsetLink().run(),
      undo: () => ed.chain().focus().undo().run(),
      redo: () => ed.chain().focus().redo().run(),
      setFontFamily: (family: string) => ed.chain().focus().setFontFamily(family).run(),
      unsetFontFamily: () => ed.chain().focus().unsetFontFamily().run(),
      setFontSize: (size: string) => ed.chain().focus().setFontSize(size).run(),
      unsetFontSize: () => ed.chain().focus().unsetFontSize().run(),
      setColor: (color: string) => ed.chain().focus().setColor(color).run(),
      unsetColor: () => ed.chain().focus().unsetColor().run(),
      setHighlight: (color: string) => ed.chain().focus().setHighlight({ color }).run(),
      unsetHighlight: () => ed.chain().focus().unsetHighlight().run(),
      insertTable: () =>
        ed.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
      insertImage: (src: string) => ed.chain().focus().setImage({ src }).run(),
      insertHorizontalRule: () => ed.chain().focus().setHorizontalRule().run(),
      insertHardBreak: () => ed.chain().focus().setHardBreak().run(),
      clearNodes: () => ed.chain().focus().unsetAllMarks().clearNodes().run(),
    };
  });

  return {
    editor,
    EditorContent,
    isActive,
    getAttributes,
    getSelection,
    getSelectedText,
    setContent,
    insertContent,
    toolbarActions,
  };
}
