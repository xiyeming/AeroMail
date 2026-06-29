use std::path::Path;
use std::sync::Arc;

use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, QueryParser, TermQuery};
use tantivy::schema::{Field, INDEXED, STORED, STRING, Schema, TEXT, Value};
use tantivy::{Index, IndexReader, ReloadPolicy};

use crate::db::pool::Database;
use crate::error::AeroError;
use crate::models::search::{SearchQuery, SearchResult, SearchStats};

pub struct SearchService {
    index: Index,
    reader: IndexReader,
    mail_id_field: Field,
    subject_field: Field,
    body_field: Field,
    from_field: Field,
    to_field: Field,
    date_field: Field,
    account_id_field: Field,
    folder_id_field: Field,
    db: Arc<Database>,
}

impl SearchService {
    /// Creates a new `SearchService` with Tantivy index.
    ///
    /// # Errors
    ///
    /// Returns an error if the index cannot be created or opened.
    pub fn new(db: Arc<Database>, index_path: &Path) -> Result<Self, AeroError> {
        // 创建索引目录
        std::fs::create_dir_all(index_path).map_err(|e| AeroError::Internal(e.to_string()))?;

        // 定义 schema
        let mut schema_builder = Schema::builder();
        let mail_id_field = schema_builder.add_text_field("mail_id", STRING | STORED);
        let subject_field = schema_builder.add_text_field("subject", TEXT | STORED);
        let body_field = schema_builder.add_text_field("body", TEXT);
        let from_field = schema_builder.add_text_field("from", TEXT | STORED);
        let to_field = schema_builder.add_text_field("to", TEXT);
        let date_field = schema_builder.add_date_field("date", INDEXED | STORED);
        let account_id_field = schema_builder.add_text_field("account_id", STRING);
        let folder_id_field = schema_builder.add_text_field("folder_id", STRING);
        let schema = schema_builder.build();

        // 打开或创建索引
        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(index_path)
                .map_err(|e| AeroError::SearchIndexError(e.to_string()))?,
            schema,
        )
        .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

        // 添加中文分词器
        index
            .tokenizers()
            .register("jieba", tantivy_jieba::JiebaTokenizer::default());

        // 创建 reader
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

