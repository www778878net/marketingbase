//! 已发送记录表 DataService
//!
//! 用于记录已发送的营销内容，按url去重

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use database::{DataAudit, DataState, LocalDB, TableConfig};
use base::upinfo::UpInfo;

/// marketing_sent 表建表 SQL (SQLite版本)
pub const SENT_CREATE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS marketing_sent (
    id TEXT NOT NULL PRIMARY KEY,
    cid TEXT NOT NULL DEFAULT '',
    platform TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL DEFAULT '',
    url TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    keyword TEXT NOT NULL DEFAULT '',
    senttime TEXT NOT NULL DEFAULT '',
    uptime TEXT NOT NULL DEFAULT '',
    upby TEXT NOT NULL DEFAULT ''
)
"#;

/// 已发送记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentRecord {
    pub id: String,
    pub cid: String,
    pub platform: String,
    pub title: String,
    pub url: String,
    pub content: String,
    pub keyword: String,
    pub senttime: String,
    pub uptime: String,
    pub upby: String,
}

/// Sent - 已发送记录表 DataService
pub struct Sent {
    pub db: LocalDB,
    pub audit: DataAudit,
    pub state: DataState,
}

impl Sent {
    pub fn new() -> Self {
        let db = LocalDB::new(None, None).expect("创建数据库失败");
        let audit = DataAudit::new("marketing_sent");

        db.ensure_table("marketing_sent", SENT_CREATE_SQL).expect("建表失败");

        let config = TableConfig {
            name: "marketing_sent".to_string(),
            ..Default::default()
        };
        let state = DataState::from_config(&config);

        Self { db, audit, state }
    }

    fn check_caller(&self, operation: &str, caller: &str) -> Result<(), String> {
        let allowed = match caller {
            "marketing_sent" => true,
            "marketing" => true,
            "marketingbase" => true,
            "zhihu" => true,
            "xiaohongshu" => true,
            _ => false,
        };

        if !allowed {
            return Err(format!("{} 无权调用 {}", caller, operation));
        }
        Ok(())
    }

    pub fn m_save(&self, record: &mut HashMap<String, Value>, caller: &str, summary: &str) -> Result<String, String> {
        self.check_caller("m_save", caller)?;
        
        let now = chrono::Utc::now().to_rfc3339();
        let id = UpInfo::new_id();
        
        record.insert("id".to_string(), Value::String(id.clone()));
        record.insert("uptime".to_string(), Value::String(now));
        
        self.state.m_save(record, caller, summary)
    }

    pub fn m_update(&self, id: &str, record: &mut HashMap<String, Value>, caller: &str, summary: &str) -> Result<bool, String> {
        self.check_caller("m_update", caller)?;
        
        let now = chrono::Utc::now().to_rfc3339();
        record.insert("uptime".to_string(), Value::String(now));
        
        self.state.m_update(id, record, caller, summary)
    }

    pub fn m_del(&self, id: &str, caller: &str, summary: &str) -> Result<bool, String> {
        self.check_caller("m_del", caller)?;
        self.state.m_del(id, caller, summary)
    }

    pub fn getone(&self, id: &str, caller: &str, summary: &str) -> Result<Option<HashMap<String, Value>>, String> {
        self.check_caller("getone", caller)?;
        self.state.get_one(id, caller, summary)
    }

    pub fn mlist(&self, caller: &str, limit: i32, summary: &str) -> Result<Vec<HashMap<String, Value>>, String> {
        self.check_caller("mlist", caller)?;
        let sql = format!("SELECT * FROM marketing_sent ORDER BY senttime DESC LIMIT {}", limit);
        self.state.do_get(&sql, &[], caller, summary)
    }

    /// 检查指定url是否已发送（去重）
    pub fn is_sent_by_url(&self, url: &str) -> Result<bool, String> {
        let sql = format!(
            "SELECT COUNT(*) as cnt FROM marketing_sent WHERE url = '{}'",
            url.replace("'", "''")
        );
        let results = self.db.query(&sql, &[])?;
        
        if let Some(row) = results.first() {
            if let Some(cnt_val) = row.get("cnt") {
                let cnt: i64 = serde_json::from_value(cnt_val.clone()).unwrap_or(0);
                return Ok(cnt > 0);
            }
        }
        Ok(false)
    }

    /// 获取今天的已发送记录
    pub fn get_today_sent(&self, platform: &str) -> Result<Vec<HashMap<String, Value>>, String> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let sql = format!(
            "SELECT * FROM marketing_sent WHERE platform = '{}' AND senttime LIKE '{}%' ORDER BY senttime DESC",
            platform.replace("'", "''"),
            today
        );
        self.db.query(&sql, &[])
    }

    /// 记录已发送
    pub fn record_sent(&self, platform: &str, title: &str, url: &str, content: &str, keyword: &str, caller: &str) -> Result<String, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut record = HashMap::new();
        record.insert("platform".to_string(), Value::String(platform.to_string()));
        record.insert("title".to_string(), Value::String(title.to_string()));
        record.insert("url".to_string(), Value::String(url.to_string()));
        record.insert("content".to_string(), Value::String(content.to_string()));
        record.insert("keyword".to_string(), Value::String(keyword.to_string()));
        record.insert("senttime".to_string(), Value::String(now));
        
        self.m_save(&mut record, caller, &format!("记录{}已发送: {}", platform, title))
    }
}

impl Default for Sent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sent_create() {
        let sent = Sent::new();
        assert!(sent.db.table_exists("marketing_sent").unwrap_or(false));
    }

    #[test]
    fn test_record_and_check() {
        let sent = Sent::new();
        
        // 记录一条已发送
        let id = sent.record_sent("zhihu", "测试标题", "https://test.com", "测试内容", "测试关键词", "marketing").expect("记录应该成功");
        assert!(!id.is_empty(), "ID不应为空");

        // 检查是否已发送（按URL）
        let is_sent = sent.is_sent_by_url("https://test.com").expect("查询应该成功");
        assert!(is_sent, "应该标记为已发送");

        // 检查未发送的URL
        let is_sent2 = sent.is_sent_by_url("https://not-sent.com").expect("查询应该成功");
        assert!(!is_sent2, "应该标记为未发送");

        // 清理
        let _ = sent.m_del(&id, "marketing", "单元测试清理");
    }

    #[test]
    fn test_get_today_sent() {
        let sent = Sent::new();
        
        // 记录一条
        let _ = sent.record_sent("xiaohongshu", "今日测试", "https://test.com", "内容", "关键词", "marketing");

        // 获取今天的
        let today_sent = sent.get_today_sent("xiaohongshu").expect("查询应该成功");
        assert!(!today_sent.is_empty(), "今天应该有记录");
    }
}
