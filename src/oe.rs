// ==========================================
// Nintendo Switch Horizon OS (nnSdk / libnx)
// ==========================================

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OperationMode {
    Handheld = 0, // 掌机模式 (720p 布局)
    Docked = 1,   // 主机/底座模式 (1080p 布局)
}

unsafe extern "C" {
    /// Horizon OS 原生 SDK 接口 (MHGU 主进程静态链接)
    /// Mangled C++ Symbol: nn::oe::GetOperationMode(void)
    #[link_name = "_ZN2nn2oe16GetOperationModeEv"]
    pub fn nn_oe_GetOperationMode() -> u8;
}

/// 通过 Switch 原生 SDK 获取当前主机/掌机运行模式
#[inline]
pub fn get_operation_mode() -> OperationMode {
    unsafe {
        match nn_oe_GetOperationMode() {
            0 => OperationMode::Handheld,
            _ => OperationMode::Docked,
        }
    }
}

/// 是否处于主机/底座模式
#[inline]
pub fn is_docked() -> bool {
    get_operation_mode() == OperationMode::Docked
}

/// 计算当前模式相对于 1080p 基准配置的缩放系数：
/// - 主机模式 (1080p): scale = 1.0
/// - 掌机模式 (720p):  scale = 720.0 / 1080.0 (~0.66667)
#[inline]
pub fn get_mode_scale() -> f32 {
    match get_operation_mode() {
        OperationMode::Docked => 1.0,
        OperationMode::Handheld => 720.0 / 1080.0,
    }
}
