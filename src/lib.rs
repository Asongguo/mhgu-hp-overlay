#![no_std]

extern crate alloc;

use alloc::format;
use core::fmt::Write;
use forge::imgui::{DrawList, color};
use forge::sys::imgui::{ImDrawFlags, ImVec2};

// ==========================================
// 数据结构定义 (基于 MHGU-Monster-Info-Overlay)
// ==========================================

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

#[derive(Copy, Clone)]
pub struct MonsterDisplay {
    pub id: u32,
    pub hp: u32,
    pub max_hp: u32,
    pub in_current_map: bool,
}

// 简易格式化缓冲（no_std 环境下的轻量字符串缓冲）
struct StringBuffer<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> StringBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> Write for StringBuffer<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = N - self.len;
        let to_copy = bytes.len().min(remaining);
        self.buf[self.len..self.len + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.len += to_copy;
        Ok(())
    }
}

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

    #[section("Display")]
    pub current_map_only: bool,
    pub show_debug: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            pos_x: 480.0,
            pos_y: 35.0,
            bar_width: 320.0,
            bar_height: 18.0,
            row_spacing: 42.0,

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

            current_map_only: false,
            show_debug: false,
        }
    }
}

static mut CONFIG: PluginConfig = PluginConfig {
    pos_x: 480.0,
    pos_y: 35.0,
    bar_width: 320.0,
    bar_height: 18.0,
    row_spacing: 42.0,

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

    current_map_only: false,
    show_debug: false,
};

static mut CACHED_LIST_PTR: *const MonsterPointerList = core::ptr::null();
static mut SCAN_COOLDOWN: u32 = 0;

fn load_ini_config() {
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
                unsafe { CONFIG = PluginConfig::from_ini(&ini); }
                return;
            }
        }
    }

    // 未找到 ini 文件时，直接保留内存中的默认配置 Default::default()
    unsafe { CONFIG = PluginConfig::default(); }
}

#[forge::entry]
fn main() {
    load_ini_config();
}

// ==========================================
// 内存扫描与怪物数据检查
// ==========================================

#[inline(always)]
fn is_valid_monster_ptr(m: *const Monster) -> bool {
    let addr = m as usize as u32;
    // 32 位 MHGU 堆和动态对象地址通常位于 0x10000000 ~ 0x50000000 区间，
    // 避免解引用靠近 0 地址的非法小数值导致模拟器报 Unmapped Read
    addr >= 0x10000000 && addr <= 0x50000000
}

unsafe fn is_monster_data_sane(m: *const Monster) -> bool {
    if !is_valid_monster_ptr(m) {
        return false;
    }
    unsafe {
        let max_hp = core::ptr::read_unaligned(core::ptr::addr_of!((*m).max_hp));
        let hp = core::ptr::read_unaligned(core::ptr::addr_of!((*m).hp));

        if max_hp <= 100 || max_hp > 999_999 {
            return false;
        }
        if hp > max_hp {
            return false;
        }
        true
    }
}

unsafe fn find_monster_pointer_list() -> *const MonsterPointerList {
    let heap = forge::mem::heap_addr();
    if heap == 0 {
        return core::ptr::null();
    }

    let start = heap.wrapping_add(SEARCH_START_OFFSET);
    let end = heap.wrapping_add(SEARCH_END_OFFSET);
    let list_size = core::mem::size_of::<MonsterPointerList>() as u32;

    if end <= start + list_size {
        return core::ptr::null();
    }

    let mut curr = start;
    while curr < end - list_size {
        let candidate = curr as *const MonsterPointerList;

        unsafe {
            let f1 = core::ptr::read_unaligned(core::ptr::addr_of!((*candidate).fixed1));
            let f2 = core::ptr::read_unaligned(core::ptr::addr_of!((*candidate).fixed2));

            if f1 != 1 || f2 != 1 {
                curr += 4;
                continue;
            }

            let unused = core::ptr::read_unaligned(core::ptr::addr_of!((*candidate).unused));
            let mut unused_sane = true;
            for &b in &unused {
                if b > 1 {
                    unused_sane = false;
                    break;
                }
            }
            if !unused_sane {
                curr += 4;
                continue;
            }

            let m0 = core::ptr::read_unaligned(core::ptr::addr_of!((*candidate).monsters[0]));
            if m0.is_null() {
                curr += 4;
                continue;
            }

            if is_monster_data_sane(m0) {
                return candidate;
            }
        }

        curr += 4;
    }

    core::ptr::null()
}

