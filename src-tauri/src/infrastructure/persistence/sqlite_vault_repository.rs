use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use sha2::{Digest, Sha256};

use crate::application::dto::sync::{
    SyncCipher, SyncCollection, SyncDomains, SyncFolder, SyncPolicy, SyncProfile, SyncSend,
    SyncUserDecryption,
};
use crate::application::ports::vault_repository_port::VaultRepositoryPort;
use crate::domain::sync::{SyncContext, SyncItemCounts, SyncState, WsStatus};
use crate::support::error::AppError;
use crate::support::result::AppResult;

pub struct SqliteVaultRepository {
    db_dir: PathBuf,
    connections: Mutex<HashMap<String, Connection>>,
}

impl SqliteVaultRepository {
    pub fn new<P: AsRef<Path>>(db_dir: P) -> AppResult<Self> {
        let db_dir = db_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&db_dir).map_err(|error| {
            AppError::internal(format!(
                "failed to create sqlite directory {}: {error}",
                db_dir.display()
            ))
        })?;

        Ok(Self {
            db_dir,
            connections: Mutex::new(HashMap::new()),
        })
    }

    fn with_account_connection<T>(
        &self,
        account_id: &str,
        f: impl FnOnce(&mut Connection) -> AppResult<T>,
    ) -> AppResult<T> {
        let mut connections = self
            .connections
            .lock()
            .map_err(|_| AppError::internal("failed to lock sqlite connections"))?;

        if !connections.contains_key(account_id) {
            let db_path = self.db_path_for_account(account_id);
            let connection = Self::open_connection(&db_path)?;
            connections.insert(account_id.to_string(), connection);
        }

        let connection = connections.get_mut(account_id).ok_or_else(|| {
            AppError::internal(format!(
                "failed to get sqlite connection for account_id={account_id}"
            ))
        })?;
        f(connection)
    }

    fn db_path_for_account(&self, account_id: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(account_id.as_bytes());
        let digest = hasher.finalize();
        let mut hex = String::with_capacity(64);
        for value in digest {
            hex.push_str(&format!("{value:02x}"));
        }
        self.db_dir.join(format!("account-{hex}.sqlite3"))
    }

    fn open_connection(db_path: &Path) -> AppResult<Connection> {
        let connection = Connection::open(db_path).map_err(|error| {
            AppError::internal(format!(
                "failed to open sqlite database {}: {error}",
                db_path.display()
            ))
        })?;
        Self::initialize_schema(&connection)?;
        Ok(connection)
    }

    fn initialize_schema(connection: &Connection) -> AppResult<()> {
        connection
            .execute_batch(
                r#"
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS sync_contexts (
                    account_id TEXT PRIMARY KEY,
                    base_url TEXT,
                    state TEXT NOT NULL,
                    ws_status TEXT NOT NULL,
                    last_revision_ms INTEGER,
                    last_sync_at_ms INTEGER,
                    last_error TEXT,
                    counts_folders INTEGER NOT NULL DEFAULT 0,
                    counts_collections INTEGER NOT NULL DEFAULT 0,
                    counts_policies INTEGER NOT NULL DEFAULT 0,
                    counts_ciphers INTEGER NOT NULL DEFAULT 0,
                    counts_sends INTEGER NOT NULL DEFAULT 0
                );

                CREATE TABLE IF NOT EXISTS sync_transactions (
                    account_id TEXT PRIMARY KEY,
                    started_at_ms INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS live_profile (
                    account_id TEXT PRIMARY KEY,
                    payload_json TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS live_folders (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS live_collections (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS live_policies (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS live_ciphers (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS live_sends (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS live_meta (
                    account_id TEXT PRIMARY KEY,
                    domains_json TEXT,
                    user_decryption_json TEXT
                );

                CREATE TABLE IF NOT EXISTS staging_profile (
                    account_id TEXT PRIMARY KEY,
                    payload_json TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS staging_folders (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS staging_collections (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS staging_policies (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS staging_ciphers (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS staging_sends (
                    account_id TEXT NOT NULL,
                    id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    PRIMARY KEY (account_id, id)
                );
                CREATE TABLE IF NOT EXISTS staging_meta (
                    account_id TEXT PRIMARY KEY,
                    domains_json TEXT,
                    user_decryption_json TEXT
                );
                "#,
            )
            .map_err(|error| {
                AppError::internal(format!("failed to initialize sqlite schema: {error}"))
            })?;
        Ok(())
    }

    fn ensure_context_row(connection: &Connection, account_id: &str) -> AppResult<()> {
        connection
            .execute(
                r#"
                INSERT INTO sync_contexts (
                    account_id,
                    base_url,
                    state,
                    ws_status,
                    last_revision_ms,
                    last_sync_at_ms,
                    last_error,
                    counts_folders,
                    counts_collections,
                    counts_policies,
                    counts_ciphers,
                    counts_sends
                ) VALUES (?1, NULL, 'idle', 'unknown', NULL, NULL, NULL, 0, 0, 0, 0, 0)
                ON CONFLICT(account_id) DO NOTHING
                "#,
                params![account_id],
            )
            .map_err(|error| {
                AppError::internal(format!("failed to ensure sync context row: {error}"))
            })?;
        Ok(())
    }

    fn read_sync_context(
        connection: &Connection,
        account_id: &str,
    ) -> AppResult<Option<SyncContext>> {
        let row = connection
            .query_row(
                r#"
                SELECT
                    account_id,
                    base_url,
                    state,
                    ws_status,
                    last_revision_ms,
                    last_sync_at_ms,
                    last_error,
                    counts_folders,
                    counts_collections,
                    counts_policies,
                    counts_ciphers,
                    counts_sends
                FROM sync_contexts
                WHERE account_id = ?1
                "#,
                params![account_id],
                |row| {
                    let state_raw: String = row.get(2)?;
                    let ws_status_raw: String = row.get(3)?;
                    let counts_folders: i64 = row.get(7)?;
                    let counts_collections: i64 = row.get(8)?;
                    let counts_policies: i64 = row.get(9)?;
                    let counts_ciphers: i64 = row.get(10)?;
                    let counts_sends: i64 = row.get(11)?;

                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        state_raw,
                        ws_status_raw,
                        row.get::<_, Option<i64>>(4)?,
                        row.get::<_, Option<i64>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        counts_folders,
                        counts_collections,
                        counts_policies,
                        counts_ciphers,
                        counts_sends,
                    ))
                },
            )
            .optional()
            .map_err(|error| AppError::internal(format!("failed to read sync context: {error}")))?;

        match row {
            None => Ok(None),
            Some((
                account_id,
                base_url,
                state_raw,
                ws_status_raw,
                last_revision_ms,
                last_sync_at_ms,
                last_error,
                counts_folders,
                counts_collections,
                counts_policies,
                counts_ciphers,
                counts_sends,
            )) => {
                let state = parse_sync_state(&state_raw)?;
                let ws_status = parse_ws_status(&ws_status_raw)?;
                Ok(Some(SyncContext {
                    account_id,
                    base_url,
                    state,
                    ws_status,
                    last_revision_ms,
                    last_sync_at_ms,
                    last_error,
                    counts: SyncItemCounts {
                        folders: to_u32(counts_folders, "counts_folders")?,
                        collections: to_u32(counts_collections, "counts_collections")?,
                        policies: to_u32(counts_policies, "counts_policies")?,
                        ciphers: to_u32(counts_ciphers, "counts_ciphers")?,
                        sends: to_u32(counts_sends, "counts_sends")?,
                    },
                }))
            }
        }
    }

    fn begin_sql_transaction(connection: &mut Connection) -> AppResult<Transaction<'_>> {
        connection.transaction().map_err(|error| {
            AppError::internal(format!("failed to begin sqlite transaction: {error}"))
        })
    }

    fn ensure_active_transaction(tx: &Transaction<'_>, account_id: &str) -> AppResult<()> {
        let exists = tx
            .query_row(
                "SELECT 1 FROM sync_transactions WHERE account_id = ?1",
                params![account_id],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|error| {
                AppError::internal(format!("failed to query active sync transaction: {error}"))
            })?
            .is_some();

        if !exists {
            return Err(AppError::validation(format!(
                "no active sync transaction for account_id={account_id}"
            )));
        }

        Ok(())
    }

    fn clear_staging_snapshot(tx: &Transaction<'_>, account_id: &str) -> AppResult<()> {
        tx.execute(
            "DELETE FROM staging_profile WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear staging profile: {error}")))?;
        tx.execute(
            "DELETE FROM staging_folders WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear staging folders: {error}")))?;
        tx.execute(
            "DELETE FROM staging_collections WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to clear staging collections: {error}"))
        })?;
        tx.execute(
            "DELETE FROM staging_policies WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to clear staging policies: {error}"))
        })?;
        tx.execute(
            "DELETE FROM staging_ciphers WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear staging ciphers: {error}")))?;
        tx.execute(
            "DELETE FROM staging_sends WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear staging sends: {error}")))?;
        tx.execute(
            "DELETE FROM staging_meta WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear staging meta: {error}")))?;
        Ok(())
    }

    fn copy_live_to_staging(tx: &Transaction<'_>, account_id: &str) -> AppResult<()> {
        tx.execute(
            r#"
            INSERT INTO staging_profile (account_id, payload_json)
            SELECT account_id, payload_json
            FROM live_profile
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy live profile to staging: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO staging_folders (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM live_folders
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy live folders to staging: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO staging_collections (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM live_collections
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!(
                "failed to copy live collections to staging: {error}"
            ))
        })?;
        tx.execute(
            r#"
            INSERT INTO staging_policies (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM live_policies
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy live policies to staging: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO staging_ciphers (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM live_ciphers
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy live ciphers to staging: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO staging_sends (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM live_sends
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy live sends to staging: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO staging_meta (account_id, domains_json, user_decryption_json)
            SELECT account_id, domains_json, user_decryption_json
            FROM live_meta
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy live meta to staging: {error}"))
        })?;
        Ok(())
    }

    fn clear_live_snapshot(tx: &Transaction<'_>, account_id: &str) -> AppResult<()> {
        tx.execute(
            "DELETE FROM live_profile WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear live profile: {error}")))?;
        tx.execute(
            "DELETE FROM live_folders WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear live folders: {error}")))?;
        tx.execute(
            "DELETE FROM live_collections WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to clear live collections: {error}"))
        })?;
        tx.execute(
            "DELETE FROM live_policies WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear live policies: {error}")))?;
        tx.execute(
            "DELETE FROM live_ciphers WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear live ciphers: {error}")))?;
        tx.execute(
            "DELETE FROM live_sends WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear live sends: {error}")))?;
        tx.execute(
            "DELETE FROM live_meta WHERE account_id = ?1",
            params![account_id],
        )
        .map_err(|error| AppError::internal(format!("failed to clear live meta: {error}")))?;
        Ok(())
    }

    fn copy_staging_to_live(tx: &Transaction<'_>, account_id: &str) -> AppResult<()> {
        tx.execute(
            r#"
            INSERT INTO live_profile (account_id, payload_json)
            SELECT account_id, payload_json
            FROM staging_profile
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy staging profile to live: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO live_folders (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM staging_folders
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy staging folders to live: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO live_collections (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM staging_collections
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!(
                "failed to copy staging collections to live: {error}"
            ))
        })?;
        tx.execute(
            r#"
            INSERT INTO live_policies (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM staging_policies
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy staging policies to live: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO live_ciphers (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM staging_ciphers
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy staging ciphers to live: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO live_sends (account_id, id, payload_json)
            SELECT account_id, id, payload_json
            FROM staging_sends
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy staging sends to live: {error}"))
        })?;
        tx.execute(
            r#"
            INSERT INTO live_meta (account_id, domains_json, user_decryption_json)
            SELECT account_id, domains_json, user_decryption_json
            FROM staging_meta
            WHERE account_id = ?1
            "#,
            params![account_id],
        )
        .map_err(|error| {
            AppError::internal(format!("failed to copy staging meta to live: {error}"))
        })?;
        Ok(())
    }

    fn to_json<T: serde::Serialize>(value: &T, label: &str) -> AppResult<String> {
        serde_json::to_string(value)
            .map_err(|error| AppError::internal(format!("failed to serialize {label}: {error}")))
    }
}

