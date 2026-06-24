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
  thinking?: string | null;
  createdAt: number;
}

export interface AiUsageSummary {
  providerId: string;
  model: string;
  totalPromptTokens: number;
  totalCompletionTokens: number;
  totalTokens: number;
  totalCost: number;
  currency: string;
}

export interface AiProviderPricing {
  id: string;
  providerId: string;
  model: string;
  inputPricePer1K: number;
  outputPricePer1K: number;
  currency: string;
  effectiveFrom: number | null;
}

export type AiMcpTransport = 'stdio' | 'sse';

export interface AiMcpServer {
  id: string;
  name: string;
  transport: AiMcpTransport;
  command?: string;
  args?: string[];
  url?: string;
  envJson?: string;
  isEnabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface AiSkill {
  id: string;
  name: string;
  description: string;
  inputSchemaJson: string;
  command: string;
  args?: string[];
  workingDir?: string;
  timeoutSeconds?: number;
  isEnabled: boolean;
  createdAt: number;
  updatedAt: number;
}