unsafe fn is_valid_pointer_list(ptr: *const MonsterPointerList) -> bool {
    unsafe {
        let f1 = core::ptr::read_unaligned(core::ptr::addr_of!((*ptr).fixed1));
        let f2 = core::ptr::read_unaligned(core::ptr::addr_of!((*ptr).fixed2));
        if f1 != 1 || f2 != 1 {
            return false;
        }
        let m0 = core::ptr::read_unaligned(core::ptr::addr_of!((*ptr).monsters[0]));
        is_monster_data_sane(m0)
    }
}


// ==========================================
// 怪物名称映射 (中文名称)
// ==========================================

// fn get_monster_name(id: u32) -> &'static str {
//     match id {
//         0x452060 => "跳狗龙王",
//         0x27A020 => "蓝速龙王",
//         0x2F2004 => "野猪王",
//         0x40A004 => "穿甲虫",
//         0x3F2044 => "重甲虫",
//         0x3F204C => "狞猛重甲虫",
//         0x39A044 => "青熊兽",
//         0x3A2044 => "红盔",
//         0x5F2020 => "白速龙王",
//         0x282020 => "黄速龙王",
//         0x292044 => "砂龙王",
//         0x25A044 => "怪鸟",
//         0x25A04C => "狞猛怪鸟",
//         0x28A020 => "红速龙王",
//         0x3CA044 => "奇猿狐",
//         0x3CA04C => "狞猛奇猿狐",
//         0x3AA004 => "白兔兽",
//         0x3B2004 => "大雪主",
//         0x262044 => "毒怪鸟",
//         0x26204C => "狞猛毒怪鸟",
//         0x3D2044 => "鬼娃",
//         0x3D204C => "狞猛鬼娃",
//         0x3DA044 => "岩穿",
//         0x2AA044 => "大名盾蟹",
//         0x2AA04C => "狞猛大名盾蟹",
//         0x2B2044 => "矛碎",
//         0x3BA044 => "赤甲兽",
//         0x5EA044 => "桃毛兽王",
//         0x5EA04C => "狞猛桃毛兽王",
//         0x35A044 => "水兽",
//         0x35A04C => "狞猛水兽",
//         0x602044 => "土砂龙",
//         0x60204C => "狞猛土砂龙",
//         0x5A2044 => "岩龙",
//         0x5A204C => "狞猛岩龙",
//         0x41A044 => "夜鸟",
//         0x41A04C => "狞猛夜鸟",
//         0x622044 => "胧隐",
//         0x3E2044 => "变形冰鲨",
//         0x3E204C => "狞猛变形冰鲨",
//         0x252044 => "白电",
//         0x25204C => "狞猛白电",
//         0x612004 => "影蜘蛛",
//         0x61200C => "狞猛影蜘蛛",
//         0x222044 => "雌火龙",
//         0x22204C => "狞猛雌火龙",
//         0x22A044 => "金火龙",
//         0x22A04C => "狞猛金火龙",
//         0x232044 => "紫毒姬",
//         0x23A044 => "火龙",
//         0x23A04C => "狞猛火龙",
//         0x242044 => "银火龙",
//         0x24204C => "狞猛银火龙",
//         0x24A044 => "黑炎王",
//         0x37A004 => "潜口龙",
//         0x37A00C => "狞猛潜口龙",
//         0x26A044 => "水龙",
//         0x26A04C => "狞猛水龙",
//         0x2C2044 => "雪狮子王",
//         0x2C204C => "狞猛雪狮子王",
//         0x312044 => "熔岩龙",
//         0x31204C => "狞猛熔岩龙",
//         0x2BA044 => "将军镰蟹",
//         0x2BA04C => "狞猛将军镰蟹",
//         0x5E2044 => "铠裂",
//         0x3EA004 => "绞蛇龙",
//         0x3EA00C => "狞猛绞蛇龙",
//         0x31A044 => "迅龙",
//         0x31A04C => "狞猛迅龙",
//         0x322044 => "白疾风",
//         0x29A044 => "黑狼鸟",
//         0x29A04C => "狞猛黑狼鸟",
//         0x2A2044 => "独眼",
//         0x342044 => "爆锤龙",
//         0x34204C => "狞猛爆锤龙",
//         0x34A044 => "宝缠",
//         0x352044 => "海龙",
//         0x35204C => "狞猛海龙",
//         0x382044 => "雷狼龙",
//         0x38204C => "狞猛雷狼龙",
//         0x38A04C => "金雷公",
//         0x5FA044 => "冰牙龙",
//         0x5FA04C => "狞猛冰牙龙",
//         0x43A044 => "泡狐龙",
//         0x43A04C => "狞猛泡狐龙",
//         0x632044 => "天眼",
//         0x432044 => "电龙",
//         0x43204C => "狞猛电龙",
//         0x62A044 => "青电主",
//         0x442004 => "巨兽",
//         0x44200C => "狞猛巨兽",
//         0x63A004 => "银峰",
//         0x422044 => "斩龙",
//         0x42204C => "狞猛斩龙",
//         0x42A04C => "烬灭刃",
//         0x362044 => "炎戈龙",
//         0x36204C => "狞猛炎戈龙",
//         0x3FA044 => "黑蚀龙",
//         0x3FA04C => "狞猛黑蚀龙",
//         0x61A044 => "混沌黑蚀龙",
//         0x412044 => "千刃龙",
//         0x41204C => "狞猛千刃龙",
//         0x372044 => "尾锤龙",
//         0x37204C => "狞猛尾锤龙",
//         0x2FA044 => "轰龙",
//         0x2FA04C => "狞猛轰龙",
//         0x302044 => "荒钩爪",
//         0x5AA044 => "铠龙",
//         0x5AA04C => "狞猛铠龙",
//         0x5B2044 => "角龙",
//         0x5B204C => "狞猛角龙",
//         0x5BA044 => "鏖魔",
//         0x272008 => "麒麟",
//         0x3C2044 => "碎龙",
//         0x3C204C => "狞猛碎龙",
//         0x60A044 => "猛爆碎龙",
//         0x40204C => "天廻龙",
//         0x64204C => "天彗龙",
//         0x2CA044 => "金狮子",
//         0x2CA04C => "狞猛金狮子",
//         0x2D204C => "激昂金狮子",
//         0x332044 => "恐暴龙",
//         0x33204C => "狞猛恐暴龙",
//         0x33A044 => "惶怒恐暴龙",
//         0x2DA044 => "钢龙",
//         0x2E204C => "霞龙",
//         0x2EA04C => "炎王龙",
//         0x5C200C => "老山龙",
//         0x30A04C => "霸龙",
//         0x32A04C => "崩龙",
//         0x39200C => "岚龙",
//         0x44A00C => "骸龙",
//         0x00000C => "触手",
//         0x64A00C => "阁螳螂·机甲",
//         0x652004 => "阁螳螂",
//         0x36A04C => "煌黑龙",
//         0x5CA00C => "黑龙",
//         0x5D200C => "红龙",
//         0x5DA00C => "祖龙",
//         _ => "未知怪物",
//     }
// }