#[async_trait]
impl VaultRepositoryPort for SqliteVaultRepository {
    async fn set_sync_running(&self, account_id: &str, base_url: &str) -> AppResult<SyncContext> {
        self.with_account_connection(account_id, |connection| {
            Self::ensure_context_row(connection, account_id)?;
            connection
                .execute(
                    r#"
                    UPDATE sync_contexts
                    SET
                        base_url = ?2,
                        state = 'running',
                        last_error = NULL
                    WHERE account_id = ?1
                    "#,
                    params![account_id, base_url],
                )
                .map_err(|error| {
                    AppError::internal(format!("failed to set sync running: {error}"))
                })?;

            Self::read_sync_context(connection, account_id)?.ok_or_else(|| {
                AppError::internal(format!(
                    "sync context missing after set_sync_running for account_id={account_id}"
                ))
            })
        })
    }

    async fn set_sync_succeeded(
        &self,
        account_id: &str,
        base_url: &str,
        revision_ms: Option<i64>,
        synced_at_ms: i64,
        counts: SyncItemCounts,
    ) -> AppResult<SyncContext> {
        self.with_account_connection(account_id, |connection| {
            Self::ensure_context_row(connection, account_id)?;
            connection
                .execute(
                    r#"
                    UPDATE sync_contexts
                    SET
                        base_url = ?2,
                        state = 'succeeded',
                        last_revision_ms = ?3,
                        last_sync_at_ms = ?4,
                        last_error = NULL,
                        counts_folders = ?5,
                        counts_collections = ?6,
                        counts_policies = ?7,
                        counts_ciphers = ?8,
                        counts_sends = ?9
                    WHERE account_id = ?1
                    "#,
                    params![
                        account_id,
                        base_url,
                        revision_ms,
                        synced_at_ms,
                        i64::from(counts.folders),
                        i64::from(counts.collections),
                        i64::from(counts.policies),
                        i64::from(counts.ciphers),
                        i64::from(counts.sends)
                    ],
                )
                .map_err(|error| {
                    AppError::internal(format!("failed to set sync succeeded: {error}"))
                })?;

            Self::read_sync_context(connection, account_id)?.ok_or_else(|| {
                AppError::internal(format!(
                    "sync context missing after set_sync_succeeded for account_id={account_id}"
                ))
            })
        })
    }

