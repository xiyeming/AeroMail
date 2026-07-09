import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { listen } from '@tauri-apps/api/event';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import { useToastStore } from './toast';
import type { MailHeader, MailDetail, FolderInfo, NewMailsEvent } from '@/types/mail';

const PAGE_SIZE = 50;

const VIRTUAL_FOLDERS = ['starred', 'sent', 'archived', 'spam', 'trash'];

export type MailFilter = 'all' | 'unread' | 'starred' | 'attachments';

export const useMailStore = defineStore('mail', () => {
  const { call } = useTauriInvoke();
  const mails = ref<MailHeader[]>([]);
  const selectedMailId = ref<string | null>(null);
  const selectedMail = ref<MailDetail | null>(null);
  const selectedMailIds = ref<string[]>([]);
  const lastSelectedIndex = ref<number>(-1);
  const folders = ref<FolderInfo[]>([]);
  const currentFolderId = ref<string>('');
  const currentAccountId = ref<string>('');
  const loading = ref(false);
  const loadingMore = ref(false);
  const hasMore = ref(true);
  const error = ref<string | null>(null);
  const isReadingMode = ref(false);
  const deletingMailIds = ref<Set<string>>(new Set());
  const activeFilter = ref<MailFilter>('all');

  // 用于防止快速切换邮件时旧响应覆盖新响应
  let currentLoadId = 0;

  const totalUnread = computed(() => folders.value.reduce((sum, f) => sum + f.unreadCount, 0));

  async function loadFolders(accountId: string) {
    try {
      currentAccountId.value = accountId;
      const newFolders = await call<FolderInfo[]>('list_folders', { accountId });
      const existing = new Map(folders.value.map((f) => [f.id, f]));
      for (const folder of newFolders) {
        existing.set(folder.id, folder);
      }
      folders.value = Array.from(existing.values());
      if (folders.value.length > 0 && !currentFolderId.value) {
        currentFolderId.value = folders.value[0].id;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function loadInboxMails(accountIds: string[], reset = true) {
    if (loading.value || accountIds.length === 0) return;

    try {
      if (reset) {
        loading.value = true;
        hasMore.value = true;
      } else {
        loadingMore.value = true;
      }

      const offset = reset ? 0 : mails.value.length;
      const newMails = await call<MailHeader[]>('get_inbox_mail_list', {
        accountIds,
        limit: PAGE_SIZE,
        offset,
      });

      // 先获取数据再替换，避免先清空导致列表闪烁
      if (reset) {
        mails.value = newMails;
      } else {
        mails.value.push(...newMails);
      }

      hasMore.value = newMails.length === PAGE_SIZE;
      error.value = null;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
      loadingMore.value = false;
    }
  }

  async function loadMails(folderId: string, reset = true) {
    if (loading.value) return;

    try {
      if (reset) {
        loading.value = true;
        hasMore.value = true;
      } else {
        loadingMore.value = true;
      }

      currentFolderId.value = folderId;
      const isVirtual = VIRTUAL_FOLDERS.includes(folderId);
      const offset = reset ? 0 : mails.value.length;
      let newMails = await call<MailHeader[]>(
        isVirtual ? 'get_virtual_mail_list' : 'get_mail_list',
        {
          folderId,
          limit: PAGE_SIZE,
          offset,
        }
      );

      // 滚动加载时，若本地无更多数据则从 IMAP 服务器拉取旧邮件
      // 仅对真实文件夹执行；虚拟文件夹无远程旧邮件源
      // 初始加载(reset=true)不触发 IMAP，避免启动时阻塞
      if (!reset && newMails.length === 0 && hasMore.value && !isVirtual) {
        await call<number>('fetch_older_mails', {
          folderId,
          limit: PAGE_SIZE,
        });
        // 重新查询本地数据库
        newMails = await call<MailHeader[]>('get_mail_list', {
          folderId,
          limit: PAGE_SIZE,
          offset,
        });
      }

      // 先获取数据再替换，避免先清空导致列表闪烁
      if (reset) {
        mails.value = newMails;
      } else {
        mails.value.push(...newMails);
      }

      hasMore.value = newMails.length === PAGE_SIZE;
      error.value = null;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
      loadingMore.value = false;
    }
  }

  async function refreshMails(folderId: string) {
    if (loading.value || loadingMore.value) return;

    try {
      currentFolderId.value = folderId;
      const isVirtual = VIRTUAL_FOLDERS.includes(folderId);
      // Fetch enough items to cover what the user has already loaded, then
      // merge with the existing list. Reusing the same object references for
      // already-visible items lets Vue's keyed list keep the DOM nodes in
      // place, which removes the post-sync flicker.
      const limit = Math.max(PAGE_SIZE, mails.value.length);
      const fetched = await call<MailHeader[]>(
        isVirtual ? 'get_virtual_mail_list' : 'get_mail_list',
        {
          folderId,
          limit,
          offset: 0,
        }
      );

      if (fetched.length > 0) {
        const existing = new Map(mails.value.map((m) => [m.id, m]));
        const merged: MailHeader[] = [];
        for (const mail of fetched) {
          const current = existing.get(mail.id);
          if (current) {
            Object.assign(current, mail);
            merged.push(current);
          } else {
            merged.push(mail);
          }
        }
        mails.value = merged;
      }

      hasMore.value = fetched.length === limit;
      error.value = null;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function refreshInboxMails(accountIds: string[]) {
    if (accountIds.length === 0 || loading.value || loadingMore.value) return;

    try {
      const limit = Math.max(PAGE_SIZE, mails.value.length);
      const fetched = await call<MailHeader[]>('get_inbox_mail_list', {
        accountIds,
        limit,
        offset: 0,
      });

      if (fetched.length > 0 || mails.value.length > 0) {
        const existing = new Map(mails.value.map((m) => [m.id, m]));
        const merged: MailHeader[] = [];
        for (const mail of fetched) {
          const current = existing.get(mail.id);
          if (current) {
            Object.assign(current, mail);
            merged.push(current);
          } else {
            merged.push(mail);
          }
        }
        mails.value = merged;
      }

      hasMore.value = fetched.length === limit;
      error.value = null;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  function insertMails(newMails: MailHeader[]) {
    if (newMails.length === 0) return;
    const existingIds = new Set(mails.value.map((m) => m.id));
    const unique = newMails.filter((m) => !existingIds.has(m.id));
    if (unique.length === 0) return;

    const seen = new Set<string>();
    const deduped = unique.filter((m) => {
      if (seen.has(m.id)) return false;
      seen.add(m.id);
      return true;
    });

    mails.value = [...deduped, ...mails.value];
    error.value = null;
  }

  async function initEventListeners() {
    const unlisten = await listen<NewMailsEvent>('sync:new_mails', (event) => {
      const { folderId, mails: newMails } = event.payload;
      if (folderId === currentFolderId.value && !loading.value) {
        insertMails(newMails);
      }
    });
    return unlisten;
  }

  async function loadMailDetail(mailId: string) {
    // 递增加载序号，用于丢弃过期的异步响应
    const loadId = ++currentLoadId;

    // 先设置 selectedMailId 并清空详情，使 UI 立即显示加载状态
    selectedMailId.value = mailId;
    selectedMail.value = null;
    try {
      const detail = await call<MailDetail>('get_mail_detail', { mailId });
      // 如果已有更新的加载请求，丢弃本次结果
      if (loadId !== currentLoadId) return;
      selectedMail.value = detail;
      error.value = null;
    } catch (e) {
      if (loadId !== currentLoadId) return;
      error.value = e instanceof Error ? e.message : String(e);
      // 加载失败时保留 selectedMailId，便于用户看到错误横幅；仅清空详情
      selectedMail.value = null;
      return;
    }

    // 标记已读：失败不应影响详情显示，仅记录错误
    if (selectedMail.value && !selectedMail.value.isRead) {
      try {
        await markRead(mailId, true);
      } catch (e) {
        console.error('Failed to mark mail as read:', e);
        error.value = e instanceof Error ? e.message : String(e);
      }
    }
  }

  async function selectMail(mailId: string) {
    await loadMailDetail(mailId);
  }

  async function markRead(mailId: string, isRead: boolean) {
    try {
      await call('mark_mail_read', { mailId, isRead });
      // Update local state immediately
      const mail = mails.value.find((m) => m.id === mailId);
      if (mail && mail.isRead !== isRead) {
        mail.isRead = isRead;
        // Update folder unread count
        const folder = folders.value.find((f) => f.id === mail.folderId);
        if (folder) {
          folder.unreadCount = Math.max(0, folder.unreadCount + (isRead ? -1 : 1));
        }
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      useToastStore().add({ type: 'error', message: msg });
    }
  }

  async function toggleStar(mailId: string) {
    try {
      const newStarred = await call<boolean>('toggle_mail_star', { mailId });
      const mail = mails.value.find((m) => m.id === mailId);
      if (mail) {
        mail.isStarred = newStarred;
      }
      if (selectedMail.value?.id === mailId) {
        selectedMail.value.isStarred = newStarred;
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      useToastStore().add({ type: 'error', message: msg });
    }
  }

  async function archiveMail(mailId: string) {
    try {
      await call('archive_mail', { mailId });
      const mail = mails.value.find((m) => m.id === mailId);
      if (mail) {
        mail.isArchived = true;
      }
      if (selectedMail.value?.id === mailId) {
        selectedMail.value.isArchived = true;
      }
      if (currentFolderId.value !== 'archived') {
        const index = mails.value.findIndex((m) => m.id === mailId);
        if (index !== -1) {
          mails.value.splice(index, 1);
        }
        if (selectedMailId.value === mailId) {
          clearSelection();
        }
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      useToastStore().add({ type: 'error', message: msg });
    }
  }

  async function unarchiveMail(mailId: string) {
    try {
      await call('unarchive_mail', { mailId });
      const mail = mails.value.find((m) => m.id === mailId);
      if (mail) {
        mail.isArchived = false;
      }
      if (selectedMail.value?.id === mailId) {
        selectedMail.value.isArchived = false;
      }
      if (currentFolderId.value === 'archived') {
        const index = mails.value.findIndex((m) => m.id === mailId);
        if (index !== -1) {
          mails.value.splice(index, 1);
        }
        if (selectedMailId.value === mailId) {
          clearSelection();
        }
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      useToastStore().add({ type: 'error', message: msg });
    }
  }

  async function toggleSpam(mailId: string) {
    try {
      const newSpam = await call<boolean>('toggle_mail_spam', { mailId });
      const mail = mails.value.find((m) => m.id === mailId);
      if (mail) {
        mail.isSpam = newSpam;
      }
      if (selectedMail.value?.id === mailId) {
        selectedMail.value.isSpam = newSpam;
      }
      // Remove from list if it no longer belongs in the current virtual folder.
      if (currentFolderId.value === 'spam' && !newSpam) {
        const index = mails.value.findIndex((m) => m.id === mailId);
        if (index !== -1) {
          mails.value.splice(index, 1);
        }
        if (selectedMailId.value === mailId) {
          clearSelection();
        }
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      useToastStore().add({ type: 'error', message: msg });
    }
  }

  async function deleteMail(mailId: string) {
    deletingMailIds.value.add(mailId);
    try {
      await call('delete_mail', { mailId });
      const index = mails.value.findIndex((m) => m.id === mailId);
      if (index !== -1) {
        const mail = mails.value[index];
        // Update folder unread count if the deleted mail was unread
        if (!mail.isRead) {
          const folder = folders.value.find((f) => f.id === mail.folderId);
          if (folder) {
            folder.unreadCount = Math.max(0, folder.unreadCount - 1);
          }
        }
        mails.value.splice(index, 1);
      }
      if (selectedMailId.value === mailId) {
        clearSelection();
      }
      if (currentFolderId.value === 'trash') {
        await refreshMails('trash');
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      useToastStore().add({ type: 'error', message: msg });
    } finally {
      deletingMailIds.value.delete(mailId);
    }
  }

  function toggleReadingMode() {
    isReadingMode.value = !isReadingMode.value;
  }

  function selectNextMail() {
    if (mails.value.length === 0) return;
    const currentIndex = mails.value.findIndex((m) => m.id === selectedMailId.value);
    const nextIndex = currentIndex < mails.value.length - 1 ? currentIndex + 1 : 0;
    selectMail(mails.value[nextIndex].id);
  }

  function selectPreviousMail() {
    if (mails.value.length === 0) return;
    const currentIndex = mails.value.findIndex((m) => m.id === selectedMailId.value);
    const prevIndex = currentIndex > 0 ? currentIndex - 1 : mails.value.length - 1;
    selectMail(mails.value[prevIndex].id);
  }

  function selectNextUnread() {
    if (mails.value.length === 0) return;
    const currentIndex = mails.value.findIndex((m) => m.id === selectedMailId.value);
    const start = currentIndex >= 0 ? currentIndex + 1 : 0;
    for (let i = 0; i < mails.value.length; i++) {
      const idx = (start + i) % mails.value.length;
      if (!mails.value[idx].isRead) {
        selectMail(mails.value[idx].id);
        return;
      }
    }
  }

  function setFilter(filter: MailFilter) {
    activeFilter.value = filter;
  }

  function clearSelection() {
    selectedMailId.value = null;
    selectedMail.value = null;
  }

  function closeReader() {
    clearSelection();
    isReadingMode.value = false;
  }

  function addToSelection(mailId: string) {
    if (!selectedMailIds.value.includes(mailId)) {
      selectedMailIds.value = [...selectedMailIds.value, mailId];
    }
    lastSelectedIndex.value = mails.value.findIndex((m) => m.id === mailId);
  }

  function removeFromSelection(mailId: string) {
    selectedMailIds.value = selectedMailIds.value.filter((id) => id !== mailId);
    lastSelectedIndex.value = mails.value.findIndex((m) => m.id === mailId);
  }

  // Batch selection
  function toggleSelection(mailId: string, ctrlKey: boolean, shiftKey: boolean) {
    if (shiftKey && lastSelectedIndex.value >= 0) {
      // Range selection
      const currentIndex = mails.value.findIndex((m) => m.id === mailId);
      const start = Math.min(lastSelectedIndex.value, currentIndex);
      const end = Math.max(lastSelectedIndex.value, currentIndex);
      const next = new Set(selectedMailIds.value);
      for (let i = start; i <= end; i++) {
        next.add(mails.value[i].id);
      }
      selectedMailIds.value = Array.from(next);
    } else if (ctrlKey) {
      // Toggle single
      const index = selectedMailIds.value.indexOf(mailId);
      if (index === -1) {
        selectedMailIds.value = [...selectedMailIds.value, mailId];
      } else {
        selectedMailIds.value = selectedMailIds.value.filter((id) => id !== mailId);
      }
    } else {
      // Single select
      selectedMailIds.value = [mailId];
    }
    lastSelectedIndex.value = mails.value.findIndex((m) => m.id === mailId);
  }

  function selectAll() {
    selectedMailIds.value = mails.value.map((m) => m.id);
  }

  function clearBulkSelection() {
    selectedMailIds.value = [];
    lastSelectedIndex.value = -1;
  }

  function invertSelection() {
    const current = new Set(selectedMailIds.value);
    selectedMailIds.value = mails.value.filter((m) => !current.has(m.id)).map((m) => m.id);
    lastSelectedIndex.value = -1;
  }

  // Batch actions
  async function bulkMarkRead(isRead: boolean) {
    for (const mailId of selectedMailIds.value) {
      await markRead(mailId, isRead);
    }
  }

  async function bulkDelete() {
    const ids = [...selectedMailIds.value];
    try {
      for (const mailId of ids) {
        await deleteMail(mailId);
      }
    } finally {
      clearBulkSelection();
    }
  }

  async function moveMail(mailId: string, targetFolderId: string) {
    try {
      await call('move_mail', { mailId, targetFolderId });
      const index = mails.value.findIndex((m) => m.id === mailId);
      if (index !== -1) {
        const mail = mails.value[index];
        // Update folder unread counts if the moved mail was unread
        if (!mail.isRead) {
          const sourceFolder = folders.value.find((f) => f.id === mail.folderId);
          if (sourceFolder) {
            sourceFolder.unreadCount = Math.max(0, sourceFolder.unreadCount - 1);
          }
          const targetFolder = folders.value.find((f) => f.id === targetFolderId);
          if (targetFolder) {
            targetFolder.unreadCount += 1;
          }
        }
        mails.value.splice(index, 1);
      }
      if (selectedMailId.value === mailId) {
        clearSelection();
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      error.value = msg;
      useToastStore().add({ type: 'error', message: msg });
    }
  }

  async function bulkMove(targetFolderId: string) {
    for (const mailId of selectedMailIds.value) {
      await moveMail(mailId, targetFolderId);
    }
    clearBulkSelection();
  }

  return {
    mails,
    selectedMailId,
    selectedMail,
    selectedMailIds,
    lastSelectedIndex,
    folders,
    currentFolderId,
    currentAccountId,
    loading,
    loadingMore,
    hasMore,
    error,
    isReadingMode,
    deletingMailIds,
    totalUnread,
    loadFolders,
    loadMails,
    loadInboxMails,
    refreshMails,
    refreshInboxMails,
    insertMails,
    initEventListeners,
    loadMailDetail,
    selectMail,
    markRead,
    toggleStar,
    archiveMail,
    unarchiveMail,
    toggleSpam,
    deleteMail,
    toggleReadingMode,
    selectNextMail,
    selectPreviousMail,
    selectNextUnread,
    clearSelection,
    closeReader,
    toggleSelection,
    selectAll,
    clearBulkSelection,
    invertSelection,
    addToSelection,
    removeFromSelection,
    bulkMarkRead,
    bulkDelete,
    moveMail,
    bulkMove,
    activeFilter,
    setFilter,
  };
});
