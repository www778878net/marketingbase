//! Marketing Base - 步骤 trait 和基础类型
//!
//! 提供所有平台共用的 Step trait 和相关类型

mod result;
mod context;
mod step;
mod task_runner;

pub use result::StepResult;
pub use context::StepContext;
pub use step::Step;
pub use task_runner::TaskRunner;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_context() {
        let ctx = StepContext::new("test_account".to_string(), "zhihu".to_string());
        assert_eq!(ctx.account, "test_account");
        assert_eq!(ctx.platform, "zhihu");
    }
}
