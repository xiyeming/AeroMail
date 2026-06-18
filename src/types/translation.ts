export type TraditionalProviderKind =
  | 'google_translate'
  | 'deep_l'
  | 'azure_translator'
  | 'baidu'
  | 'youdao'
  | 'tencent_translator'
  | 'aliyun_translator'
  | 'custom';

export interface TraditionalTranslationProvider {
  type: 'traditional';
  id: string;
  name: string;
  kind: TraditionalProviderKind;
  endpoint?: string;
  extra: Record<string, string>;
}

export interface AiTranslationProvider {
  type: 'ai';
  id: string;
  name: string;
  aiProviderId: string;
  promptTemplate?: string;
}

export type TranslationProvider = TraditionalTranslationProvider | AiTranslationProvider;

export interface TranslationProviderSummary {
  id: string;
  name: string;
  providerType: string;
}
