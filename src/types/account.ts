export type MailProvider =
  | 'Gmail'
  | 'Outlook'
  | 'QQ'
  | 'Netease163'
  | 'Aliyun'
  | 'TencentExmail'
  | 'Custom';

export type TlsMode = 'required' | 'starttls' | 'none';

export interface ServerConfig {
  host: string;
  port: number;
  tlsMode: TlsMode;
}

export type AuthConfig =
  | {
      type: 'OAuth2';
      accessToken: string;
      refreshToken: string;
      expiresAt: number;
    }
  | {
      type: 'Password';
      passwordEncrypted: number[];
    };

export interface AdvancedConfig {
  caCertPath: string | null;
  verifyCertificate: boolean;
  connectTimeoutSecs: number;
  readTimeoutSecs: number;
  keepalive: boolean;
}

export interface AccountConfig {
  id: string;
  name: string;
  provider: MailProvider;
  imap: ServerConfig;
  smtp: ServerConfig;
  auth: AuthConfig;
  advanced: AdvancedConfig;
  syncIntervalSecs: number;
  excludedFolders: string[];
}

export interface AccountSummary {
  id: string;
  name: string;
  provider: MailProvider;
  imapHost: string;
  smtpHost: string;
}
