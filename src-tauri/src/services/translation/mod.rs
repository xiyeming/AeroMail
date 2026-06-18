pub mod ai;
pub mod traditional;

use crate::db::pool::Database;
use crate::error::AeroError;
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub struct TranslationService {
    pub db: Arc<Database>,
}

impl TranslationService {
    #[must_use]
    pub const fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Translates `source_text` into `target_lang` using the given `provider_id`.
    ///
    /// Results are cached in the database keyed by SHA-256 hash.
    ///
    /// # Errors
    ///
    /// Returns an error if the text is empty, the provider is not found,
    /// or the translation API call fails.
    pub fn translate_mail(
        &self,
        source_text: &str,
        target_lang: &str,
        provider_id: &str,
    ) -> Result<String, AeroError> {
        if source_text.trim().is_empty() {
            return Err(AeroError::TranslationNoText);
        }

        let source_hash = sha256_hex(source_text);

        if let Some(cached) = self.db.get_translation(&source_hash, target_lang, provider_id)? {
            return Ok(cached.translated_text);
        }

        let provider = self.db.get_translation_provider(provider_id)?;

        let translated = match &provider {
            crate::models::translation::TranslationProvider::Traditional { .. } => {
                traditional::translate(&provider, source_text, target_lang)?
            }
            crate::models::translation::TranslationProvider::Ai { ai_provider_id, .. } => {
                let ai_provider = self.db.get_ai_provider(ai_provider_id)?;
                ai::translate(&ai_provider, source_text, target_lang)?
            }
        };

        self.db
            .save_translation(&source_hash, target_lang, provider_id, &translated)?;

        Ok(translated)
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