// ==========================================
// 怪物名称映射 (英文名称)
// ==========================================

fn get_monster_name(id: u32) -> &'static str {
    match id {
        0x452060 => "Gt. Maccao",
        0x27A020 => "Velocidrom",
        0x2F2004 => "Bulldrome",
        0x40A004 => "Seltas",
        0x3F2044 => "Sel. Queen",
        0x3F204C => "Sel. Queen",
        0x39A044 => "Arzuros",
        0x3A2044 => "Redhelm",
        0x5F2020 => "Giadrome",
        0x282020 => "Gendrome",
        0x292044 => "Cephadrome",
        0x25A044 => "Yian K.K.",
        0x25A04C => "Yian K.K.",
        0x28A020 => "Iodrome",
        0x3CA044 => "K. Wacha",
        0x3CA04C => "K. Wacha",
        0x3AA004 => "Lagombi",
        0x3B2004 => "Snowbaron",
        0x262044 => "Gypceros",
        0x26204C => "Gypceros",
        0x3D2044 => "Tetsucabra",
        0x3D204C => "Tetsucabra",
        0x3DA044 => "Drilltusk",
        0x2AA044 => "Daimyo H.",
        0x2AA04C => "Daimyo H.",
        0x2B2044 => "Stonefist",
        0x3BA044 => "Volvidon",
        0x5EA044 => "Congalala",
        0x5EA04C => "Congalala",
        0x35A044 => "R. Ludroth",
        0x35A04C => "R. Ludroth",
        0x602044 => "Barroth",
        0x60204C => "Barroth",
        0x5A2044 => "Basarios",
        0x5A204C => "Basarios",
        0x41A044 => "Malfestio",
        0x41A04C => "Malfestio",
        0x622044 => "Hazewing",
        0x3E2044 => "Zamtrios",
        0x3E204C => "Zamtrios",
        0x252044 => "Khezu",
        0x25204C => "Khezu",
        0x612004 => "Nerscylla",
        0x61200C => "Nerscylla",
        0x222044 => "Rathian",
        0x22204C => "Rathian",
        0x22A044 => "Gld. Rath.",
        0x22A04C => "Gld. Rath.",
        0x232044 => "Dreadqueen",
        0x23A044 => "Rathalos",
        0x23A04C => "Rathalos",
        0x242044 => "Slv. Rath.",
        0x24204C => "Slv. Rath.",
        0x24A044 => "Dreadking",
        0x37A004 => "Nibelsnarf",
        0x37A00C => "Nibelsnarf",
        0x26A044 => "Plesioth",
        0x26A04C => "Plesioth",
        0x2C2044 => "Blangonga",
        0x2C204C => "Blangonga",
        0x312044 => "Lavasioth",
        0x31204C => "Lavasioth",
        0x2BA044 => "Shogun C.",
        0x2BA04C => "Shogun C.",
        0x5E2044 => "Shredclaw",
        0x3EA004 => "Najarala",
        0x3EA00C => "Najarala",
        0x31A044 => "Nargacuga",
        0x31A04C => "Nargacuga",
        0x322044 => "Silverwind",
        0x29A044 => "Y. Garuga",
        0x29A04C => "Y. Garuga",
        0x2A2044 => "Deadeye",
        0x342044 => "Uragaan",
        0x34204C => "Uragaan",
        0x34A044 => "Crystalbrd",
        0x352044 => "Lagiacrus",
        0x35204C => "Lagiacrus",
        0x382044 => "Zinogre",
        0x38204C => "Zinogre",
        0x38A04C => "Thunderlrd",
        0x5FA044 => "Barioth",
        0x5FA04C => "Barioth",
        0x43A044 => "Mizutsune",
        0x43A04C => "Mizutsune",
        0x632044 => "Divinsight",
        0x432044 => "Astalos",
        0x43204C => "Astalos",
        0x62A044 => "Azurebolt",
        0x442004 => "Gammoth",
        0x44200C => "Gammoth",
        0x63A004 => "Frostpeak",
        0x422044 => "Glavenus",
        0x42204C => "Glavenus",
        0x42A04C => "Hellblade",
        0x362044 => "Agnaktor",
        0x36204C => "Agnaktor",
        0x3FA044 => "G. Magala",
        0x3FA04C => "G. Magala",
        0x61A044 => "Chaotic GM",
        0x412044 => "Seregios",
        0x41204C => "Seregios",
        0x372044 => "Duramboros",
        0x37204C => "Duramboros",
        0x2FA044 => "Tigrex",
        0x2FA04C => "Tigrex",
        0x302044 => "Grimclaw",
        0x5AA044 => "Gravios",
        0x5AA04C => "Gravios",
        0x5B2044 => "Diablos",
        0x5B204C => "Diablos",
        0x5BA044 => "Bloodlust",
        0x272008 => "Kirin",
        0x3C2044 => "Brachydios",
        0x3C204C => "Brachydios",
        0x60A044 => "Rg. Brachy",
        0x40204C => "S. Magala",
        0x64204C => "Valfalk",
        0x2CA044 => "Rajang",
        0x2CA04C => "Rajang",
        0x2D204C => "Fr. Rajang",
        0x332044 => "Deviljho",
        0x33204C => "Deviljho",
        0x33A044 => "Savage D.",
        0x2DA044 => "K. Daora",
        0x2E204C => "Chameleos",
        0x2EA04C => "Teostra",
        0x5C200C => "LaoShan L.",
        0x30A04C => "Akantor",
        0x32A04C => "Ukanlos",
        0x39200C => "Amatsu",
        0x44A00C => "Nakarkos",
        0x00000C => "Tentacle",
        0x64A00C => "Altal Thrn",
        0x652004 => "Atlal Ka",
        0x36A04C => "Alatreon",
        0x5CA00C => "Fatalis",
        0x5D200C => "Crimson F.",
        0x5DA00C => "White F.",
        _ => "Unknown",
    }
}

