import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AiChatSession, AiChatMessage } from '@/types/ai';

export function useAiChat() {
  const sessions = ref<AiChatSession[]>([]);
  const messages = ref<AiChatMessage[]>([]);
  const isLoading = ref(false);
  const activeSessionId = ref<string | null>(null);

  async function loadSessions() {
    sessions.value = await invoke<AiChatSession[]>('list_chat_sessions');
  }

  async function createSession(providerId: string, contextMailId?: string) {
    const session = await invoke<AiChatSession>('create_chat_session', {
      providerId,
      contextMailId,
    });
    sessions.value.unshift(session);
    activeSessionId.value = session.id;
    return session;
  }

  async function sendMessage(sessionId: string, content: string) {
    isLoading.value = true;
    try {
      await invoke('send_chat_message', {
        sessionId,
        content,
      });

      await loadMessages(sessionId);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadMessages(sessionId: string) {
    messages.value = await invoke<AiChatMessage[]>('get_chat_messages', {
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
    activeSessionId,
    loadSessions,
    createSession,
    sendMessage,
    loadMessages,
    selectSession,
  };
}
