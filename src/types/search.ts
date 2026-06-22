export interface SearchQuery {
  query: string;
  folderId?: string;
  accountId?: string;
  dateFrom?: number;
  dateTo?: number;
  hasAttachment?: boolean;
  isRead?: boolean;
}

export interface SearchResult {
  mailId: string;
  score: number;
  snippet: string | null;
}

export interface SearchStats {
  totalIndexed: number;
  lastIndexTime: string | null;
}
