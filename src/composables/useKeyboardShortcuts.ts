import { onMounted, onUnmounted } from 'vue';
import { useMailStore } from '@/stores/mail';

export function useKeyboardShortcuts() {
  const mailStore = useMailStore();

  function handleKeydown(e: KeyboardEvent) {
    // 忽略输入框内的按键
    if (
      e.target instanceof HTMLInputElement ||
      e.target instanceof HTMLTextAreaElement ||
      (e.target as HTMLElement).isContentEditable
    ) {
      return;
    }

    // Ctrl/Cmd 组合键
    if (e.ctrlKey || e.metaKey) {
      switch (e.key.toLowerCase()) {
        case 'r':
          if (e.shiftKey) {
            e.preventDefault();
            mailStore.toggleReadingMode();
          }
          break;
        case 'a':
          e.preventDefault();
          mailStore.selectAll();
          break;
      }
      return;
    }

    // 单个按键
    switch (e.key) {
      case 'j':
      case 'ArrowDown':
        e.preventDefault();
        mailStore.selectNextMail();
        break;
      case 'k':
      case 'ArrowUp':
        e.preventDefault();
        mailStore.selectPreviousMail();
        break;
      case 'Enter':
        if (mailStore.selectedMailId) {
          e.preventDefault();
          mailStore.loadMailDetail(mailStore.selectedMailId);
        }
        break;
      case 'Escape':
        e.preventDefault();
        if (mailStore.isReadingMode) {
          mailStore.toggleReadingMode();
        } else if (mailStore.selectedMailIds.length > 0) {
          mailStore.clearBulkSelection();
        } else if (mailStore.selectedMailId) {
          mailStore.clearSelection();
        }
        break;
      case 's':
        if (mailStore.selectedMailId) {
          e.preventDefault();
          mailStore.toggleStar(mailStore.selectedMailId);
        }
        break;
      case 'u':
        e.preventDefault();
        mailStore.selectNextUnread();
        break;
      case 'e':
        if (mailStore.selectedMailId) {
          e.preventDefault();
          const mail = mailStore.mails.find((m) => m.id === mailStore.selectedMailId);
          if (mail) {
            mailStore.markRead(mailStore.selectedMailId, !mail.isRead);
          }
        }
        break;
      case 'Delete':
      case 'Backspace':
        if (mailStore.selectedMailId && !e.shiftKey) {
          e.preventDefault();
          mailStore.deleteMail(mailStore.selectedMailId);
        }
        break;
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
}
