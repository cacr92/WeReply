use crate::secret::ApiKeyManager;
use crate::types::{ChatKind, ChatSummary};
use crate::ui_automation::IncomingMessage;
use anyhow::{anyhow, Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const WECHAT_CONTAINER_DIR: &str =
    "Library/Containers/com.tencent.xinWeChat/Data/Documents/xwechat_files";

#[derive(Debug, Default, Clone)]
struct DbCursor {
    last_timestamp: Option<i64>,
    last_msg_id: Option<i64>,
}

pub struct MacosDb {
    session_db: PathBuf,
    message_dbs: Vec<PathBuf>,
    key_info_db: PathBuf,
    key: Mutex<Option<Vec<u8>>>,
    cursor: Mutex<DbCursor>,
}

impl MacosDb {
    pub fn discover() -> Result<Self> {
        let root = wechat_data_root().context("WeChat 数据目录不存在")?;
        let user_root = resolve_latest_user_root(&root).context("未找到 WeChat 用户目录")?;
        let key_info_db = resolve_key_info_db(&root, &user_root)?;
        let session_db = user_root.join("db_storage/session/session.db");
        let message_dbs = resolve_message_dbs(&user_root)?;
        Ok(Self {
            session_db,
            message_dbs,
            key_info_db,
            key: Mutex::new(None),
            cursor: Mutex::new(DbCursor::default()),
        })
    }

    pub fn list_recent_chats(&self) -> Result<Vec<ChatSummary>> {
        let key = self.ensure_db_key()?;
        let conn = open_sqlcipher_readonly(&self.session_db, &key)?;
        let (table, chat_col, title_col) = locate_session_table(&conn)?;
        let sql = format!(
            "SELECT {chat_col}, {title_col} FROM {table} ORDER BY rowid DESC LIMIT 200"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| {
            let chat_id: String = row.get(0)?;
            let chat_title: String = row.get(1)?;
            Ok(ChatSummary {
                chat_id,
                chat_title,
                kind: ChatKind::Unknown,
            })
        })?;
        let mut seen = HashSet::new();
        let mut chats = Vec::new();
        for item in rows {
            let chat = item?;
            if chat.chat_id.trim().is_empty() {
                continue;
            }
            if !seen.insert(chat.chat_id.clone()) {
                continue;
            }
            chats.push(chat);
        }
        if chats.is_empty() {
            return Err(anyhow!("会话列表为空"));
        }
        Ok(chats)
    }

    pub fn poll_latest_message(&self) -> Result<Option<IncomingMessage>> {
        let key = self.ensure_db_key()?;
        let message_db = self
            .message_dbs
            .iter()
            .find(|path| path.exists())
            .cloned()
            .ok_or_else(|| anyhow!("消息数据库不存在"))?;
        let conn = open_sqlcipher_readonly(&message_db, &key)?;
        let table = locate_message_table(&conn)?;
        let columns = load_table_columns(&conn, &table)?;
        let chat_col = pick_column(&columns, &CHAT_ID_COLUMNS).ok_or_else(|| anyhow!("chat 列缺失"))?;
        let text_col = pick_column(&columns, &TEXT_COLUMNS).ok_or_else(|| anyhow!("text 列缺失"))?;
        let time_col = pick_column(&columns, &TIME_COLUMNS);
        let id_col = pick_column(&columns, &ID_COLUMNS);
        let has_time = time_col.is_some();
        let has_id = id_col.is_some();
        let mut cursor = self.cursor.lock().map_err(|_| anyhow!("cursor lock poisoned"))?;
        let (sql, args): (String, Vec<i64>) = if let Some(time_col) = time_col.clone() {
            let last_time = cursor.last_timestamp.unwrap_or(0);
            let last_id = cursor.last_msg_id.unwrap_or(0);
            if let Some(id_col) = id_col.clone() {
                (
                    format!(
                        "SELECT {chat_col}, {text_col}, {time_col}, {id_col} FROM {table} \
                         WHERE {time_col} > ? OR ({time_col} = ? AND {id_col} > ?) \
                         ORDER BY {time_col} DESC, {id_col} DESC LIMIT 1"
                    ),
                    vec![last_time, last_time, last_id],
                )
            } else {
                (
                    format!(
                        "SELECT {chat_col}, {text_col}, {time_col} FROM {table} \
                         WHERE {time_col} > ? ORDER BY {time_col} DESC LIMIT 1"
                    ),
                    vec![last_time],
                )
            }
        } else {
            (
                format!(
                    "SELECT {chat_col}, {text_col}, rowid FROM {table} \
                     WHERE rowid > ? ORDER BY rowid DESC LIMIT 1"
                ),
                vec![cursor.last_msg_id.unwrap_or(0)],
            )
        };
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(args))?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let chat_id: String = row.get(0)?;
        let text: String = row.get(1)?;
        let (timestamp, msg_id) = if has_time {
            let time_val: i64 = row.get(2)?;
            let msg_id_val = if has_id { row.get(3)? } else { 0 };
            (time_val, msg_id_val)
        } else {
            let rowid_val: i64 = row.get(2)?;
            (rowid_val, rowid_val)
        };
        cursor.last_timestamp = Some(timestamp);
        cursor.last_msg_id = Some(msg_id);
        Ok(Some(IncomingMessage {
            chat_id,
            text,
            timestamp: timestamp.max(0) as u64,
            msg_id: Some(msg_id.to_string()),
        }))
    }

    fn ensure_db_key(&self) -> Result<Vec<u8>> {
        if let Some(key) = self.key.lock().map_err(|_| anyhow!("key lock poisoned"))?.clone() {
            if can_open_db(&self.session_db, &key) {
                return Ok(key);
            }
        }
        if let Ok(encoded) = ApiKeyManager::get_wechat_db_key() {
            if let Ok(key) = decode_hex(&encoded) {
                if can_open_db(&self.session_db, &key) {
                    *self.key.lock().map_err(|_| anyhow!("key lock poisoned"))? = Some(key.clone());
                    return Ok(key);
                }
            }
        }
        let candidates = extract_key_candidates_from_db(&self.key_info_db)?;
        for candidate in candidates {
            if can_open_db(&self.session_db, &candidate) {
                let encoded = encode_hex(&candidate);
                let _ = ApiKeyManager::set_wechat_db_key(&encoded);
                *self.key.lock().map_err(|_| anyhow!("key lock poisoned"))? = Some(candidate.clone());
                return Ok(candidate);
            }
        }
        Err(anyhow!("无法解析 WeChat 数据库密钥"))
    }

    #[cfg(test)]
    pub fn for_tests(session_db: PathBuf, message_dbs: Vec<PathBuf>, key: Vec<u8>) -> Self {
        Self {
            session_db,
            message_dbs,
            key_info_db: PathBuf::new(),
            key: Mutex::new(Some(key)),
            cursor: Mutex::new(DbCursor::default()),
        }
    }
}

