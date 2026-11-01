//! MHGU 1.4.0 current hunter equipment pack, shared with the game's skill detail UI.
use core::ptr;
unsafe fn word(a:u32)->u32 { unsafe{ptr::read_unaligned(a as *const u32)} }
fn valid(a:u32)->bool {(0x10000..=0xeffff000).contains(&a)}

pub const UNKNOWN:i16=i16::MIN;

/// Compare the selected inventory entry with the actual equipped entry index.
/// Item name/type equality alone cannot distinguish duplicate equipment.
pub fn attachment_target_worn(base:u32,data:u32,upper_cursor:u32)->Option<bool>{
    if !valid(data)||!valid(upper_cursor){return None}
    let kind=unsafe{ptr::read_unaligned((data+0x18) as *const u16)} as u32&31;
    let slot=crate::equipment_context::equipment_slot(kind)?;
    // The item-box and smithy controllers retain their equipment-grid cursors at
    // different offsets from the upper socket cursor. Validate the native type.
    let mut index=None;
    for grid in [upper_cursor.wrapping_sub(0xb4),upper_cursor+0x48]{
        if !valid(grid)||unsafe{word(grid)}!=base+0x176cb9c{continue}
        let b=|off:u32|unsafe{ptr::read((grid+off) as *const u8)} as u32;
        let h=|off:u32|unsafe{ptr::read_unaligned((grid+off) as *const u16)} as u32;
        if let Some(i)=crate::equipment_context::equipment_grid_index(b(0x14),b(0x15),b(0x16),b(0x17),h(0x18),h(0x1a)){
            if index.is_some_and(|previous|previous!=i){return None}
            index=Some(i);
        }
    }
    let index=index?;
    let hunter=unsafe{word(base+0x1885b58)};
    if !valid(hunter){return None}
    let pack=hunter+0x7c;
    if unsafe{word(pack)}!=base+0x1747bb4{return None}
    // cFestaStatusParam's seven inventory entry indices begin at status+4.
    let worn=unsafe{ptr::read_unaligned((pack+8+slot as u32*2) as *const u16)};
    Some(worn!=0xffff&&worn==index)
}

/// Remove an equipped armor/charm's complete column, including socketed jewels.
/// Torso Up copies are represented in the body's column and must also be reduced
/// when removing a piece that supplies Torso Up. The live pack is read only.
pub fn after_removal(base:u32,slot:usize)->[i16;206]{
    let unknown=[UNKNOWN;206];
    if !(1..=6).contains(&slot){return unknown}
    let hunter=unsafe{word(base+0x1885b58)};
    if !valid(hunter){return unknown}
    let pack=hunter+0x7c;
    if unsafe{word(pack)}!=base+0x1747bb4{return unknown}
    let table=unsafe{word(pack+0x18c)};
    if table==0{return [0;206]}
    if !valid(table){return unknown}
    let mut torso=0;let mut removed_torso=0;
    for i in 0..77{
        let row=table+i*12;let id=unsafe{ptr::read_unaligned((row+6) as *const u16)};
        if id==0||id==206{break}
        if id==203{
            for col in 0..7{torso+=unsafe{ptr::read((row+8+col) as *const u8)} as i32;}
            removed_torso=unsafe{ptr::read((row+8+slot as u32) as *const u8)} as i32;
        }
    }
    let mut result=[0;206];
    for i in 0..77{
        let row=table+i*12;let id=unsafe{ptr::read_unaligned((row+6) as *const u16)};
        if id==0||id==206{break}
        if id>=206{continue}
        let total=unsafe{ptr::read((row+15) as *const i8)} as i32;
        let own=unsafe{ptr::read((row+8+slot as u32) as *const i8)} as i32;
        let body=unsafe{ptr::read((row+10) as *const i8)} as i32;
        result[id as usize]=if id==203{0}else{
            crate::comparison::remove_slot(total,own,body,torso,removed_torso,slot==2).map_or(UNKNOWN,|n|n as i16)
        };
    }
    result
}

/// The same armor and weapon type predicates used by MHGU's equipment preview.
/// In particular, shared headgear is not rejected merely because of its name.
pub fn preview_compatible(base:u32,equip:u32)->Option<bool>{
    if !valid(equip){return None}
    let kind=unsafe{ptr::read_unaligned((equip+4) as *const u16)}&31;
    if !(1..=5).contains(&kind){return Some(true)}
    let hunter=unsafe{word(base+0x1885b58)};
    if !valid(hunter){return None}
    let pack=hunter+0x7c;
    if unsafe{word(pack)}!=base+0x1747bb4{return None}
    // cFestaStatusParam starts at pack+4; slot 0 is at status+0x14.
    let weapon=pack+0x18;
    let weapon_kind=unsafe{ptr::read_unaligned((weapon+4) as *const u16)}&31;
    if !(7..=21).contains(&weapon_kind){return None}
    let blade:unsafe extern "C" fn(u32)->u32=unsafe{core::mem::transmute(base+0xdb0b0)};
    let gunner:unsafe extern "C" fn(u32)->u32=unsafe{core::mem::transmute(base+0xdb160)};
    let weapon_blade:unsafe extern "C" fn(u32)->u32=unsafe{core::mem::transmute(base+0xdaf18)};
    Some(crate::comparison::matches_equipment_type(unsafe{weapon_blade(weapon)}!=0,unsafe{blade(equip)}!=0,unsafe{gunner(equip)}!=0))
}
/// Copy while the detail renderer's calculated equipment pack is alive.
/// Use the game's totals so Torso Up and socket contributions are accounted for.
pub fn snapshot(base:u32,pack:u32)->[i16;206]{
    let unknown=[UNKNOWN;206];
    if !valid(pack)||unsafe{word(pack)}!=base+0x1747bb4{return unknown}
    let table=unsafe{word(pack+4+0x188)};
    if table!=0&&!valid(table){return unknown}
    let mut result=[0;206];
    if table!=0{for i in 0..77{
        let row=table+i*12;
        let id=unsafe{ptr::read_unaligned((row+6) as *const u16)};
        if id==0||id==206{break}
        if (id as usize)<result.len(){result[id as usize]=unsafe{ptr::read((row+15) as *const i8)} as i16;}
    }}
    result
}

pub fn total(base:u32,gu_tree:u16)->Option<i32> {
    // 0x144788 loads this singleton; 0x277454 returns its active pack at +0x7c.
    // cFestaEquipPack owns its calculated cFestaStatusParam at +4.
    let hunter=unsafe{word(base+0x1885b58)};
    if !valid(hunter){return None}
    let pack=hunter+0x7c;
    if unsafe{word(pack)}!=base+0x1747bb4{return None}
    // 0xe0e7c returns the skill table pointer from status +0x188.
    let table=unsafe{word(pack+4+0x188)};
    if table==0{return Some(0)}
    if !valid(table){return None}
    // The game's 0x631830 renderer reads up to 77 rows, 12-byte stride:
    // tree ID at +6, seven equipment columns at +8..14, signed total at +15.
    for i in 0..77 {
        let row=table+i*12;
        let id=unsafe{ptr::read_unaligned((row+6) as *const u16)};
        if id==0||id==206{break}
        if id==gu_tree{return Some(unsafe{ptr::read((row+15) as *const i8)} as i32)}
    }
    Some(0)
}
