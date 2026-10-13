---
name: mhgu-forge-mod-dev
description: >-
  怪物猎人GU (MHGU / MHXX Nintendo Switch) 基于 forge / mhgu-forge 原生插件 (NRO / cdylib) 开发指南与全流程规范。
  适用于开发怪物显血、数据覆盖、功能Hook、Dear ImGui HUD 界面定制、内存特征扫描等插件。
---

# 怪物猎人GU (MHGU) Forge 插件开发指南

本 Skill 沉淀了使用 Rust 和 `mhgu-forge` 开发 Nintendo Switch《怪物猎人GU》原生插件的全流程规范、内存结构模型、渲染集成及防崩注意事项。

---

## 一、 开发环境与工具链配置

### 1. 基础依赖
- **Rust Toolchain**: `nightly` 通道（需包含 `rust-src` 组件）。
  - `rust-toolchain.toml`:
    ```toml
    [toolchain]
    channel = "nightly"
    components = ["rust-src"]
    ```
- **ARM 32位交叉编译器**: Switch 32位 ARM 用户空间 (`arm-none-eabi-gcc`)。
- **打包工具**: `cargo-mhgu-forge`（提供 `cargo-forge` 二进制命令）。
- **目标配置**: `armv7a-none-eabi.json`。

### 2. Cargo.toml 配置示例
```toml
[package]
name = "my-mhgu-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]
test = false
doctest = false
bench = false

[dependencies]
mhgu-forge = { version = "1.4.1", features = ["imgui", "mt"] }
```

---

## 二、 怪物猎人GU 内存数据模型

来源于逆向工程与 `MHGU-Monster-Info-Overlay` 的指针链与结构定义：

### 1. 结构体定义 (对齐方式为 packed)
```rust
pub const MAX_POINTERS_IN_LIST: usize = 10;
pub const SEARCH_START_OFFSET: u32 = 0x10A00000;
pub const SEARCH_END_OFFSET: u32 = 0x10F00000;

#[repr(C, packed)]
pub struct Part {
    pub unknown: u16,        // 0x0
    pub fixed: u32,          // 0x2
    pub stagger_count: u8,   // 0x6: 硬直计数
    pub break_count: u8,     // 0x7: 部位破坏计数
    pub stagger_hp: u16,     // 0x8: 当前硬直血量
    pub break_hp: i16,       // 0xA: 当前破坏血量
}

#[repr(C, packed)]
pub struct Monster {
    pub pad_0x0: [u8; 0xD],
    pub location_flag: u8,      // 0xD: 0x4C 同区, 0x44 异区/不在当前地图
    pub pad_0xe: [u8; 0x15DC],
    pub identifier2: u8,        // 0x15EA: 怪物类型分类 (0x04/0x44 常规大怪, 0x0C/0x4C 狞猛/特殊怪, 0x08 麒麟)
    pub pad_0x15eb: [u8; 0x1C5],
    pub hp: u32,                // 0x17B0: 当前血量
    pub max_hp: u32,            // 0x17B4: 最大血量
    pub pad_0x17b8: [u8; 0x3E],
    pub parts: [Part; 8],       // 0x17F6: 部位信息列表 (8个部位)
    pub pad_0x1856: [u8; 0x5DD2],
    pub identifier1: u16,       // 0x7628: 怪物ID (identifier1 << 8 | identifier2 组合为怪物唯一ID)
}

#[repr(C, packed)]
pub struct MonsterPointerList {
    pub fixed1: u8,                             // 固定为 0x01
    pub fixed2: u8,                             // 固定为 0x01
    pub unused: [u8; 0x16],                     // 22 字节预留校验 (数值均 <= 1)
    pub monsters: [*const Monster; MAX_POINTERS_IN_LIST], // 0x18: 32位怪物指针数组
    pub count: u8,                              // 怪物数量
}
```

### 2. 内存特征扫描 (AOB 定位)
- 堆内存基址：`let heap = forge::mem::heap_addr();`
- 搜索区间：`[heap + 0x10A00000, heap + 0x10F00000]`
- 匹配条件：
  1. `fixed1 == 1 && fixed2 == 1`
  2. `unused` 22字节数组内数值满足 `<= 1`
  3. `monsters[0]` 非空，且指向对象的 `max_hp > 100 && max_hp <= 999999 && hp <= max_hp`
- 优化原则：成功定位后缓存指针，未进入任务或指针失效时设置 60 帧以上的扫描冷却（Cooldown），避免每帧暴力遍历堆导致掉帧。

### 3. 怪物 HP 的基址,名称定义

[boss.md](boss.md)

---

## 三、 UI 渲染与生命周期

`mhgu-forge` 提供了基于 Dear ImGui 的完整生命周期回调：

### 1. 核心导出属性宏
- `#[forge::entry]`: 插件加载与初始化入口点（自动生成 `forge_onLoad` / `forge_onInit`）。
- `#[forge::imgui_render]`: 嵌入到 Forge 官方菜单的设置界面（当玩家呼出 Forge 菜单时渲染）。
- `#[forge::imgui_free_render]`: 自由全屏 HUD 覆盖层（每帧直接在游戏画面上叠加绘制）。

### 2. 避免游戏崩溃的关键军规 (CRITICAL)
> [!CAUTION]
> **绝对不要在渲染回调中重建或动态修改 Dear ImGui 的 `ImFontAtlas`！**
> 
> Forge 宿主在启动游戏时已经将字库烘焙（Bake）为 GPU 显存纹理，Atlas 处于锁定状态。在 `#[forge::imgui_render]` 或 `#[forge::imgui_free_render]` 中调用 `ImFontAtlas_AddFontFromFileTTF` 或 `igImFontAtlasBuildMain` 会引发**段错误 (Segmentation Fault)**，导致游戏秒崩。

### 3. HUD 绘制模式最佳实践
- 使用 `DrawList::foreground()` 获取游戏最顶层绘制列表。
- 支持绘制：
  - `add_rect_filled`: 绘制半透明黑色遮罩背景、已损失血量（红色底层）、剩余血量（绿色进度层）。
  - `add_rect`: 绘制高亮细边框（推荐灰色 `color(230, 230, 230, 220)`）。
  - `add_text`: 绘制怪物名称与数值（ASCII 文本兼容性最佳）。

---

## 四、 编译与部署路径

### 1. 编译命令
```powershell
# 一键编译并生成 .nro
cargo-forge forge build --release
```
- 中间产物：`target/armv7a-none-eabi/release/lib<name>.so`
- 最终产物：项目根目录 `<name>.nro`

### 2. Switch SD卡安装路径
- 插件放于：`romfs/nativeNX/plugins/<name>.nro`
- 配置文件（若有）：`romfs/nativeNX/plugins/<name>.ini` 或 `savedata:/<name>.ini`
- 外部字库/资源放于：`romfs/nativeNX/forge/fonts/`
