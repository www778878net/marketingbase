use chromiumoxide::Browser;
use std::collections::HashMap;

use crate::{Step, StepContext, StepResult};

/// TaskRunner - 任务运行器
///
/// 管理步骤注册和执行流程
pub struct TaskRunner {
    /// 步骤注册表
    steps: HashMap<String, Box<dyn Step>>,
    /// 上下文
    pub context: StepContext,
}

impl TaskRunner {
    pub fn new(account: String, platform: String) -> Self {
        Self {
            steps: HashMap::new(),
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