        Ok(Self {
            index,
            reader,
            mail_id_field,
            subject_field,
            body_field,
            from_field,
            to_field,
            date_field,
            account_id_field,
            folder_id_field,
            db,
        })
    }

    /// Indexes a single mail.
    ///
    /// # Errors
    ///
    /// Returns an error if indexing fails.
    #[allow(clippy::too_many_arguments)]
    pub fn index_mail(
        &self,
        mail_id: &str,
        subject: Option<&str>,
        body_text: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
        date: Option<i64>,
        account_id: &str,
        folder_id: &str,
    ) -> Result<(), AeroError> {
        let mut writer: tantivy::IndexWriter = self
            .index
            .writer(50_000_000) // 50MB heap size
            .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

        // 删除旧文档
        writer.delete_term(tantivy::Term::from_field_text(self.mail_id_field, mail_id));

        // 构建文档
        let date = date.map_or_else(
            || tantivy::DateTime::from_timestamp_secs(0),
            tantivy::DateTime::from_timestamp_secs,
        );

        let mut doc = tantivy::TantivyDocument::default();
        doc.add_text(self.mail_id_field, mail_id);
        doc.add_text(self.subject_field, subject.unwrap_or(""));
        doc.add_text(self.body_field, body_text.unwrap_or(""));
        doc.add_text(self.from_field, from.unwrap_or(""));
        doc.add_text(self.to_field, to.unwrap_or(""));
        doc.add_date(self.date_field, date);
        doc.add_text(self.account_id_field, account_id);
        doc.add_text(self.folder_id_field, folder_id);

        writer
            .add_document(doc)
            .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

        writer
            .commit()
            .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

        // 更新数据库中的 indexed_at
        self.db.update_mail_indexed_at(mail_id)?;

        Ok(())
    }

    /// Indexes pending mails (not yet indexed) using a single shared writer
    /// and one commit at the end, avoiding the overhead of creating a 50 MB
    /// writer per mail.
    ///
    /// # Errors
    ///
    /// Returns an error if indexing fails.
    pub fn index_pending_mails(&self) -> Result<u64, AeroError> {
        let mails = self.db.get_unindexed_mails()?;
        if mails.is_empty() {
            return Ok(0);
        }

        let count = mails.len() as u64;

        let mut writer: tantivy::IndexWriter = self
            .index
            .writer(50_000_000) // 50MB heap size
            .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

        let mut indexed_ids: Vec<String> = Vec::with_capacity(mails.len());

        for mail in &mails {
            // 删除旧文档
            writer.delete_term(tantivy::Term::from_field_text(self.mail_id_field, &mail.id));

            let date = mail.date.map_or_else(
                || tantivy::DateTime::from_timestamp_secs(0),
                tantivy::DateTime::from_timestamp_secs,
            );

            let mut doc = tantivy::TantivyDocument::default();
            doc.add_text(self.mail_id_field, &mail.id);
            doc.add_text(self.subject_field, mail.subject.as_deref().unwrap_or(""));
            doc.add_text(self.body_field, mail.body_text.as_deref().unwrap_or(""));
            doc.add_text(self.from_field, mail.from_address.as_deref().unwrap_or(""));
            doc.add_text(self.to_field, mail.to_addresses.as_deref().unwrap_or(""));
            doc.add_date(self.date_field, date);
            doc.add_text(self.account_id_field, &mail.account_id);
            doc.add_text(self.folder_id_field, &mail.folder_id);

            writer
                .add_document(doc)
                .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

            indexed_ids.push(mail.id.clone());
        }

        // 一次提交所有文档
        writer
            .commit()
            .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

        // 批量更新数据库中的 indexed_at
        for mail_id in &indexed_ids {
            self.db.update_mail_indexed_at(mail_id)?;
        }

        Ok(count)
    }

    /// Searches mails using full-text search.
    ///
    /// # Errors
    ///
    /// Returns an error if search fails.
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, AeroError> {
        let searcher = self.reader.searcher();

        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.subject_field, self.body_field, self.from_field],
        );

        // Parse the free-text portion of the query.
        let user_query = if query.query.trim().is_empty() {
            // Match all documents when no text query is provided.
            Box::new(tantivy::query::AllQuery) as Box<dyn tantivy::query::Query>
        } else {
            query_parser
                .parse_query(&query.query)
                .map_err(|e| AeroError::SearchQueryError(e.to_string()))?
        };

        // Build filter queries for account and folder using TermQuery to avoid
        // query-string injection.
        let mut clauses: Vec<(Occur, Box<dyn tantivy::query::Query>)> =
            vec![(Occur::Must, user_query)];

        if let Some(ref account_id) = query.account_id {
            let term = tantivy::Term::from_field_text(self.account_id_field, account_id);
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    term,
                    tantivy::schema::IndexRecordOption::Basic,
                )),
            ));
        }
        if let Some(ref folder_id) = query.folder_id {
            let term = tantivy::Term::from_field_text(self.folder_id_field, folder_id);
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    term,
                    tantivy::schema::IndexRecordOption::Basic,
                )),
            ));
        }

        let final_query = BooleanQuery::new(clauses);

        let top_docs = searcher
            .search(&final_query, &TopDocs::with_limit(100).order_by_score())
            .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

        let mut results = Vec::new();
        for (score, doc_addr) in top_docs {
            let doc = searcher
                .doc::<tantivy::TantivyDocument>(doc_addr)
                .map_err(|e| AeroError::SearchIndexError(e.to_string()))?;

            if let Some(mail_id) = doc.get_first(self.mail_id_field).and_then(|v| v.as_str()) {
                // Generate snippet using the original free-text query only.
                let snippet = self.generate_snippet(&doc, &query.query);

                results.push(SearchResult {
                    mail_id: mail_id.to_string(),
                    score,
                    snippet,
                });
            }
        }

        Ok(results)
    }

    /// Generates a snippet for search results.
    fn generate_snippet(&self, doc: &tantivy::TantivyDocument, query_str: &str) -> Option<String> {
        // 获取正文
        let body = doc.get_first(self.body_field)?.as_str()?;

        // 简单的高亮处理：查找关键词并添加高亮标记
        let snippet = Self::highlight_text(body, query_str, 200);

        if !snippet.is_empty() {
            return Some(snippet);
        }

        // 如果没有匹配，返回正文的前 200 个字符
        if body.len() > 200 {
            Some(format!("{}...", &body[..200]))
        } else {
            Some(body.to_string())
        }
    }

    /// Highlights search terms in text.
    fn highlight_text(text: &str, query: &str, max_length: usize) -> String {
        let query_trimmed = query.trim();
        if query_trimmed.is_empty() {
            return String::new();
        }

        let Some((start, end)) = Self::find_case_insensitive(text, query_trimmed) else {
            return String::new();
        };

        let snippet_start = start.saturating_sub(50);
        let snippet_end = std::cmp::min(text.len(), end + 150);
        let snippet_end = std::cmp::min(snippet_end, snippet_start + max_length);

        // Ensure slice boundaries fall on UTF-8 char boundaries.
        let snippet = text
            .char_indices()
            .skip_while(|(idx, _)| *idx < snippet_start)
            .take_while(|(idx, _)| *idx < snippet_end)
            .map(|(_, c)| c)
            .collect::<String>();
        format!("...{snippet}...")
    }

    /// Finds the first case-insensitive occurrence of `needle` in `haystack`
    /// and returns its byte range in the original text.
    fn find_case_insensitive(haystack: &str, needle: &str) -> Option<(usize, usize)> {
        if needle.is_empty() {
            return None;
        }
        let needle_lower = needle.to_lowercase();
        let haystack_chars: Vec<(usize, char)> = haystack.char_indices().collect();
        let needle_len = needle_lower.chars().count();

        for i in 0..haystack_chars.len().saturating_sub(needle_len - 1) {
            let window = &haystack_chars[i..i + needle_len];
            let window_lower: String = window.iter().flat_map(|(_, c)| c.to_lowercase()).collect();
            if window_lower == needle_lower {
                let start = window.first().map_or(0, |(idx, _)| *idx);
                let end = haystack_chars
                    .get(i + needle_len)
                    .map_or(haystack.len(), |(idx, _)| *idx);
                return Some((start, end));
            }
        }
        None
    }

    /// Returns search index statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if stats cannot be retrieved.
    pub fn stats(&self) -> Result<SearchStats, AeroError> {
        let searcher = self.reader.searcher();
        let total = searcher.num_docs();

        Ok(SearchStats {
            total_indexed: total,
            last_index_time: None, // TODO: 跟踪最后索引时间
        })
    }
}
