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
    pub enabled: bool,
    pub native_style: bool,
    pub mirror: bool,
    pub reverse_drain: bool,
    pub use_english: bool,
    pub current_map_only: bool,
    pub show_debug: bool,
    pub show_parts: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 736.0,
            bar_width: 320.0,
            bar_height: 45.0,
            row_spacing: 52.0,

            font_size: 24.0,
            font_outline: true,
            text_offset_y: -0.0,
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
            green_a: 120,

            red_r: 210,
            red_g: 45,
            red_b: 45,
            red_a: 120,

            bg_r: 20,
            bg_g: 20,
            bg_b: 20,
            bg_a: 0,

            border_r: 230,
            border_g: 230,
            border_b: 230,
            border_a: 0,

            totem_r: 70,
            totem_g: 130,
            totem_b: 230,
            totem_a: 200,

            enabled: true,
            native_style: true,
            mirror: true,
            reverse_drain: false,
            use_english: false,
            current_map_only: false,
            show_debug: false,
            show_parts: true,
        }
    }
}

pub static mut CONFIG: PluginConfig = PluginConfig {
    pos_x: 0.0,
    pos_y: 736.0,
    bar_width: 320.0,
    bar_height: 45.0,
    row_spacing: 52.0,

    font_size: 24.0,
    font_outline: true,
    text_offset_y: -0.0,
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
    green_a: 120,

    red_r: 210,
    red_g: 45,
    red_b: 45,
    red_a: 120,

    bg_r: 20,
    bg_g: 20,
    bg_b: 20,
    bg_a: 0,

    border_r: 230,
    border_g: 230,
    border_b: 230,
    border_a: 0,

    totem_r: 70,
    totem_g: 130,
    totem_b: 230,
    totem_a: 200,

    enabled: true,
    native_style: true,
    mirror: true,
    reverse_drain: false,
    use_english: false,
    current_map_only: false,
    show_debug: false,
    show_parts: true,
};

pub static mut CUSTOM_FONT_PATH: [u8; 128] = {
    let mut buf = [0u8; 128];
    let src = b"LXGWWenKai-Regular.ttf";
    let mut i = 0;
    while i < src.len() {
        buf[i] = src[i];
        i += 1;
    }
    buf
};

pub fn get_custom_font_path() -> &'static str {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(CUSTOM_FONT_PATH) as *const u8;
        let mut len = 0;
        while len < 128 && *ptr.add(len) != 0 {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(ptr, len);
        core::str::from_utf8(slice).unwrap_or("")
    }
}

pub fn set_custom_font_path(path: &str) {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(CUSTOM_FONT_PATH) as *mut u8;
        let bytes = path.as_bytes();
        let copy_len = bytes.len().min(127);
        for i in 0..copy_len {
            *ptr.add(i) = bytes[i];
        }
        *ptr.add(copy_len) = 0;
    }
}

pub fn clear_custom_font_path() {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(CUSTOM_FONT_PATH) as *mut u8;
        *ptr = 0;
    }
}

pub struct SafeSaveDataMount(bool);

impl SafeSaveDataMount {
    pub fn try_mount() -> Option<Self> {
        if forge::fs::savedata::mount() {
            Some(SafeSaveDataMount(true))
        } else {
            None
        }
    }
}

impl Drop for SafeSaveDataMount {
    fn drop(&mut self) {
        if self.0 {
            forge::fs::savedata::unmount();
        }
    }
}

fn apply_ini(ini: &forge::config::ini::Ini) {
    use forge::config::IniDeserialize;
    unsafe {
        CONFIG = PluginConfig::from_ini(ini);

        if let Some(font_p) = ini.get("Font", "font_path") {
            let trimmed = font_p.trim();
            if !trimmed.is_empty() && trimmed != "default" {
                set_custom_font_path(trimmed);
            } else {
                set_custom_font_path("LXGWWenKai-Regular.ttf");
            }
        } else {
            set_custom_font_path("LXGWWenKai-Regular.ttf");
        }
    }
}

pub fn load_ini_config() {
    let ini_name = "hp_overlay.ini";
    let alternate_name = "hp-overlay.ini";

    // 1. 优先读取 savedata 下保存的配置（用户在 Forge 菜单中修改并保存的 ini）
    if let Some(_mount) = SafeSaveDataMount::try_mount() {
        let save_mount = forge::fs::savedata::mount_point();
        let save_path = format!("{}:/{}", save_mount, ini_name);
        if forge::fs::file_exists(&save_path) {
            if let Ok(ini) = forge::config::ini::Reader::read(&save_path) {
                apply_ini(&ini);
                return;
            }
        }
    }

    // 2. 备选：从 mod 目录 (romfs/nativeNX/plugins/) 读取默认包内的 ini 配置
    let romfs_mount = forge::fs::romfs::mount_point();
    let paths = [
        format!("{}:/nativeNX/plugins/{}", romfs_mount, ini_name),
        format!("{}:/nativeNX/plugins/{}", romfs_mount, alternate_name),
        format!("{}:/{}", romfs_mount, ini_name),
    ];

    for path in &paths {
        if forge::fs::file_exists(path) {
            if let Ok(ini) = forge::config::ini::Reader::read(path) {
                apply_ini(&ini);
                return;
            }
        }
    }

    // 未找到 ini 文件时，保留内存中的默认配置 Default::default()
    unsafe {
        CONFIG = PluginConfig::default();
        set_custom_font_path("LXGWWenKai-Regular.ttf");
    }
}

pub fn save_ini_config() -> Result<(), &'static str> {
    use forge::config::IniSerialize;

    // Switch 系统对 romfs 具有硬性只读限制，试图写/删 romfs 会触发 nn::fs::DeleteFile 导致 nnSdk panic 崩溃。
    // 因此写入必须保存至 savedata (即 MHGU save: /hp_overlay.ini)
    let _mount = SafeSaveDataMount::try_mount().ok_or("Failed to mount savedata")?;
    let save_mount = forge::fs::savedata::mount_point();
    let path = format!("{}:/hp_overlay.ini", save_mount);

    let cfg = unsafe { *core::ptr::addr_of!(CONFIG) };
    let mut ini = cfg.to_ini();

    let font_p = get_custom_font_path();
    let font_to_save = if font_p.is_empty() || font_p == "default" {
        "LXGWWenKai-Regular.ttf"
    } else {
        font_p
    };
    ini.set("Font", "font_path", alloc::string::String::from(font_to_save));

    forge::config::ini::Writer::write(&path, &ini)
        .map_err(|_| "Failed to write INI file to savedata")?;

    Ok(())
}