    async fn set_sync_failed(
        &self,
        account_id: &str,
        base_url: &str,
        error_message: String,
    ) -> AppResult<SyncContext> {
        self.with_account_connection(account_id, |connection| {
            Self::ensure_context_row(connection, account_id)?;
            connection
                .execute(
                    r#"
                    UPDATE sync_contexts
                    SET
                        base_url = ?2,
                        state = 'failed',
                        last_error = ?3
                    WHERE account_id = ?1
                    "#,
                    params![account_id, base_url, error_message],
                )
                .map_err(|error| {
                    AppError::internal(format!("failed to set sync failed: {error}"))
                })?;

            Self::read_sync_context(connection, account_id)?.ok_or_else(|| {
                AppError::internal(format!(
                    "sync context missing after set_sync_failed for account_id={account_id}"
                ))
            })
        })
    }

    async fn get_sync_context(&self, account_id: &str) -> AppResult<Option<SyncContext>> {
        self.with_account_connection(account_id, |connection| {
            Self::read_sync_context(connection, account_id)
        })
    }

    async fn begin_sync_transaction(&self, account_id: &str) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;

            let exists = tx
                .query_row(
                    "SELECT 1 FROM sync_transactions WHERE account_id = ?1",
                    params![account_id],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(|error| {
                    AppError::internal(format!(
                        "failed to check existing sync transaction: {error}"
                    ))
                })?
                .is_some();
            if exists {
                return Err(AppError::validation(format!(
                    "sync transaction already started for account_id={account_id}"
                )));
            }

            tx.execute(
                "INSERT INTO sync_transactions (account_id, started_at_ms) VALUES (?1, ?2)",
                params![account_id, now_unix_ms()?],
            )
            .map_err(|error| {
                AppError::internal(format!("failed to create sync transaction marker: {error}"))
            })?;

            Self::clear_staging_snapshot(&tx, account_id)?;
            Self::copy_live_to_staging(&tx, account_id)?;

            tx.commit().map_err(|error| {
                AppError::internal(format!("failed to commit begin sync transaction: {error}"))
            })?;
            Ok(())
        })
    }

    async fn commit_sync_transaction(&self, account_id: &str) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;

            Self::clear_live_snapshot(&tx, account_id)?;
            Self::copy_staging_to_live(&tx, account_id)?;
            Self::clear_staging_snapshot(&tx, account_id)?;
            tx.execute(
                "DELETE FROM sync_transactions WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(|error| {
                AppError::internal(format!("failed to clear sync transaction marker: {error}"))
            })?;

            tx.commit().map_err(|error| {
                AppError::internal(format!("failed to commit sync transaction: {error}"))
            })?;
            Ok(())
        })
    }

    async fn rollback_sync_transaction(&self, account_id: &str) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;
            Self::clear_staging_snapshot(&tx, account_id)?;
            tx.execute(
                "DELETE FROM sync_transactions WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(|error| {
                AppError::internal(format!("failed to clear sync transaction marker: {error}"))
            })?;
            tx.commit().map_err(|error| {
                AppError::internal(format!("failed to rollback sync transaction: {error}"))
            })?;
            Ok(())
        })
    }

    async fn upsert_profile(&self, account_id: &str, profile: SyncProfile) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;
            let payload_json = Self::to_json(&profile, "sync profile")?;
            tx.execute(
                r#"
                INSERT INTO staging_profile (account_id, payload_json)
                VALUES (?1, ?2)
                ON CONFLICT(account_id) DO UPDATE SET payload_json = excluded.payload_json
                "#,
                params![account_id, payload_json],
            )
            .map_err(|error| {
                AppError::internal(format!("failed to upsert staging profile: {error}"))
            })?;
            tx.commit().map_err(|error| {
                AppError::internal(format!(
                    "failed to commit upsert profile transaction: {error}"
                ))
            })?;
            Ok(())
        })
    }

    async fn upsert_folders(&self, account_id: &str, folders: Vec<SyncFolder>) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;
            {
                let mut statement = tx
                    .prepare(
                        r#"
                        INSERT INTO staging_folders (account_id, id, payload_json)
                        VALUES (?1, ?2, ?3)
                        ON CONFLICT(account_id, id) DO UPDATE SET payload_json = excluded.payload_json
                        "#,
                    )
                    .map_err(|error| {
                        AppError::internal(format!("failed to prepare folder upsert statement: {error}"))
                    })?;
                for folder in folders {
                    let payload_json = Self::to_json(&folder, "sync folder")?;
                    statement
                        .execute(params![account_id, folder.id, payload_json])
                        .map_err(|error| {
                            AppError::internal(format!("failed to upsert staging folder: {error}"))
                        })?;
                }
            }
            tx.commit().map_err(|error| {
                AppError::internal(format!("failed to commit upsert folders transaction: {error}"))
            })?;
            Ok(())
        })
    }

    async fn upsert_collections(
        &self,
        account_id: &str,
        collections: Vec<SyncCollection>,
    ) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;
            {
                let mut statement = tx
                    .prepare(
                        r#"
                        INSERT INTO staging_collections (account_id, id, payload_json)
                        VALUES (?1, ?2, ?3)
                        ON CONFLICT(account_id, id) DO UPDATE SET payload_json = excluded.payload_json
                        "#,
                    )
                    .map_err(|error| {
                        AppError::internal(format!(
                            "failed to prepare collection upsert statement: {error}"
                        ))
                    })?;
                for collection in collections {
                    let payload_json = Self::to_json(&collection, "sync collection")?;
                    statement
                        .execute(params![account_id, collection.id, payload_json])
                        .map_err(|error| {
                            AppError::internal(format!(
                                "failed to upsert staging collection: {error}"
                            ))
                        })?;
                }
            }
            tx.commit().map_err(|error| {
                AppError::internal(format!(
                    "failed to commit upsert collections transaction: {error}"
                ))
            })?;
            Ok(())
        })
    }

    async fn upsert_policies(&self, account_id: &str, policies: Vec<SyncPolicy>) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;
            {
                let mut statement = tx
                    .prepare(
                        r#"
                        INSERT INTO staging_policies (account_id, id, payload_json)
                        VALUES (?1, ?2, ?3)
                        ON CONFLICT(account_id, id) DO UPDATE SET payload_json = excluded.payload_json
                        "#,
                    )
                    .map_err(|error| {
                        AppError::internal(format!("failed to prepare policy upsert statement: {error}"))
                    })?;
                for policy in policies {
                    let payload_json = Self::to_json(&policy, "sync policy")?;
                    statement
                        .execute(params![account_id, policy.id, payload_json])
                        .map_err(|error| {
                            AppError::internal(format!("failed to upsert staging policy: {error}"))
                        })?;
                }
            }
            tx.commit().map_err(|error| {
                AppError::internal(format!("failed to commit upsert policies transaction: {error}"))
            })?;
            Ok(())
        })
    }

    async fn upsert_ciphers(&self, account_id: &str, ciphers: Vec<SyncCipher>) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;
            {
                let mut statement = tx
                    .prepare(
                        r#"
                        INSERT INTO staging_ciphers (account_id, id, payload_json)
                        VALUES (?1, ?2, ?3)
                        ON CONFLICT(account_id, id) DO UPDATE SET payload_json = excluded.payload_json
                        "#,
                    )
                    .map_err(|error| {
                        AppError::internal(format!("failed to prepare cipher upsert statement: {error}"))
                    })?;
                for cipher in ciphers {
                    let payload_json = Self::to_json(&cipher, "sync cipher")?;
                    statement
                        .execute(params![account_id, cipher.id, payload_json])
                        .map_err(|error| {
                            AppError::internal(format!("failed to upsert staging cipher: {error}"))
                        })?;
                }
            }
            tx.commit().map_err(|error| {
                AppError::internal(format!("failed to commit upsert ciphers transaction: {error}"))
            })?;
            Ok(())
        })
    }

    async fn upsert_sends(&self, account_id: &str, sends: Vec<SyncSend>) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;
            {
                let mut statement = tx
                    .prepare(
                        r#"
                        INSERT INTO staging_sends (account_id, id, payload_json)
                        VALUES (?1, ?2, ?3)
                        ON CONFLICT(account_id, id) DO UPDATE SET payload_json = excluded.payload_json
                        "#,
                    )
                    .map_err(|error| {
                        AppError::internal(format!("failed to prepare send upsert statement: {error}"))
                    })?;
                for send in sends {
                    let payload_json = Self::to_json(&send, "sync send")?;
                    statement
                        .execute(params![account_id, send.id, payload_json])
                        .map_err(|error| {
                            AppError::internal(format!("failed to upsert staging send: {error}"))
                        })?;
                }
            }
            tx.commit().map_err(|error| {
                AppError::internal(format!("failed to commit upsert sends transaction: {error}"))
            })?;
            Ok(())
        })
    }

    async fn upsert_domains(
        &self,
        account_id: &str,
        domains: Option<SyncDomains>,
    ) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;
            tx.execute(
                r#"
                INSERT INTO staging_meta (account_id, domains_json, user_decryption_json)
                VALUES (?1, NULL, NULL)
                ON CONFLICT(account_id) DO NOTHING
                "#,
                params![account_id],
            )
            .map_err(|error| {
                AppError::internal(format!(
                    "failed to ensure staging meta row for domains: {error}"
                ))
            })?;

            let domains_json = domains
                .as_ref()
                .map(|value| Self::to_json(value, "sync domains"))
                .transpose()?;
            tx.execute(
                "UPDATE staging_meta SET domains_json = ?2 WHERE account_id = ?1",
                params![account_id, domains_json],
            )
            .map_err(|error| {
                AppError::internal(format!("failed to upsert staging domains: {error}"))
            })?;

            tx.commit().map_err(|error| {
                AppError::internal(format!(
                    "failed to commit upsert domains transaction: {error}"
                ))
            })?;
            Ok(())
        })
    }

    async fn upsert_user_decryption(
        &self,
        account_id: &str,
        user_decryption: Option<SyncUserDecryption>,
    ) -> AppResult<()> {
        self.with_account_connection(account_id, |connection| {
            let tx = Self::begin_sql_transaction(connection)?;
            Self::ensure_active_transaction(&tx, account_id)?;
            tx.execute(
                r#"
                INSERT INTO staging_meta (account_id, domains_json, user_decryption_json)
                VALUES (?1, NULL, NULL)
                ON CONFLICT(account_id) DO NOTHING
                "#,
                params![account_id],
            )
            .map_err(|error| {
                AppError::internal(format!(
                    "failed to ensure staging meta row for user decryption: {error}"
                ))
            })?;

            let user_decryption_json = user_decryption
                .as_ref()
                .map(|value| Self::to_json(value, "sync user decryption"))
                .transpose()?;
            tx.execute(
                "UPDATE staging_meta SET user_decryption_json = ?2 WHERE account_id = ?1",
                params![account_id, user_decryption_json],
            )
            .map_err(|error| {
                AppError::internal(format!("failed to upsert staging user decryption: {error}"))
            })?;

            tx.commit().map_err(|error| {
                AppError::internal(format!(
                    "failed to commit upsert user decryption transaction: {error}"
                ))
            })?;
            Ok(())
        })
    }
}

fn parse_sync_state(value: &str) -> AppResult<SyncState> {
    match value {
        "idle" => Ok(SyncState::Idle),
        "running" => Ok(SyncState::Running),
        "succeeded" => Ok(SyncState::Succeeded),
        "failed" => Ok(SyncState::Failed),
        _ => Err(AppError::internal(format!(
            "invalid sync state value in database: {value}"
        ))),
    }
}

fn parse_ws_status(value: &str) -> AppResult<WsStatus> {
    match value {
        "unknown" => Ok(WsStatus::Unknown),
        "connected" => Ok(WsStatus::Connected),
        "disconnected" => Ok(WsStatus::Disconnected),
        _ => Err(AppError::internal(format!(
            "invalid ws status value in database: {value}"
        ))),
    }
}

fn to_u32(value: i64, field: &str) -> AppResult<u32> {
    if value < 0 || value > i64::from(u32::MAX) {
        return Err(AppError::internal(format!(
            "invalid non-u32 value for {field}: {value}"
        )));
    }
    Ok(value as u32)
}

fn now_unix_ms() -> AppResult<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| AppError::internal(format!("system clock before unix epoch: {error}")))?;
    Ok(duration.as_millis().min(i64::MAX as u128) as i64)
}
