# MHGU 怪物血条与部位耐久显示模组 (HP Overlay)

一个为 怪物猎人GU（Monster Hunter Generations Ultimate）设计的高度可定制的怪物血条 HUD 模组，支持PC和安卓Switch模拟器。

基于 [Forge](https://github.com/Fexty12573/forge) 插件框架与 Rust 开发。

---

## ✨ 主要特性

- **仿原生 UI 视觉设计**：
  - 支持 **仿 MHGU 原生猎人图腾点阵风格** 与 **现代扁平极简风格** 自由切换。
  - 支持 **水平左右镜像反转**（可将图腾和血条流向镜像，非常适合放置于屏幕左侧，完美贴合游戏原生 UI 且文字保持正常阅读）。
  - 支持 **反向扣血流向** 配置。
- **详细的怪物数据监控**：
  - **总血量**：实时显示怪物当前 HP、最大 HP 及百分比。
  - **同区/异区状态区分**：当怪物与玩家不在同一地图分区时，文字自动暗化提示；同时支持“仅显示同区怪物”过滤。
- **掌机 / 底座模式自适应**：
  - 自动检测 Switch 运行模式（720p 掌机模式 / 1080p 底座模式），自动对坐标、尺寸与字体进行等比缩放，切换无缝不走样。
- **中英双语怪物名称支持**：
  - 内置完整 MHGU 全大型怪物（含常规怪、狞猛化个体、二名特殊个体等）中英文对照表。
- **游戏内可视化实时调参**：
  - 在游戏中按下 `Insert` 键即可呼出 Forge ImGui 菜单，直观拖动滑动条调节位置、长宽、各通道颜色，并可直接在游戏内一键保存至 `hp_overlay.ini`，所见即所得。
- **中文字体与内存优化**：
  - 支持加载外置 `.ttf` 中文字体。
  - 采用字形范围动态构建器，仅对怪物血条所需的汉字与字符生成字形纹理图集，显存与内存开销极低。

---

## 📂 文件结构与部署

### **前置安装**

1. 参考 [Forge](https://github.com/Fexty12573/forge) 的相关说明安装前置。

2. 将编译生成的模组和配置文件放置plugins目录

以Eden模拟器为例，最终的目录结构如下：

```text

eden
    └─ load
        └─ 0100770008DD8000
            └─ mods
                └── switch-mod
                    ├── exefs
                    │   ├── main.npdm
                    │   └── subsdk9
                    └── romfs
                        └── nativeNX
                            ├── forge
                            │   ├── config.ini
                            │   ├── fonts
                            │   │   └── Roboto-Medium.ttf
                            │   └── shaders
                            │       ├── compiled
                            │       │   └── imgui.fbin
                            │       ├── imgui_frag.fsh
                            │       └── imgui_vert.vsh
                            └── plugins
                                ├── hp_overlay.ini  
                                ├── hp_overlay.nro
                                └── LXGWWenKai-Regular.ttf 

```

---

## 🎮 游戏内使用说明

1. **进入任务**：
   - 进入任务地图后，模组会自动完成内存扫描并呈现怪物血条。
2. **呼出配置菜单**：
   - 在已连接键盘的情况下按下键盘 **`Insert`** 键，展开 **`Plugin UI` -> `Monster HP Overlay`**。
   - 菜单内可自由勾选显示选项、调节位置尺寸、更改 RGBA 颜色，点击底部 **Save config to ini** 即可将当前修改保存至存档路径的 `hp_overlay.ini`。注意：该文件存在时，将优先读取此文件，而不是MOD同级目录的配置文件。

---

## 🔍 Debug 调试信息说明

当在配置或菜单中开启 `show_debug = true` 时，屏幕左上方会显示状态诊断栏：

```text
[DEBUG] Mode:Docked Scr:1920x1080 | Ptr:0x50C62094 | Cd:0 | M:1 | Pos:(0,736) Size:(320x45) Font:[Ready]
```

* **`Mode`**：当前运行模式（`Docked` 底座模式 / `Handheld` 掌机模式）。
* **`Scr`**：当前渲染视口分辨率（例如 `1920x1080` 或 `1280x720`）。
* **`Ptr`**：当前定位到的**怪物列表总表（`MonsterPointerList`）在堆内存中的虚拟地址**。非 `0x0`（如 `0x50C62094`）表示已正常锁定怪物数据列表；若为 `0x0` 表示尚未扫到或处于非战斗区域。
* **`Cd`**：内存扫描冷却倒计时（Cooldown，单位：帧）。全区扫描未命中后会重置为 60 帧，期间挂起扫描以保护性能。
* **`M`**：当前识别并符合显示条件的大型怪物有效数量。
* **`Font`**：字体加载器状态（如 `[Ready]` 或加载错误提示）。

---

## 🛠️ 源码编译指南

### 基础环境

参考[Forge-rs](https://github.com/Fexty12573/forge-rs) 的相关说明

### 编译步骤
在项目目录下执行：
```bash
cargo forge build --release
```

---

## 📜 鸣谢与参考

- 感谢[Forge](https://github.com/Fexty12573/forge), [Forge-rs](https://github.com/Fexty12573/forge-rs) 提供的 Switch 注入与 ImGui 渲染基座支持。
- 数据结构与内存偏移参考自开源项目 [MHGU-Monster-Info-Overlay](https://github.com/minazuki19/MHGU-Monster-Info-Overlay)。
