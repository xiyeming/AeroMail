import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Link from '@tiptap/extension-link';
import Image from '@tiptap/extension-image';
import Placeholder from '@tiptap/extension-placeholder';
import Underline from '@tiptap/extension-underline';
import { computed, type Ref } from 'vue';

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
      StarterKit,
      Underline,
      Link.configure({ openOnClick: false }),
      Image.configure({ allowBase64: true }),
      Placeholder.configure({ placeholder: options.placeholder ?? '' }),
    ],
    onUpdate: ({ editor }) => {
      options.onUpdate?.(editor.getHTML(), editor.getText());
    },
    editorProps: {
      handlePaste: (_view, event) => {
        const items = event.clipboardData?.items;
        if (!items) return false;
        for (const item of Array.from(items)) {
          if (item.type.startsWith('image/')) {
            const file = item.getAsFile();
            if (file) {
              options.onImagePasted?.(file);
            }
          }
        }
        return false;
      },
      handleDrop: (_view, event) => {
        const files = event.dataTransfer?.files;
        if (!files) return false;
        for (const file of Array.from(files)) {
          if (file.type.startsWith('image/')) {
            options.onImagePasted?.(file);
          }
        }
        return false;
      },
    },
  });

  const isActive = (name: string, attrs?: Record<string, unknown>) => {
    return editor.value?.isActive(name, attrs) ?? false;
  };

  const toolbarActions = computed(() => ({
    bold: () => editor.value?.chain().focus().toggleBold().run(),
    italic: () => editor.value?.chain().focus().toggleItalic().run(),
    underline: () => editor.value?.chain().focus().toggleUnderline().run(),
    strike: () => editor.value?.chain().focus().toggleStrike().run(),
    h1: () => editor.value?.chain().focus().toggleHeading({ level: 1 }).run(),
    h2: () => editor.value?.chain().focus().toggleHeading({ level: 2 }).run(),
    paragraph: () => editor.value?.chain().focus().setParagraph().run(),
    bulletList: () => editor.value?.chain().focus().toggleBulletList().run(),
    orderedList: () => editor.value?.chain().focus().toggleOrderedList().run(),
    blockquote: () => editor.value?.chain().focus().toggleBlockquote().run(),
    link: (url: string) => editor.value?.chain().focus().setLink({ href: url }).run(),
    unsetLink: () => editor.value?.chain().focus().unsetLink().run(),
    undo: () => editor.value?.chain().focus().undo().run(),
    redo: () => editor.value?.chain().focus().redo().run(),
  }));

  return { editor, EditorContent, isActive, toolbarActions };
}