fn wechat_data_root() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(WECHAT_CONTAINER_DIR))
}

fn resolve_latest_user_root(root: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    let entries = fs::read_dir(root).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name()?.to_string_lossy();
        if !name.starts_with("wxid_") {
            continue;
        }
        let meta = entry.metadata().ok()?;
        let modified = meta.modified().ok()?;
        candidates.push((modified, path));
    }
    candidates.sort_by_key(|(modified, _)| *modified);
    candidates.pop().map(|(_, path)| path)
}

fn resolve_key_info_db(root: &Path, user_root: &Path) -> Result<PathBuf> {
    let wxid = user_root
        .file_name()
        .ok_or_else(|| anyhow!("wxid 不存在"))?
        .to_string_lossy()
        .to_string();
    let path = root
        .join("all_users/login")
        .join(wxid)
        .join("key_info.db");
    if !path.exists() {
        return Err(anyhow!("key_info.db 不存在"));
    }
    Ok(path)
}

fn resolve_message_dbs(user_root: &Path) -> Result<Vec<PathBuf>> {
    let base = user_root.join("db_storage/message");
    let mut dbs = Vec::new();
    let entries = fs::read_dir(base).context("message db 目录不存在")?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !name.starts_with("message_") || !name.ends_with(".db") {
            continue;
        }
        dbs.push(path);
    }
    dbs.sort();
    if dbs.is_empty() {
        return Err(anyhow!("未找到 message db"));
    }
    Ok(dbs)
}

