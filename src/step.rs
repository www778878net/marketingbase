use async_trait::async_trait;
use chromiumoxide::Browser;

use crate::{StepContext, StepResult};

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
