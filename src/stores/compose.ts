import { ref, computed } from 'vue';
import { defineStore } from 'pinia';
import { useDebounceFn } from '@vueuse/core';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import type {
  ComposeDraft,
  ComposeDraftSummary,
  ReplyKind,
} from '@/types/compose';
import { useToastStore } from './toast';
import router from '@/router';

const emptyDraft = (): ComposeDraft => ({
  id: '',
  accountId: '',
  replyContext: undefined,
  subject: '',
  to: [],
  cc: [],
  bcc: [],
  bodyHtml: '',
  bodyText: '',
  attachments: [],
  savedAt: 0,
});

export const useComposeStore = defineStore('compose', () => {
  const { call } = useTauriInvoke();
  const draft = ref<ComposeDraft>(emptyDraft());
  const drafts = ref<ComposeDraftSummary[]>([]);
  const loading = ref(false);
  const saving = ref(false);
  const lastError = ref<string | null>(null);

  const hasDraft = computed(() => draft.value.id !== '');

  function setDraft(value: ComposeDraft) {
    draft.value = value;
  }

  function updateField<K extends keyof ComposeDraft>(
    key: K,
    value: ComposeDraft[K]
  ) {
    draft.value[key] = value;
    triggerAutosave();
  }

  function updateBody(html: string, text: string) {
    draft.value.bodyHtml = html;
    draft.value.bodyText = text;
    triggerAutosave();
  }

  const saveToBackend = useDebounceFn(async () => {
    if (!draft.value.accountId) return;
    saving.value = true;
    try {
      const saved = await call<ComposeDraft>('save_draft', {
        draft: draft.value,
      });
      draft.value.id = saved.id;
      draft.value.savedAt = saved.savedAt;
      lastError.value = null;
      triggerImapSync();
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      useToastStore().add({
        type: 'error',
        message: lastError.value,
        duration: 5000,
      });
    } finally {
      saving.value = false;
    }
  }, 2000);

  const triggerAutosave = () => {
    saveToBackend();
  };

  const syncToImap = useDebounceFn(async () => {
    if (!draft.value.id) return;
    try {
      await call('sync_draft_to_imap', { draftId: draft.value.id });
    } catch (e) {
      console.warn('IMAP draft sync failed', e);
    }
  }, 5000);

  const triggerImapSync = () => {
    syncToImap();
  };

  async function loadDraft(draftId: string) {
    loading.value = true;
    try {
      const loaded = await call<ComposeDraft>('get_draft', { draftId });
      draft.value = loaded;
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      useToastStore().add({
        type: 'error',
        message: lastError.value,
        duration: 5000,
      });
    } finally {
      loading.value = false;
    }
  }

  async function loadDrafts(accountId?: string) {
    drafts.value = await call<ComposeDraftSummary[]>('get_drafts', {
      accountId: accountId ?? null,
    });
  }

  async function deleteDraft(draftId: string) {
    await call('delete_draft', { draftId });
    drafts.value = drafts.value.filter((d) => d.id !== draftId);
  }

  async function prepareReply(mailId: string, kind: ReplyKind) {
    loading.value = true;
    try {
      const reply = await call<ComposeDraft>('prepare_reply', {
        mailId,
        kind,
      });
      draft.value = reply;
    } finally {
      loading.value = false;
    }
  }

  async function sendMail() {
    if (!draft.value.id) {
      await saveToBackend();
    }
    if (!draft.value.id) {
      useToastStore().add({
        type: 'error',
        message: 'Failed to save draft before sending',
        duration: 5000,
      });
      return;
    }
    loading.value = true;
    try {
      await call('send_mail', { draftId: draft.value.id });
      useToastStore().add({
        type: 'success',
        message: 'Mail sent',
        duration: 3000,
      });
      draft.value = emptyDraft();
      await router.push({ name: 'inbox' });
    } catch (e) {
      lastError.value = e instanceof Error ? e.message : String(e);
      useToastStore().add({
        type: 'error',
        message: lastError.value,
        duration: 5000,
      });
    } finally {
      loading.value = false;
    }
  }

  function reset() {
    draft.value = emptyDraft();
  }

  return {
    draft,
    drafts,
    loading,
    saving,
    lastError,
    hasDraft,
    setDraft,
    updateField,
    updateBody,
    loadDraft,
    loadDrafts,
    deleteDraft,
    prepareReply,
    sendMail,
    reset,
  };
});
