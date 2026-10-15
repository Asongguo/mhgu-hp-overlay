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

// ==========================================
// 内存扫描与怪物数据检查
// ==========================================

#[inline(always)]
pub fn is_valid_monster_ptr(m: *const Monster) -> bool {
    let addr = m as usize as u32;
    // 32 位 MHGU 堆和动态对象地址通常位于 0x10000000 ~ 0x50000000 区间，
    // 避免解引用靠近 0 地址的非法小数值导致模拟器报 Unmapped Read
    addr >= 0x10000000 && addr <= 0x50000000
}

pub unsafe fn is_monster_data_sane(m: *const Monster) -> bool {
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

pub unsafe fn find_monster_pointer_list() -> *const MonsterPointerList {
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

pub unsafe fn is_valid_pointer_list(ptr: *const MonsterPointerList) -> bool {
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
