use std::path::PathBuf;

use fig_util::directories::fig_data_dir;
use tokio::fs;

use crate::Result;
use super::Secret;
use super::sqlite::SqliteSecretStore;

pub struct SecretStoreImpl {
    sqlite: SqliteSecretStore,
}

impl SecretStoreImpl {
    pub async fn new() -> Result<Self> {
        // On Windows, we'll use SQLite for storing secrets
        // This is a simpler approach than using the Windows Credential Manager
        let data_dir = fig_data_dir()?;
        let db_path = get_db_path(data_dir);
        
        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Use the default SQLite implementation for now
        // In the future, we can implement a Windows-specific version using the Windows Credential Manager
        let sqlite = SqliteSecretStore::new().await?;
        Ok(Self { sqlite })
    }

    pub async fn set(&self, key: &str, password: &str) -> Result<()> {
        self.sqlite.set(key, password).await
    }

    pub async fn get(&self, key: &str) -> Result<Option<Secret>> {
        self.sqlite.get(key).await
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        self.sqlite.delete(key).await
    }
}

fn get_db_path(data_dir: PathBuf) -> PathBuf {
    data_dir.join("secrets").join("secrets.db")
}

impl std::fmt::Debug for SecretStoreImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretStoreImpl").finish()
    }
}