// ==========================================
// 绘制逻辑
// ==========================================

fn draw_outlined_text(
    draw_list: &DrawList,
    pos: ImVec2,
    text_col: forge::sys::imgui::ImU32,
    outline_col: forge::sys::imgui::ImU32,
    font_outline: bool,
    text: &str,
) {
    if font_outline {
        let offsets = [
            (-1.0, 0.0),
            (1.0, 0.0),
            (0.0, -1.0),
            (0.0, 1.0),
            (-1.0, -1.0),
            (1.0, -1.0),
            (-1.0, 1.0),
            (1.0, 1.0),
        ];
        for (dx, dy) in offsets {
            draw_list.add_text(
                ImVec2::new(pos.x + dx, pos.y + dy),
                outline_col,
                text,
            );
        }
    }

    draw_list.add_text(pos, text_col, text);
}

fn draw_debug_hud(
    heap: u32,
    list_ptr: *const MonsterPointerList,
    cooldown: u32,
    valid_count: usize,
    extra_log: Option<&str>,
) {
    let draw_list = DrawList::foreground();
    let text_yellow = color(255, 230, 80, 240);
    let bg_color = color(0, 0, 0, 160);

    let mut buf = StringBuffer::<256>::new();
    let ptr_val = list_ptr as usize as u32;

    // 获取当前屏幕视口分辨率 (优先通过 ImGuiIO::DisplaySize 获取，比 Viewport 更通用稳定)
    let (scr_w, scr_h) = unsafe {
        let io = forge::sys::imgui::igGetIO_Nil();
        if !io.is_null() {
            ((*io).DisplaySize.x, (*io).DisplaySize.y)
        } else {
            (0.0, 0.0)
        }
    };

    if let Some(extra) = extra_log {
        let _ = write!(
            buf,
            "[DEBUG] Scr:{:.0}x{:.0} | Ptr:0x{:X} | Cd:{} | M:{} | {}",
            scr_w, scr_h, ptr_val, cooldown, valid_count, extra
        );
    } else {
        let _ = write!(
            buf,
            "[DEBUG] Scr:{:.0}x{:.0} | Ptr:0x{:X} | Cd:{} | M:{}",
            scr_w, scr_h, ptr_val, cooldown, valid_count
        );
    }

    // 每个英文字符宽约 7~8 像素，据此动态估算背景框宽度
    let text_str = buf.as_str();
    let estimated_width = (text_str.len() as f32 * 7.5 + 24.0).max(560.0);

    draw_list.add_rect_filled(
        ImVec2::new(8.0, 8.0),
        ImVec2::new(8.0 + estimated_width, 28.0),
        bg_color,
        2.0,
    );
    draw_list.add_text(ImVec2::new(12.0, 10.0), text_yellow, text_str);
}

