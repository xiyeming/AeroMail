import { ref } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import type { AiChatSession, AiChatMessage, AiUsageSummary } from '@/types/ai';

export function useAiChat() {
  const { call } = useTauriInvoke();
  const sessions = ref<AiChatSession[]>([]);
  const messages = ref<AiChatMessage[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const activeSessionId = ref<string | null>(null);
  const usageSummary = ref<AiUsageSummary[]>([]);
  const sessionUsage = ref<AiUsageSummary | null>(null);

  async function loadSessions() {
    sessions.value = await call<AiChatSession[]>('list_chat_sessions');
  }

  async function loadUsageSummary() {
    usageSummary.value = await call<AiUsageSummary[]>('get_ai_usage_summary', {
      providerId: null,
    });
  }

  async function loadSessionUsage(sessionId: string) {
    sessionUsage.value = await call<AiUsageSummary | null>('get_ai_session_usage', {
      sessionId,
    });
  }

  async function createSession(providerId: string, contextMailId?: string) {
    const session = await call<AiChatSession>('create_chat_session', {
      providerId,
      contextMailId,
    });
    sessions.value.unshift(session);
    activeSessionId.value = session.id;
    return session;
  }

  async function sendMessage(sessionId: string, content: string) {
    isLoading.value = true;
    error.value = null;

    // Optimistically render the user's message immediately.
    const tempId = `temp-${Date.now()}`;
    messages.value.push({
      id: tempId,
      sessionId,
      role: 'user',
      content,
      createdAt: Math.floor(Date.now() / 1000),
    });

    try {
      await call('send_chat_message', {
        sessionId,
        content,
      });

      await loadMessages(sessionId);
      await loadSessionUsage(sessionId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      // Remove the optimistic message on failure so the user can retry.
      messages.value = messages.value.filter((msg) => msg.id !== tempId);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function loadMessages(sessionId: string) {
    messages.value = await call<AiChatMessage[]>('get_chat_messages', {
      sessionId,
    });
  }

  async function deleteSession(sessionId: string) {
    await call('delete_chat_session', { sessionId });
    sessions.value = sessions.value.filter((s) => s.id !== sessionId);
    if (activeSessionId.value === sessionId) {
      activeSessionId.value = null;
      messages.value = [];
    }
  }

  async function clearSession(sessionId: string) {
    await call('clear_chat_session', { sessionId });
    if (activeSessionId.value === sessionId) {
      messages.value = [];
    }
  }

  async function renameSession(sessionId: string, title: string) {
    await call('rename_chat_session', { sessionId, title });
    const session = sessions.value.find((s) => s.id === sessionId);
    if (session) {
      session.title = title;
    }
  }

  async function quoteMail(sessionId: string, mailId: string) {
    await call('quote_mail_to_chat', { sessionId, mailId });
    if (activeSessionId.value === sessionId) {
      await loadMessages(sessionId);
    }
  }

  async function setSessionProvider(sessionId: string, providerId: string) {
    await call('set_chat_session_provider', { sessionId, providerId });
    const session = sessions.value.find((s) => s.id === sessionId);
    if (session) {
      session.providerId = providerId;
    }
  }

  function selectSession(id: string) {
    activeSessionId.value = id;
    void loadMessages(id);
    void loadSessionUsage(id);
  }

  async function summarizeMail(mailId: string, providerId: string) {
    return await call<string>('summarize_mail', { mailId, providerId });
  }

  async function extractTodos(mailId: string, providerId: string) {
    return await call<string[]>('extract_todos', { mailId, providerId });
  }

  return {
    sessions,
    messages,
    isLoading,
    error,
    activeSessionId,
    usageSummary,
    sessionUsage,
    loadSessions,
    loadUsageSummary,
    loadSessionUsage,
    createSession,
    sendMessage,
    loadMessages,
    selectSession,
    deleteSession,
    clearSession,
    renameSession,
    setSessionProvider,
    quoteMail,
    summarizeMail,
    extractTodos,
  };
}
