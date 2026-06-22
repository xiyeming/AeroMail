import { ref } from 'vue';
import { useTauriInvoke } from '@/composables/useTauriInvoke';
import type { AiChatSession, AiChatMessage } from '@/types/ai';

export function useAiChat() {
  const { call } = useTauriInvoke();
  const sessions = ref<AiChatSession[]>([]);
  const messages = ref<AiChatMessage[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const activeSessionId = ref<string | null>(null);

  async function loadSessions() {
    sessions.value = await call<AiChatSession[]>('list_chat_sessions');
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
    try {
      await call('send_chat_message', {
        sessionId,
        content,
      });

      await loadMessages(sessionId);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
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

  function selectSession(id: string) {
    activeSessionId.value = id;
    void loadMessages(id);
  }

  return {
    sessions,
    messages,
    isLoading,
    error,
    activeSessionId,
    loadSessions,
    createSession,
    sendMessage,
    loadMessages,
    selectSession,
  };
}
