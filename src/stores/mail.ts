import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { MailHeader, MailDetail, FolderInfo } from '@/types/mail';

const PAGE_SIZE = 50;

const VIRTUAL_FOLDERS = ['starred', 'sent', 'archived', 'spam'];

export const useMailStore = defineStore('mail', () => {
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

  const totalUnread = computed(() => folders.value.reduce((sum, f) => sum + f.unreadCount, 0));

  async function loadFolders(accountId: string) {
    try {
      currentAccountId.value = accountId;
      const newFolders = await invoke<FolderInfo[]>('list_folders', { accountId });
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
        mails.value = [];
        hasMore.value = true;
      } else {
        loadingMore.value = true;
      }

      const offset = reset ? 0 : mails.value.length;
      const newMails = await invoke<MailHeader[]>('get_inbox_mail_list', {
        accountIds,
        limit: PAGE_SIZE,
        offset,
      });

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
        mails.value = [];
        hasMore.value = true;
      } else {
        loadingMore.value = true;
      }

      currentFolderId.value = folderId;
      const isVirtual = VIRTUAL_FOLDERS.includes(folderId);
      const offset = reset ? 0 : mails.value.length;
      let newMails = await invoke<MailHeader[]>(
        isVirtual ? 'get_virtual_mail_list' : 'get_mail_list',
        {
          folderId,
          limit: PAGE_SIZE,
          offset,
        }
      );

      // When the local page is empty and we expect more, ask the backend to
      // backfill older messages from the IMAP server, then try the same page
      // again. Only do this for real folders; virtual folders have no remote
      // older-mail source.
      if (!reset && newMails.length === 0 && hasMore.value && !isVirtual) {
        const fetched = await invoke<number>('fetch_older_mails', {
          folderId,
          limit: PAGE_SIZE,
        });
        if (fetched > 0) {
          newMails = await invoke<MailHeader[]>('get_mail_list', {
            folderId,
            limit: PAGE_SIZE,
            offset,
          });
        }
      }

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
      const fetched = await invoke<MailHeader[]>(
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
      const fetched = await invoke<MailHeader[]>('get_inbox_mail_list', {
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

  async function loadMailDetail(mailId: string) {
    try {
      selectedMail.value = await invoke<MailDetail>('get_mail_detail', { mailId });
      selectedMailId.value = mailId;

      // Mark as read
      if (selectedMail.value && !selectedMail.value.isRead) {
        await markRead(mailId, true);
        // Update in list
        const mail = mails.value.find((m) => m.id === mailId);
        if (mail) {
          mail.isRead = true;
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function selectMail(mailId: string) {
    await loadMailDetail(mailId);
  }

  async function markRead(mailId: string, isRead: boolean) {
    try {
      await invoke('mark_mail_read', { mailId, isRead });
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function toggleStar(mailId: string) {
    try {
      const newStarred = await invoke<boolean>('toggle_mail_star', { mailId });
      const mail = mails.value.find((m) => m.id === mailId);
      if (mail) {
        mail.isStarred = newStarred;
      }
      if (selectedMail.value?.id === mailId) {
        selectedMail.value.isStarred = newStarred;
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function archiveMail(mailId: string) {
    try {
      await invoke('archive_mail', { mailId });
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
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function toggleSpam(mailId: string) {
    try {
      const newSpam = await invoke<boolean>('toggle_mail_spam', { mailId });
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
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  async function deleteMail(mailId: string) {
    try {
      await invoke('delete_mail', { mailId });
      // Remove from list
      const index = mails.value.findIndex((m) => m.id === mailId);
      if (index !== -1) {
        mails.value.splice(index, 1);
      }
      // Clear selection if deleted mail was selected
      if (selectedMailId.value === mailId) {
        clearSelection();
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
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
    selectedMailIds.value = mails.value
      .filter((m) => !current.has(m.id))
      .map((m) => m.id);
    lastSelectedIndex.value = -1;
  }

  // Batch actions
  async function bulkMarkRead(isRead: boolean) {
    for (const mailId of selectedMailIds.value) {
      await markRead(mailId, isRead);
      const mail = mails.value.find((m) => m.id === mailId);
      if (mail) {
        mail.isRead = isRead;
      }
    }
  }

  async function bulkDelete() {
    for (const mailId of selectedMailIds.value) {
      await deleteMail(mailId);
    }
    clearBulkSelection();
  }

  async function moveMail(mailId: string, targetFolderId: string) {
    try {
      await invoke('move_mail', { mailId, targetFolderId });
      // Remove from list
      const index = mails.value.findIndex((m) => m.id === mailId);
      if (index !== -1) {
        mails.value.splice(index, 1);
      }
      if (selectedMailId.value === mailId) {
        clearSelection();
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
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
    totalUnread,
    loadFolders,
    loadMails,
    loadInboxMails,
    refreshMails,
    refreshInboxMails,
    loadMailDetail,
    selectMail,
    markRead,
    toggleStar,
    archiveMail,
    toggleSpam,
    deleteMail,
    toggleReadingMode,
    selectNextMail,
    selectPreviousMail,
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
  };
});