fn open_sqlcipher_readonly(path: &Path, key: &[u8]) -> Result<Connection> {
    let params = [
        SqlcipherParams::new(4, None, None),
        SqlcipherParams::new(4, Some(64000), Some(4096)),
        SqlcipherParams::new(3, Some(64000), Some(1024)),
    ];
    for params in params {
        if let Ok(conn) = try_open_with_params(path, key, &params) {
            return Ok(conn);
        }
    }
    Err(anyhow!("无法解密数据库"))
}

fn can_open_db(path: &Path, key: &[u8]) -> bool {
    open_sqlcipher_readonly(path, key).is_ok()
}

fn try_open_with_params(path: &Path, key: &[u8], params: &SqlcipherParams) -> Result<Connection> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY;
    let conn = Connection::open_with_flags(path, flags)?;
    apply_sqlcipher_key(&conn, key, Some(params))?;
    let _: i64 = conn.query_row("SELECT count(*) FROM sqlite_master;", [], |row| row.get(0))?;
    Ok(conn)
}

struct SqlcipherParams {
    compat: i32,
    kdf_iter: Option<i32>,
    page_size: Option<i32>,
}

impl SqlcipherParams {
    fn new(compat: i32, kdf_iter: Option<i32>, page_size: Option<i32>) -> Self {
        Self {
            compat,
            kdf_iter,
            page_size,
        }
    }
}

fn apply_sqlcipher_key(conn: &Connection, key: &[u8], params: Option<&SqlcipherParams>) -> Result<()> {
    let hex = encode_hex(key);
    if let Some(params) = params {
        let mut pragma = format!(
            "PRAGMA cipher_compatibility = {}; PRAGMA key = \"x'{}'\";",
            params.compat, hex
        );
        if let Some(kdf) = params.kdf_iter {
            pragma.push_str(&format!(" PRAGMA kdf_iter = {};", kdf));
        }
        if let Some(page) = params.page_size {
            pragma.push_str(&format!(" PRAGMA cipher_page_size = {};", page));
        }
        conn.execute_batch(&pragma)?;
    } else {
        conn.execute_batch(&format!(
            "PRAGMA cipher_compatibility = 4; PRAGMA key = \"x'{}'\";",
            hex
        ))?;
    }
    Ok(())
}

fn extract_key_candidates_from_db(path: &Path) -> Result<Vec<Vec<u8>>> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut stmt = conn.prepare("SELECT key_info_data FROM LoginKeyInfoTable")?;
    let rows = stmt.query_map([], |row| row.get::<_, Vec<u8>>(0))?;
    let mut all = Vec::new();
    for item in rows {
        all.extend(extract_key_candidates(&item?));
    }
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for key in all {
        let hex = encode_hex(&key);
        if seen.insert(hex) {
            deduped.push(key);
        }
    }
    Ok(deduped)
}

fn extract_key_candidates(blob: &[u8]) -> Vec<Vec<u8>> {
    let mut candidates = Vec::new();
    candidates.extend(extract_windows(blob, 32, 12));
    candidates.extend(extract_windows(blob, 16, 6));
    candidates
}

fn extract_windows(blob: &[u8], size: usize, entropy_threshold: usize) -> Vec<Vec<u8>> {
    if blob.len() < size {
        return Vec::new();
    }
    let mut out = Vec::new();
    for start in 0..=blob.len() - size {
        let window = &blob[start..start + size];
        if unique_bytes(window) < entropy_threshold {
            continue;
        }
        out.push(window.to_vec());
    }
    out
}

