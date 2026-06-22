export type ReplyKind = 'reply' | 'reply_all' | 'forward';

export interface ReplyContext {
  originalMailId: string;
  originalMessageId: string | null;
  kind: ReplyKind;
}

export interface AttachmentDraft {
  id: string;
  filename: string;
  mimeType: string;
  size: number;
  localPath?: string;
  contentId?: string;
  isInline: boolean;
  previewUrl?: string;
}

export interface ComposeDraft {
  id: string;
  accountId: string;
  replyContext?: ReplyContext;
  subject: string;
  to: string[];
  cc: string[];
  bcc: string[];
  bodyHtml: string;
  bodyText: string;
  attachments: AttachmentDraft[];
  savedAt: number;
  syncedAt?: number;
  remoteUid?: number;
}

export interface ComposeDraftSummary {
  id: string;
  accountId: string;
  subject: string;
  to: string[];
  savedAt: number;
  hasAttachments: boolean;
}

export interface SendMailRequest {
  draftId: string;
}
