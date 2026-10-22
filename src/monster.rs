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
    if m.is_null() {
        return false;
    }
    let addr = m as usize as u32;

    // 1. 必须 4 字节对齐（非法指针通常奇数对齐，解引用会导致 Unmapped Read）
    if (addr & 3) != 0 {
        return false;
    }

    // 2. 严格限制在 MHGU 堆内存有效分配范围内
    let heap = forge::mem::heap_addr();
    if heap != 0 {
        if addr < heap || addr > heap.wrapping_add(0x18000000) {
            return false;
        }
    } else {
        if addr < 0x10000000 || addr > 0x38000000 {
            return false;
        }
    }
    true
}

pub unsafe fn is_monster_data_sane(m: *const Monster) -> bool {
    if !is_valid_monster_ptr(m) {
        return false;
    }
    unsafe {
        let max_hp = core::ptr::read_unaligned(core::ptr::addr_of!((*m).max_hp));
        let hp = core::ptr::read_unaligned(core::ptr::addr_of!((*m).hp));

        // 基础血量区间校验 (大型怪物通常血量在 500 以上)
        if max_hp <= 100 || max_hp > 999_999 {
            return false;
        }
        if hp > max_hp {
            return false;
        }

        // 校验怪物类型标识 identifier2 (0x15EA):
        // MHGU 大怪物类型标识一般包括 0x04/0x44 (常规大怪), 0x0C/0x4C (狞猛/特殊怪), 0x08 (麒麟), 0x20/0x60 (跳狗龙等鸟龙王)
        let identifier2 = core::ptr::read_unaligned(core::ptr::addr_of!((*m).identifier2));
        if identifier2 == 0 {
            return false;
        }

        // 校验位置标志 location_flag (0xD):
        // 0x4C 表示同区，0x44 表示异区。村庄中残留实体或非战斗对象通常不具备此特征
        let location_flag = core::ptr::read_unaligned(core::ptr::addr_of!((*m).location_flag));
        if location_flag != 0x4C && location_flag != 0x44 {
            return false;
        }

        // 校验合成怪物 ID 是否在已知大怪图鉴中
        let identifier1 = core::ptr::read_unaligned(core::ptr::addr_of!((*m).identifier1)) as u32;
        let id = (identifier1 << 8) | (identifier2 as u32);
        if !crate::names::is_valid_monster_id(id) {
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

            // 严格校验怪物数量 count (通常为 1 ~ 5，最多不超过 MAX_POINTERS_IN_LIST)
            let count = core::ptr::read_unaligned(core::ptr::addr_of!((*candidate).count));
            if count == 0 || count > MAX_POINTERS_IN_LIST as u8 {
                curr += 4;
                continue;
            }

            // 只要列表中存在任意一个有效且健康的怪物对象，即认为找到了合法的怪物列表
            let mut found_valid = false;
            let check_len = (count as usize).min(MAX_POINTERS_IN_LIST);
            for i in 0..check_len {
                let m = core::ptr::read_unaligned(core::ptr::addr_of!((*candidate).monsters[i]));
                if is_valid_monster_ptr(m) && is_monster_data_sane(m) {
                    found_valid = true;
                    break;
                }
            }

            if found_valid {
                return candidate;
            }
        }

        curr += 4;
    }

    core::ptr::null()
}

pub unsafe fn is_valid_pointer_list(ptr: *const MonsterPointerList) -> bool {
    if ptr.is_null() {
        return false;
    }
    unsafe {
        let f1 = core::ptr::read_unaligned(core::ptr::addr_of!((*ptr).fixed1));
        let f2 = core::ptr::read_unaligned(core::ptr::addr_of!((*ptr).fixed2));
        if f1 != 1 || f2 != 1 {
            return false;
        }

        let count = core::ptr::read_unaligned(core::ptr::addr_of!((*ptr).count));
        if count == 0 || count > MAX_POINTERS_IN_LIST as u8 {
            return false;
        }

        let check_len = (count as usize).min(MAX_POINTERS_IN_LIST);
        for i in 0..check_len {
            let m = core::ptr::read_unaligned(core::ptr::addr_of!((*ptr).monsters[i]));
            if is_valid_monster_ptr(m) && is_monster_data_sane(m) {
                return true;
            }
        }

        false
    }
}
