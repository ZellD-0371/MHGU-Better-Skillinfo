//! Decoration attachment uses a different projection from decoration crafting.
use core::ptr;
use crate::{Content,Projection,State,DECOS,comparison,rd,byte};
fn valid(p:u32)->bool{(0x10000..0xeffff000).contains(&p)}
unsafe fn half(p:u32)->u16{unsafe{ptr::read_unaligned(p as *const u16)}}
pub fn active(s:&State)->bool{
    let owner=s.socket_owner;
    valid(owner)&&unsafe{rd(owner)==s.base+0x1776d0c&&byte(owner+0x18)!=0}
}
pub(crate) fn center_layout(s:&State,data:u32)->Option<bool>{
    // Follow the game's actual description-panel layout, not attachment-data +7:
    // that byte also changes with socket contents and is not a location flag.
    let info=s.socket_info_owner;
    // Selecting Remove hides the native description, but the live object still
    // owns the same layout. Visibility must not suppress our removal projection.
    if !valid(info)||unsafe{rd(info)!=s.base+0x177613c||rd(info+0x4c)!=data}{return None}
    let layout=unsafe{rd(info+0x24)};
    if !valid(layout){return None}
    // Same group lookup as the native coordinate getter at +545A04.
    let group=0x23;let count=unsafe{rd(layout+0x298)};
    let (table,index)=if group<count{(unsafe{rd(layout+0xf8)},group)}else{(unsafe{rd(layout+0x294)},group-count)};
    if !valid(table){return None}
    let node=unsafe{rd(table+index*4)};
    if !valid(node){return None}
    let x=f32::from_bits(unsafe{rd(node+0x90)});
    if !x.is_finite(){return None}
    // Item box shifts this group left to -442; the smithy keeps it at 0.
    // Native coordinates are independent of the overlay's resolution scaling.
    Some(x< -200.)
}
pub fn content(s:&State)->Option<Content>{
    if !s.config.deco_enabled{return None}
    let owner=s.socket_owner;
    let data=unsafe{rd(owner+0x4c)};
    let cursor=unsafe{rd(owner+0x50)};
    if !valid(data)||!valid(cursor){return None}
    // +4: 0=select installed socket, 1=select replacement/removal.
    // +5: highlighted row in the active list; upper cursor retains the socket index.
    // Each UI item is 24 bytes; quantity at +6 and global item ID at +8.
    // Verified against the running game's slot/removal/replacement states.
    let phase=unsafe{byte(data+4)};
    let slot=unsafe{byte(cursor+0xd)} as u32;
    let choice=unsafe{byte(data+5)} as u32;
    if phase>1||slot>=3||choice>=8{return None}
    let installed=data+0x40+slot*24;
    let old_id=if unsafe{half(installed+6)}!=0{unsafe{half(installed+8)}}else{0};
    let new_id=if phase==1{
        let row=data+0x88+choice*24;
        if unsafe{half(row+6)}!=0{unsafe{half(row+8)}}else{0}
    }else{old_id};
    let center=center_layout(s,data)?;
    let worn=crate::equipped::attachment_target_worn(s.base,data,cursor)==Some(true);
    let mut result=Content{
        points:[(0,0);32],compared:[None;32],total_before:[None;32],total_after:[None;32],count:0,
        key:(0x534f434b00000000|((phase as u64)<<28)|((slot as u64)<<24)|((old_id as u64)<<12)|new_id as u64)^((center as u64)<<32)^((worn as u64)<<33),
        preview:false,center,incompatible:false,
        projection:if !worn{Projection::UnwornDecoration}else if phase==0{Projection::CurrentOnly}else{Projection::ReplaceDecoration},
    };
    for (item,after) in [(old_id,false),(new_id,true)]{
        if let Some(deco)=DECOS.iter().find(|d|d.gu_item==item){
            for &(id,p) in deco.points{comparison::insert(&mut result.points,&mut result.compared,&mut result.count,id,p,after);}
        }
    }
    if phase==0{for p in &mut result.compared[..result.count]{*p=None;}}
    (result.count>0).then_some(result)
}
