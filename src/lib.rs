//! Marketing Base - 步骤 trait 和基础类型
//!
//! 提供所有平台共用的 Step trait 和相关类型

use async_trait::async_trait;
use chromiumoxide::Browser;
use serde_json::Value;
use std::collections::HashMap;

/// 步骤执行结果
#[derive(Debug, Clone)]
pub enum StepResult {
    /// 继续当前步骤
    Continue,
    /// 切换到下一步骤
    NextStep(String),
    /// 任务结束
    Over,
    /// 错误
    Error(String),
    /// 设备丢失
    DeviceLost(String),
}

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

/// 步骤 Trait
///
/// 所有平台步骤都需要实现这个 trait
#[async_trait]
pub trait Step: Send {
    /// 步骤名称
    fn name(&self) -> &str;
    
    /// 执行步骤
    async fn execute(
        &mut self,
        browser: &Browser,
        context: &mut StepContext,
    ) -> StepResult;
}

/// TaskRunner - 任务运行器
///
/// 管理步骤注册和执行流程
pub struct TaskRunner {
    /// 步骤注册表
    steps: std::collections::HashMap<String, Box<dyn Step>>,
    /// 上下文
    pub context: StepContext,
}

impl TaskRunner {
    pub fn new(account: String, platform: String) -> Self {
        Self {
            steps: std::collections::HashMap::new(),
            context: StepContext::new(account, platform),
        }
    }

    /// 注册步骤
    pub fn register<S: Step + 'static>(&mut self, step: S) {
        self.steps.insert(step.name().to_string(), Box::new(step));
    }

    /// 运行任务
    pub async fn run(&mut self, browser: &Browser, start_step: &str) -> StepResult {
        let mut current_step = start_step.to_string();
        
        loop {
            let step = match self.steps.get_mut(&current_step) {
                Some(s) => s,
                None => return StepResult::Error(format!("步骤不存在: {}", current_step)),
            };

            let result = step.execute(browser, &mut self.context).await;

            match result {
                StepResult::Continue => continue,
                StepResult::NextStep(name) => {
                    current_step = name;
                }
                StepResult::Over => break,
                StepResult::Error(e) => {
                    return StepResult::Error(e);
                }
                StepResult::DeviceLost(e) => {
                    return StepResult::DeviceLost(e);
                }
            }
        }

        StepResult::Over
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStep {
        name: String,
    }

    #[async_trait]
    impl Step for TestStep {
        fn name(&self) -> &str {
            &self.name
        }

        async fn execute(&mut self, _browser: &Browser, _context: &mut StepContext) -> StepResult {
            StepResult::Over
        }
    }

    #[test]
    fn test_step_context() {
        let ctx = StepContext::new("test_account".to_string(), "zhihu".to_string());
        assert_eq!(ctx.account, "test_account");
        assert_eq!(ctx.platform, "zhihu");
    }
}
