use crate::db::pool::Database;
use std::sync::Arc;

pub struct AccountManager {
    #[allow(dead_code)]
    db: Arc<Database>,
}

impl AccountManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}
