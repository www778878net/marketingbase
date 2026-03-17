use serde_json::Value;
use std::collections::HashMap;

/// 步骤上下文
pub struct StepContext {
    /// 账号
    pub account: String,
    /// 平台
    pub platform: String,
    /// 当前子步骤
    pub sub_step: String,
    /// 端口号
    pub port: Option<u16>,
    /// 共享数据
    pub data: HashMap<String, Value>,
}

impl StepContext {
    pub fn new(account: String, platform: String) -> Self {
        Self {
            account,
            platform,
            sub_step: String::new(),
            port: None,
            data: HashMap::new(),
        }
    }

    pub fn change_sub_step(&mut self, step: &str) {
        self.sub_step = step.to_string();
    }

    pub fn clear_sub_step(&mut self) {
        self.sub_step = String::new();
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = Some(port);
    }
}
