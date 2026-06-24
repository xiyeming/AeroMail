<template>
  <div class="relative flex h-full flex-col bg-base text-primary">
    <ComposeHeader
      :draft="store.draft"
      :accounts="accounts"
      :sending="store.loading"
      @update:draft="store.setDraft"
      @send="store.sendMail"
    />
    <ComposeEditor
      :model-value="store.draft.bodyHtml"
      @change="({ html, text }) => store.updateBody(html, text)"
      @image-pasted="handleImagePasted"
    />
    <ComposeAttachmentList :attachments="store.draft.attachments" @remove="removeAttachment" />

    <div class="flex items-center gap-3 border-t border-border p-3">
      <button
        type="button"
        class="rounded-md border border-border bg-elevated px-3 py-2 text-sm text-primary transition-colors hover:bg-raised disabled:opacity-50"
        :disabled="isAttaching"
        @click="addAttachment"
      >
        {{ isAttaching ? $t('compose.attaching') : $t('compose.addAttachment') }}
      </button>
      <span v-if="store.saving" class="text-xs text-tertiary">{{ $t('compose.saving') }}</span>
      <span v-else-if="store.draft.savedAt > 0" class="text-xs text-tertiary">{{
        $t('compose.saved')
      }}</span>
    </div>

    <!-- Sending overlay -->
    <div
      v-if="store.loading"
      class="absolute inset-0 z-50 flex flex-col items-center justify-center gap-3 bg-base/80 backdrop-blur-sm"
    >
      <Loader2 class="h-8 w-8 animate-spin text-accent" />
      <p class="text-sm font-medium text-primary">{{ $t('compose.sending') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, computed, ref } from 'vue';
import { useRoute } from 'vue-router';
import { Loader2 } from 'lucide-vue-next';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import { useComposeStore } from '@/stores/compose';
import { useAccountStore } from '@/stores/account';
import ComposeHeader from '@/components/compose/ComposeHeader.vue';
import ComposeEditor from '@/components/compose/ComposeEditor.vue';
import ComposeAttachmentList from '@/components/compose/ComposeAttachmentList.vue';
import type { AttachmentDraft } from '@/types/compose';
import { useToastStore } from '@/stores/toast';
import { useI18n } from 'vue-i18n';

const route = useRoute();
const store = useComposeStore();
const accountStore = useAccountStore();
const toast = useToastStore();
const { call } = useTauriInvoke();
const { t } = useI18n();

const accounts = computed(() => accountStore.accounts);
const isAttaching = ref(false);

onMounted(async () => {
  await accountStore.loadAccounts();

  const draftId = route.params.draftId as string | undefined;
  const mailId = route.params.mailId as string | undefined;
  const kind = route.name as string | undefined;

  if (draftId) {
    await store.loadDraft(draftId);
  } else if (mailId && kind) {
    const replyKind = kind === 'reply-all' ? 'reply_all' : (kind as 'reply' | 'forward');
    await store.prepareReply(mailId, replyKind);
  } else {
    store.reset();
    store.setDraft({
      ...store.draft,
      accountId: accounts.value[0]?.id ?? '',
    });
  }
});

async function ensureDraftId(): Promise<string | null> {
  if (store.draft.id) return store.draft.id;
  if (!store.draft.accountId) {
    toast.add({
      type: 'error',
      message: t('compose.noAccountSelected'),
      duration: 5000,
    });
    return null;
  }
  await store.saveNow();
  return store.draft.id || null;
}

function readFileAsArrayBuffer(file: File): Promise<ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as ArrayBuffer);
    reader.onerror = () =>
      reject(new Error(t('compose.attachmentReadFailed', { name: file.name })));
    reader.readAsArrayBuffer(file);
  });
}

async function handleImagePasted(file: File) {
  const draftId = await ensureDraftId();
  if (!draftId) return;

  isAttaching.value = true;
  try {
    const arrayBuffer = await readFileAsArrayBuffer(file);
    const base64 = btoa(
      new Uint8Array(arrayBuffer).reduce((data, byte) => data + String.fromCharCode(byte), '')
    );
    const contentId = `att-${crypto.randomUUID()}`;
    const attachment: AttachmentDraft = {
      id: crypto.randomUUID(),
      filename: file.name,
      mimeType: file.type,
      size: file.size,
      isInline: true,
      contentId,
      previewUrl: `data:${file.type};base64,${base64}`,
    };
    await call('save_attachment', {
      draftId,
      attachment,
      data: Array.from(new Uint8Array(arrayBuffer)),
    });
    store.draft.attachments.push(attachment);
    const imgHtml = `<img src="cid:${contentId}" alt="${attachment.filename}" />`;
    store.draft.bodyHtml += imgHtml;
    await store.saveNow();
  } catch (e) {
    toast.add({
      type: 'error',
      message: e instanceof Error ? e.message : String(e),
      duration: 5000,
    });
  } finally {
    isAttaching.value = false;
  }
}

async function addAttachment() {
  const draftId = await ensureDraftId();
  if (!draftId) return;

  const input = document.createElement('input');
  input.type = 'file';
  input.multiple = true;
  input.onchange = async () => {
    const files = Array.from(input.files ?? []);
    if (files.length === 0) return;

    isAttaching.value = true;
    try {
      for (const file of files) {
        const arrayBuffer = await readFileAsArrayBuffer(file);
        const attachment: AttachmentDraft = {
          id: crypto.randomUUID(),
          filename: file.name,
          mimeType: file.type,
          size: file.size,
          isInline: false,
        };
        await call('save_attachment', {
          draftId,
          attachment,
          data: Array.from(new Uint8Array(arrayBuffer)),
        });
        store.draft.attachments.push(attachment);
      }
      await store.saveNow();
    } catch (e) {
      toast.add({
        type: 'error',
        message: e instanceof Error ? e.message : String(e),
        duration: 5000,
      });
    } finally {
      isAttaching.value = false;
    }
  };
  input.click();
}

async function removeAttachment(attachmentId: string) {
  store.draft.attachments = store.draft.attachments.filter((a) => a.id !== attachmentId);
  await store.saveNow();
}
</script>
