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
