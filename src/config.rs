use alloc::format;

// ==========================================
// 插件配置 (支持 INI 反序列化与序列化)
// ==========================================

#[derive(Clone, Copy, forge::IniDeserialize, forge::IniSerialize)]
pub struct PluginConfig {
    #[section("Position")]
    pub pos_x: f32,
    pub pos_y: f32,
    pub bar_width: f32,
    pub bar_height: f32,
    pub row_spacing: f32,

    #[section("Font")]
    pub font_size: f32,
    pub font_outline: bool,
    pub text_offset_y: f32,
    pub font_color_r: u8,
    pub font_color_g: u8,
    pub font_color_b: u8,
    pub font_color_a: u8,

    pub outline_color_r: u8,
    pub outline_color_g: u8,
    pub outline_color_b: u8,
    pub outline_color_a: u8,

    #[section("BarColor")]
    pub green_r: u8,
    pub green_g: u8,
    pub green_b: u8,
    pub green_a: u8,

    pub red_r: u8,
    pub red_g: u8,
    pub red_b: u8,
    pub red_a: u8,

    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
    pub bg_a: u8,

    pub border_r: u8,
    pub border_g: u8,
    pub border_b: u8,
    pub border_a: u8,

    pub totem_r: u8,
    pub totem_g: u8,
    pub totem_b: u8,
    pub totem_a: u8,

    #[section("Display")]
    pub native_style: bool,
    pub mirror: bool,
    pub reverse_drain: bool,
    pub use_english: bool,
    pub current_map_only: bool,
    pub show_debug: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            pos_x: 480.0,
            pos_y: 35.0,
            bar_width: 320.0,
            bar_height: 33.6,
            row_spacing: 52.0,

            font_size: 14.0,
            font_outline: true,
            text_offset_y: -3.0,
            font_color_r: 255,
            font_color_g: 255,
            font_color_b: 255,
            font_color_a: 255,

            outline_color_r: 0,
            outline_color_g: 0,
            outline_color_b: 0,
            outline_color_a: 220,

            green_r: 45,
            green_g: 205,
            green_b: 75,
            green_a: 240,

            red_r: 210,
            red_g: 45,
            red_b: 45,
            red_a: 230,

            bg_r: 20,
            bg_g: 20,
            bg_b: 20,
            bg_a: 180,

            border_r: 230,
            border_g: 230,
            border_b: 230,
            border_a: 220,

            totem_r: 0,
            totem_g: 185,
            totem_b: 245,
            totem_a: 153,

            native_style: true,
            mirror: true,
            reverse_drain: false,
            use_english: false,
            current_map_only: false,
            show_debug: false,
        }
    }
}

pub static mut CONFIG: PluginConfig = PluginConfig {
    pos_x: 480.0,
    pos_y: 35.0,
    bar_width: 320.0,
    bar_height: 33.6,
    row_spacing: 52.0,

    font_size: 14.0,
    font_outline: true,
    text_offset_y: -3.0,
    font_color_r: 255,
    font_color_g: 255,
    font_color_b: 255,
    font_color_a: 255,

    outline_color_r: 0,
    outline_color_g: 0,
    outline_color_b: 0,
    outline_color_a: 220,

    green_r: 45,
    green_g: 205,
    green_b: 75,
    green_a: 240,

    red_r: 210,
    red_g: 45,
    red_b: 45,
    red_a: 230,

    bg_r: 20,
    bg_g: 20,
    bg_b: 20,
    bg_a: 180,

    border_r: 230,
    border_g: 230,
    border_b: 230,
    border_a: 220,

    totem_r: 0,
    totem_g: 185,
    totem_b: 245,
    totem_a: 153,

    native_style: true,
    mirror: true,
    reverse_drain: false,
    use_english: false,
    current_map_only: false,
    show_debug: false,
};

pub fn load_ini_config() {
    let ini_name = "hp_overlay.ini";
    let alternate_name = "hp-overlay.ini";

    // 仅从 mod 同级目录 (romfs/nativeNX/plugins/) 读取配置，禁止使用 savedata 也禁止向 savedata 写入
    let romfs_mount = forge::fs::romfs::mount_point();
    let paths = [
        format!("{}:/nativeNX/plugins/{}", romfs_mount, ini_name),
        format!("{}:/nativeNX/plugins/{}", romfs_mount, alternate_name),
        format!("{}:/{}", romfs_mount, ini_name),
    ];

    for path in &paths {
        if forge::fs::file_exists(path) {
            if let Ok(ini) = forge::config::ini::Reader::read(path) {
                use forge::config::IniDeserialize;
                unsafe {
                    CONFIG = PluginConfig::from_ini(&ini);
                }
                return;
            }
        }
    }

    // 未找到 ini 文件时，直接保留内存中的默认配置 Default::default()
    unsafe {
        CONFIG = PluginConfig::default();
    }
}