fn unique_bytes(data: &[u8]) -> usize {
    let mut set = HashSet::new();
    for b in data {
        set.insert(*b);
    }
    set.len()
}

fn load_table_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    let mut cols = Vec::new();
    for col in rows {
        cols.push(col?);
    }
    Ok(cols)
}

fn locate_session_table(conn: &Connection) -> Result<(String, String, String)> {
    let tables = list_tables(conn)?;
    let mut best: Option<(i32, String, String, String)> = None;
    for table in tables {
        let columns = load_table_columns(conn, &table)?;
        let chat_col = pick_column(&columns, &CHAT_ID_COLUMNS);
        let title_col = pick_column(&columns, &TITLE_COLUMNS);
        if chat_col.is_none() || title_col.is_none() {
            continue;
        }
        let mut score = 0;
        if table.to_lowercase().contains("session") {
            score += 2;
        }
        if table.to_lowercase().contains("chat") {
            score += 1;
        }
        score += chat_col.as_ref().map(|_| 2).unwrap_or(0);
        score += title_col.as_ref().map(|_| 1).unwrap_or(0);
        if best.as_ref().map(|item| item.0).unwrap_or(-1) < score {
            best = Some((
                score,
                table.clone(),
                chat_col.unwrap(),
                title_col.unwrap(),
            ));
        }
    }
    best.map(|item| (item.1, item.2, item.3))
        .ok_or_else(|| anyhow!("未找到 session 表"))
}

fn locate_message_table(conn: &Connection) -> Result<String> {
    let tables = list_tables(conn)?;
    let mut best: Option<(i32, String)> = None;
    for table in tables {
        let columns = load_table_columns(conn, &table)?;
        let chat_col = pick_column(&columns, &CHAT_ID_COLUMNS);
        let text_col = pick_column(&columns, &TEXT_COLUMNS);
        if chat_col.is_none() || text_col.is_none() {
            continue;
        }
        let mut score = 0;
        if table.to_lowercase().contains("message") {
            score += 3;
        }
        if table.to_lowercase().contains("msg") {
            score += 1;
        }
        score += chat_col.as_ref().map(|_| 2).unwrap_or(0);
        score += text_col.as_ref().map(|_| 1).unwrap_or(0);
        if best.as_ref().map(|item| item.0).unwrap_or(-1) < score {
            best = Some((score, table.clone()));
        }
    }
    best.map(|item| item.1)
        .ok_or_else(|| anyhow!("未找到 message 表"))
}

fn list_tables(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    let mut names = Vec::new();
    for name in rows {
        names.push(name?);
    }
    Ok(names)
}

fn pick_column(columns: &[String], candidates: &[&str]) -> Option<String> {
    for &candidate in candidates {
        if let Some(found) = columns
            .iter()
            .find(|col| col.eq_ignore_ascii_case(candidate))
        {
            return Some(found.clone());
        }
    }
    None
}

const CHAT_ID_COLUMNS: [&str; 9] = [
    "chat_id",
    "session_id",
    "talker",
    "username",
    "user_name",
    "user",
    "chatid",
    "conversation_id",
    "usrname",
];

const TITLE_COLUMNS: [&str; 6] = [
    "chat_title",
    "title",
    "name",
    "nick",
    "nickname",
    "display_name",
];

const TEXT_COLUMNS: [&str; 6] = [
    "content",
    "text",
    "message",
    "msg",
    "strcontent",
    "body",
];

const TIME_COLUMNS: [&str; 8] = [
    "create_time",
    "createtime",
    "timestamp",
    "msg_time",
    "msgcreatetime",
    "time",
    "msgtime",
    "createTime",
];