fn draw_hp_overlay(monsters: &[MonsterDisplay]) {
    let draw_list = DrawList::foreground();

    let cfg = unsafe { CONFIG };

    let start_x = cfg.pos_x;
    let mut current_y = cfg.pos_y;
    let bar_width = cfg.bar_width;
    let bar_height = cfg.bar_height;
    let row_spacing = cfg.row_spacing;

    let bg_color = color(cfg.bg_r, cfg.bg_g, cfg.bg_b, cfg.bg_a);
    let border_color = color(cfg.border_r, cfg.border_g, cfg.border_b, cfg.border_a);
    let red_lost = color(cfg.red_r, cfg.red_g, cfg.red_b, cfg.red_a);
    let green_remain = color(cfg.green_r, cfg.green_g, cfg.green_b, cfg.green_a);

    let text_color = color(cfg.font_color_r, cfg.font_color_g, cfg.font_color_b, cfg.font_color_a);
    let text_away_color = color(
        cfg.font_color_r.saturating_sub(60),
        cfg.font_color_g.saturating_sub(60),
        cfg.font_color_b.saturating_sub(60),
        cfg.font_color_a.saturating_sub(50).max(120),
    );
    let outline_color = color(
        cfg.outline_color_r,
        cfg.outline_color_g,
        cfg.outline_color_b,
        cfg.outline_color_a,
    );

    for m in monsters {
        let bar_top = current_y;
        let bar_bottom = bar_top + bar_height;

        // 1. 底框背景
        draw_list.add_rect_filled(
            ImVec2::new(start_x, bar_top),
            ImVec2::new(start_x + bar_width, bar_bottom),
            bg_color,
            3.0,
        );

        // 2. 红色底层: 代表已损失血量
        draw_list.add_rect_filled(
            ImVec2::new(start_x, bar_top),
            ImVec2::new(start_x + bar_width, bar_bottom),
            red_lost,
            3.0,
        );

        // 3. 绿色顶层: 代表剩余血量
        if m.max_hp > 0 && m.hp > 0 {
            let ratio = (m.hp as f32 / m.max_hp as f32).clamp(0.0, 1.0);
            let green_width = bar_width * ratio;
            draw_list.add_rect_filled(
                ImVec2::new(start_x, bar_top),
                ImVec2::new(start_x + green_width, bar_bottom),
                green_remain,
                3.0,
            );
        }

        // 4. 外边框
        draw_list.add_rect(
            ImVec2::new(start_x, bar_top),
            ImVec2::new(start_x + bar_width, bar_bottom),
            border_color,
            3.0,
            1.2,
            ImDrawFlags::None,
        );

        // 5. 血条上层覆盖文字描述，例如: "青熊兽: 500/1000"
        let mut text_buf = StringBuffer::<64>::new();
        let name = get_monster_name(m.id);
        let _ = write!(text_buf, "{}: {}/{}", name, m.hp, m.max_hp);


        let label_color = if m.in_current_map { text_color } else { text_away_color };

        // 垂直居中于血条 (综合字体行高基准与大写字母光学中心，配合 cfg.text_offset_y 微调)
        let text_y = bar_top + (bar_height - cfg.font_size) * 0.5 + cfg.text_offset_y;
        let text_x = start_x + 6.0;

        draw_outlined_text(
            &draw_list,
            ImVec2::new(text_x, text_y),
            label_color,
            outline_color,
            cfg.font_outline,
            text_buf.as_str(),
        );

        current_y += row_spacing;
    }
}

