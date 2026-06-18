export type AiProviderKind =
  | 'openai'
  | 'anthropic'
  | 'gemini'
  | 'azure_openai'
  | 'deepseek'
  | 'moonshot'
  | 'qwen'
  | 'zhipu'
  | 'minimax'
  | 'baichuan'
  | 'custom_openai_compatible';

export interface AiProvider {
  id: string;
  name: string;
  kind: AiProviderKind;
  apiKeyEncrypted: number[];
  baseUrl?: string;
  model: string;
  maxTokens?: number;
}

export interface AiProviderSummary {
  id: string;
  name: string;
  kind: string;
  model: string;
}

export interface AiChatSession {
  id: string;
  title: string | null;
  providerId: string;
  model: string;
  contextMailId: string | null;
  createdAt: number;
  updatedAt: number;
}

export interface AiChatMessage {
  id: string;
  sessionId: string;
  role: 'system' | 'user' | 'assistant';
  content: string;
  createdAt: number;
}