const ID_COLUMNS: [&str; 7] = [
    "msg_id",
    "id",
    "local_id",
    "msgid",
    "server_id",
    "msgsvrid",
    "meslocalid",
];

fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

fn decode_hex(input: &str) -> Result<Vec<u8>> {
    let input = input.trim();
    if !input.len().is_multiple_of(2) {
        return Err(anyhow!("hex 长度非法"));
    }
    let mut out = Vec::with_capacity(input.len() / 2);
    let bytes = input.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = (bytes[i] as char).to_digit(16).ok_or_else(|| anyhow!("hex 非法"))?;
        let lo = (bytes[i + 1] as char).to_digit(16).ok_or_else(|| anyhow!("hex 非法"))?;
        out.push(((hi << 4) + lo) as u8);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_sqlcipher_db(path: &Path, key: &[u8], setup_sql: &str) -> Result<()> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        let hex = encode_hex(key);
        conn.execute_batch(&format!(
            "PRAGMA cipher_compatibility = 4; PRAGMA key = \"x'{}'\";",
            hex
        ))?;
        conn.execute_batch(setup_sql)?;
        Ok(())
    }

    #[test]
    fn extract_key_candidates_includes_known_key() {
        let key: Vec<u8> = (0u8..32).collect();
        let mut blob = vec![0u8; 64];
        blob[16..48].copy_from_slice(&key);
        let candidates = extract_key_candidates(&blob);
        assert!(candidates.iter().any(|item| item == &key));
    }

    #[test]
    fn opens_sqlcipher_db_with_key() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("session.db");
        let key = vec![0x11; 32];
        create_sqlcipher_db(
            &db_path,
            &key,
            "CREATE TABLE session (chat_id TEXT, chat_title TEXT);",
        )
        .unwrap();
        assert!(can_open_db(&db_path, &key));
    }

    #[test]
    fn list_recent_chats_from_session_db() {
        let dir = tempdir().unwrap();
        let session_db = dir.path().join("session.db");
        let message_db = dir.path().join("message_0.db");
        let key = vec![0x22; 32];
        create_sqlcipher_db(
            &session_db,
            &key,
            "CREATE TABLE session (chat_id TEXT, chat_title TEXT);\n\
             INSERT INTO session (chat_id, chat_title) VALUES ('c1', 'Chat 1'), ('c2', 'Chat 2');",
        )
        .unwrap();
        create_sqlcipher_db(
            &message_db,
            &key,
            "CREATE TABLE message (chat_id TEXT, content TEXT, create_time INTEGER, msg_id INTEGER);",
        )
        .unwrap();
        let db = MacosDb::for_tests(session_db, vec![message_db], key);
        let chats = db.list_recent_chats().unwrap();
        assert_eq!(chats.len(), 2);
    }

    #[test]
    fn poll_latest_message_returns_latest() {
        let dir = tempdir().unwrap();
        let session_db = dir.path().join("session.db");
        let message_db = dir.path().join("message_0.db");
        let key = vec![0x33; 32];
        create_sqlcipher_db(
            &session_db,
            &key,
            "CREATE TABLE session (chat_id TEXT, chat_title TEXT);\n\
             INSERT INTO session (chat_id, chat_title) VALUES ('c1', 'Chat 1');",
        )
        .unwrap();
        create_sqlcipher_db(
            &message_db,
            &key,
            "CREATE TABLE message (chat_id TEXT, content TEXT, create_time INTEGER, msg_id INTEGER);\n\
             INSERT INTO message (chat_id, content, create_time, msg_id) VALUES ('c1', 'hi', 10, 1);\n\
             INSERT INTO message (chat_id, content, create_time, msg_id) VALUES ('c1', 'latest', 20, 2);",
        )
        .unwrap();
        let db = MacosDb::for_tests(session_db, vec![message_db], key);
        let message = db.poll_latest_message().unwrap().expect("message");
        assert_eq!(message.text, "latest");
        let none = db.poll_latest_message().unwrap();
        assert!(none.is_none());
    }
}