// ==========================================
// 游戏画面 HUD 叠加渲染回调
// ==========================================

#[forge::imgui_free_render]
fn render_overlay() {
    unsafe {
        let heap = forge::mem::heap_addr();

        // 1. 获取/刷新列表指针
        if CACHED_LIST_PTR.is_null() || !is_valid_pointer_list(CACHED_LIST_PTR) {
            if SCAN_COOLDOWN == 0 {
                CACHED_LIST_PTR = find_monster_pointer_list();
                SCAN_COOLDOWN = 60; // 冷却 60 帧
            } else {
                SCAN_COOLDOWN -= 1;
            }
        }

        // 2. 读取怪物并过滤
        let mut list: [MonsterDisplay; MAX_POINTERS_IN_LIST] = [
            MonsterDisplay {
                id: 0,
                hp: 0,
                max_hp: 0,
                in_current_map: false,
            };
            MAX_POINTERS_IN_LIST
        ];
        let mut valid_count = 0;

        if !CACHED_LIST_PTR.is_null() {
            for i in 0..MAX_POINTERS_IN_LIST {
                let m_ptr = core::ptr::read_unaligned(core::ptr::addr_of!((*CACHED_LIST_PTR).monsters[i]));
                if m_ptr.is_null() {
                    break;
                }

                if is_monster_data_sane(m_ptr) {
                    let location_flag = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).location_flag));
                    let identifier2 = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).identifier2)) as u32;
                    let identifier1 = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).identifier1)) as u32;
                    let id = (identifier1 << 8) | identifier2;
                    let hp = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).hp));
                    let max_hp = core::ptr::read_unaligned(core::ptr::addr_of!((*m_ptr).max_hp));

                    let in_current_map = location_flag == 0x4C;

                    let should_display = if CONFIG.current_map_only {
                        in_current_map && max_hp > 500
                    } else {
                        max_hp > 500
                    };

                    if should_display {
                        list[valid_count] = MonsterDisplay {
                            id,
                            hp,
                            max_hp,
                            in_current_map,
                        };
                        valid_count += 1;
                    }
                }
            }
        }

        let cfg = CONFIG;

        // 3. 诊断 HUD 渲染 (可选)
        if cfg.show_debug {
            // 在 debug 栏中打印当前血条使用的 (pos_x, pos_y) 坐标及尺寸，方便排查是否在屏幕外
            let mut info_buf = StringBuffer::<64>::new();
            let _ = write!(
                info_buf,
                "Pos:({:.0},{:.0}) Size:({:.0}x{:.0})",
                cfg.pos_x, cfg.pos_y, cfg.bar_width, cfg.bar_height
            );
            draw_debug_hud(heap, CACHED_LIST_PTR, SCAN_COOLDOWN, valid_count, Some(info_buf.as_str()));
        }

        // 4. 无怪物时的处理：若开启 debug 模式，显示占位血条以便用户校准位置
        if valid_count == 0 {
            if cfg.show_debug {
                // 设置为 500/1000，确保同时能看到绿条（剩余血量）和红条（损失血量）
                let dummy = [MonsterDisplay {
                    id: 0,
                    hp: 500,
                    max_hp: 1000,
                    in_current_map: true,
                }];
                draw_hp_overlay(&dummy);
            }
            return;
        }

        // 4. 按照总血量从大到小排序 (纵向排列)
        let active_slice = &mut list[..valid_count];
        for i in 0..active_slice.len() {
            for j in (i + 1)..active_slice.len() {
                if active_slice[j].max_hp > active_slice[i].max_hp {
                    active_slice.swap(i, j);
                }
            }
        }

        // 5. 屏幕 HUD 叠加绘制
        draw_hp_overlay(active_slice);
    }
}