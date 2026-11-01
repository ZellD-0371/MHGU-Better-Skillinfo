//! Snapshots of the game's equipment detail renderer; no retained stack pointers.
use core::ptr;
use crate::{Armor,STATE,TREES,rd,byte};
fn valid(p:u32)->bool{(0x10000..0xeffff000).contains(&p)}
fn add(a:&mut Armor,gu:u16,points:i32){
    if gu==0{return}
    let Some(tree)=TREES.iter().find(|t|t.gu_id==gu) else{return};
    crate::equipment_skills::add(&mut a.points,&mut a.count,tree.id,points);
}
pub fn capture(owner:u32,equip:u32,page_kind:u32){
    if !valid(owner){return}
    let (base,enabled)=STATE.with(|s|(s.base,s.config.enabled&&(s.config.armor_enabled||s.config.change_enabled||s.config.deco_enabled)));
    if !enabled{return}
    let mut a=Armor{owner,..Armor::EMPTY};
    // The dispatch writes owner+F8. Its page list starts at +5C, terminated by kind 28.
    // An optional socket page (8) also appears in forging; it cannot identify the operation.
    let mut page_count=0;let mut sockets=false;
    for i in 0..11{let k=unsafe{rd(owner+0x5c+i*4)};if k==28{break}page_count+=1;sockets|=k==8;}
    let context=crate::equipment_context::classify(unsafe{rd(owner+0x54)},unsafe{rd(owner+0x58)},page_count,sockets);
    a.production=matches!(context,crate::equipment_context::EquipmentContext::Production|crate::equipment_context::EquipmentContext::ProductionComparison);
    a.change=matches!(context,crate::equipment_context::EquipmentContext::Change|crate::equipment_context::EquipmentContext::ProductionComparison);
    a.upgrade=context==crate::equipment_context::EquipmentContext::Upgrade;
    a.relocate=context==crate::equipment_context::EquipmentContext::Move;
    a.inspect=context==crate::equipment_context::EquipmentContext::Inspect;
    a.attachment=crate::equipment_context::attachment_browse(unsafe{rd(owner+0x54)},unsafe{rd(owner+0x58)},unsafe{rd(owner+0x50)});
    a.selected=unsafe{rd(owner+0x50)}&0x4000!=0;
    if valid(equip){
        let raw=unsafe{rd(equip+4)};
        a.identity=raw as u64|((unsafe{rd(equip+8)} as u64)<<32);
        let kind=raw&31;
        let items=core::array::from_fn(|slot|unsafe{ptr::read_unaligned((equip+0xa+slot as u32*2) as *const u16)});
        if (1..=6).contains(&kind){a.slot=kind as u8;}
        let pager=unsafe{rd(owner+0x4c)};
        let page=if valid(pager){unsafe{byte(pager+0xd)}}else{0};
        a.eligible=if a.attachment{
            crate::equipment_context::attachment_skill_page(kind,page_kind,items.iter().any(|&item|item!=0))
        }else if a.relocate||a.inspect{
            crate::equipment_context::relocation_skill_page(kind,page_kind)
        }else if a.change{
            (matches!(page,3|4)&&matches!(page_kind,7|8))||(kind==6&&page==0&&page_kind==4)
        }else if a.upgrade{
            crate::equipment_context::upgrade_skill_page(kind,page_kind)
        }else{(1..=5).contains(&kind)&&page_kind==7};
        if a.eligible{
            // A mismatched armor can still have valid intrinsic skills, but it
            // cannot supply a meaningful After value for the current weapon type.
            // Keep UNKNOWN instead of presenting the game's excluded skill as 0.
            if !a.inspect&&!a.attachment{
                let compatible=crate::equipped::preview_compatible(base,equip);
                a.incompatible=compatible==Some(false);
                if compatible==Some(true){
                    a.pack_totals=crate::equipped::snapshot(base,unsafe{rd(owner+0xb8)});
                }
            }
            if a.attachment{
                if kind==6{
                    for n in 0..2{add(&mut a,unsafe{byte(equip+0x10+n)} as u16,unsafe{ptr::read((equip+0x12+n) as *const i8)} as i32);}
                }else{
                    let table=unsafe{rd(equip+0x28)};
                    if valid(table){for i in 0..11{
                        let row=table+i*4;
                        add(&mut a,unsafe{ptr::read_unaligned(row as *const u16)},unsafe{ptr::read((row+2) as *const i8)} as i32);
                    }}
                }
                crate::equipment_skills::decorations(&mut a.points,&mut a.count,items);
            }else if page_kind==8{
                // Same helper and signed byte layout as the game's socket-page renderer.
                let deco:unsafe extern "C" fn(u32,u32)->u32=unsafe{core::mem::transmute((base+0xdb7b8) as usize)};
                for slot in 0..3{
                    let id=unsafe{ptr::read_unaligned((equip+0xa+slot*2) as *const u16)};
                    if id==0{continue}
                    let row=unsafe{deco(equip,slot)};
                    if valid(row){for n in 0..2{add(&mut a,unsafe{byte(row+5+n)} as u16,unsafe{ptr::read((row+7+n) as *const i8)} as i32);}}
                }
            }else if kind==6&&page_kind==4{
                // Charm first-page renderer +587AF0 reads these two intrinsic skills.
                for n in 0..2{add(&mut a,unsafe{byte(equip+0x10+n)} as u16,unsafe{ptr::read((equip+0x12+n) as *const i8)} as i32);}
            }else{
                let table=unsafe{rd(equip+0x28)};
                if valid(table){for i in 0..11{
                    let row=table+i*4;
                    add(&mut a,unsafe{ptr::read_unaligned(row as *const u16)},unsafe{ptr::read((row+2) as *const i8)} as i32);
                }}
            }
        }
    }
    STATE.with(|s|{
        let index=s.armors.iter().position(|x|x.owner==owner).or_else(||s.armors.iter().position(|x|x.owner==0)).unwrap_or(0);
        s.armors[index]=a;
    });
}
