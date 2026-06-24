export interface MailHeader {
  id: string;
  accountId: string;
  folderId: string;
  uid: number;
  subject: string | null;
  fromName: string | null;
  fromAddress: string | null;
  date: number | null;
  isRead: boolean;
  isStarred: boolean;
  isArchived: boolean;
  isSpam: boolean;
  hasAttachments: boolean;
}

export interface MailDetail extends MailHeader {
  toAddresses: string | null;
  ccAddresses: string | null;
  bodyHtml: string | null;
  bodyText: string | null;
  flags: string | null;
  messageId: string | null;
}

export interface FolderInfo {
  id: string;
  accountId: string;
  name: string;
  path: string;
  unreadCount: number;
  totalCount: number;
  uidValidity: number | null;
  lastSyncAt: number | null;
}

export interface SyncProgress {
  accountId: string;
  status: 'idle' | 'syncing' | 'error' | 'completed';
  syncedCount: number;
  totalCount: number;
  lastSyncTime: string | null;
  message?: string;
}

export interface AttachmentInfo {
  id: string;
  mailId: string;
  filename: string;
  mimeType: string;
  size: number;
  contentId: string | null;
  isInline: boolean;
}
