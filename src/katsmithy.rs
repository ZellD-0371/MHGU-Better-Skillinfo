//! Optional KatSmithy integration. Discover once, then read only live selection.
use core::{ptr,sync::atomic::{AtomicBool,AtomicU32,Ordering}};
use crate::{Content,Projection,State,TREES};
static SCANNED:AtomicBool=AtomicBool::new(false);
static MODULE:AtomicU32=AtomicU32::new(0);
static SETUP:AtomicU32=AtomicU32::new(0);
static ACTIVE:AtomicBool=AtomicBool::new(false);
static MODULE_SIZE:AtomicU32=AtomicU32::new(0);
static WINDOW:AtomicU32=AtomicU32::new(0);
static INITIAL0:AtomicU32=AtomicU32::new(0);
static INITIAL1:AtomicU32=AtomicU32::new(0);
const MODE_OFFSET:u32=0x1e4a40;
// KatSmithy's own filtered set list. Unlike the native cursor, this also tracks
// pieces that cannot currently be crafted. Verified against locked torso and
// craftable arms of the same set (series 33).
const SET_IDS:u32=0x1e2664;
const SET_COUNT:u32=0x1e308c;
const SET_ROW:u32=0x1e3090;
const SET_PIECE:u32=0x1e3094;
const ARMOR:&[u8]=include_bytes!("../assets/armor-skills.bin");
#[repr(C)]#[derive(Default)]
struct Map{base:u64,size:u64,kind:u32,attr:u32,perm:u32,ipc:u32,device:u32,pad:u32}
// Horizon AArch32 SVC ABI. Recomp implements the same QueryMemory call.
#[unsafe(naked)]
unsafe extern "C" fn query(_info:*mut Map,_address:u32)->u32{
 core::arch::naked_asm!("mov r2, r1","svc #6","bx lr");
}
fn readable(address:u32,size:u32)->bool{
 let Some(end)=address.checked_add(size)else{return false};let mut at=address;
 while at<end{let mut m=Map::default();if unsafe{query(&mut m,at)}!=0||m.perm&1==0{return false}
 let next=m.base.saturating_add(m.size);if next<=at as u64{return false}at=next.min(end as u64) as u32;
 }true
}
unsafe fn word(a:u32)->u32{unsafe{ptr::read_unaligned(a as *const u32)}}
unsafe fn half(a:u32)->u16{unsafe{ptr::read_unaligned(a as *const u16)}}
pub fn discover_once(){
 if SCANNED.swap(true,Ordering::AcqRel){return}
 let own=discover_once as *const () as u32;let mut at=0u32;
 // QueryMemory skips unmapped gaps; never blindly dereference the address space.
 for _ in 0..16384{
  let mut m=Map::default();if unsafe{query(&mut m,at)}!=0{break}
  let next=m.base.saturating_add(m.size);if next<=at as u64{break}
  if m.perm&5==5&&m.base>=0x10000&&m.size>=0x80&&m.base<=u32::MAX as u64{
   let base=m.base as u32;
   if readable(base,0x80)&&unsafe{word(base+0x10)}==0x304f524e{
    let ro=unsafe{word(base+0x28)};let len=unsafe{word(base+0x2c)};
    let image=unsafe{word(base+0x18)};let bss=unsafe{word(base+0x38)};
    let total=image.checked_add(bss);
    if let Some(total)=total{if !(base..base.saturating_add(total)).contains(&own)&&len<=16*1024*1024&&len>=32{
     if let Some(start)=base.checked_add(ro){if readable(start,len){
      let bytes=unsafe{core::slice::from_raw_parts(start as *const u8,len as usize)};
      if MODULE.load(Ordering::Acquire)==0&&crate::katsmithy_model::identifies(bytes)&&MODE_OFFSET.checked_add(4).is_some_and(|n|n<=total)&&readable(base+MODE_OFFSET,4){
       MODULE_SIZE.store(total,Ordering::Release);MODULE.store(base,Ordering::Release);crate::log_info!("[better-skillinfo] KatSmithy detected at {:08X}",base);
      }
     }}
     // Wishlist publishes its detail-page view in its data segment.
     let (data,data_len)=unsafe{(word(base+0x30),word(base+0x34))};
     if !crate::wishlist_view::found()&&data_len<=16*1024*1024&&data.checked_add(data_len).is_some_and(|n|n<=total){
      if let Some(start)=base.checked_add(data){if readable(start,data_len){crate::wishlist_view::locate(start,data_len);}}
     }
    }}
   }
  }
  if MODULE.load(Ordering::Acquire)!=0&&crate::wishlist_view::found(){break}
  if next>=0x100000000{break}at=next as u32;
 }
 if MODULE.load(Ordering::Acquire)==0{crate::log_info!("[better-skillinfo] KatSmithy not detected");}
 if !crate::wishlist_view::found(){crate::log_info!("[better-skillinfo] Wishlist skill view not detected");}
 crate::log_info!("[better-skillinfo] module discovery complete");
}
/// Armor skill points of one KatSmithy/Wishlist piece: series ID and native kind 1..5.
pub fn armor_points(id:u16,kind:u16)->([(u16,i32);32],usize){
 let mut points=[(0u16,0i32);32];let mut count=0;
 let key=(id,kind);let mut lo=0usize;let mut hi=ARMOR.len()/6;
 while lo<hi{let mid=(lo+hi)/2;let r=&ARMOR[mid*6..mid*6+6];
  if (u16::from_le_bytes([r[0],r[1]]),r[2] as u16)<key{lo=mid+1}else{hi=mid}
 }
 for r in ARMOR[lo*6..].chunks_exact(6){
  if u16::from_le_bytes([r[0],r[1]])!=id||r[2] as u16!=kind{break}
  let gu=u16::from_le_bytes([r[3],r[4]]);
  if let Some(t)=TREES.iter().find(|t|t.gu_id==gu){crate::equipment_skills::add(&mut points,&mut count,t.id,r[5] as i8 as i32);}
 }
 (points,count)
}
/// One-time search for the Forge trampoline of a hook placed over KatSmithy's: its
/// copied entry words are KatSmithy's jump. Only small executable regions outside
/// every NRO and the game are read (Forge's trampoline area); run at most once.
fn kat_chained(base:u32,module:u32,size:u32)->bool{
 let back=base+0x6f1af0;let own=kat_chained as *const () as u32;let mut at=0u32;
 for _ in 0..16384{
  let mut m=Map::default();if unsafe{query(&mut m,at)}!=0{break}
  let next=m.base.saturating_add(m.size);if next<=at as u64{break}
  if m.perm&5==5&&m.base>=0x10000&&m.size>=0x20&&m.size<=0x100000&&next<=0x100000000{
   let start=m.base as u32;let end=next as u32;
   let foreign=!(start..end).contains(&base)&&!(start..end).contains(&own)&&!(start..end).contains(&module);
   if foreign&&readable(start,0x14)&&unsafe{word(start+0x10)}!=0x304f524e&&readable(start,end-start){
    let mut a=start;
    while a+16<=end{
     if unsafe{word(a)}==crate::katsmithy_model::FORGE_JUMP{
      let w=unsafe{[word(a),word(a+4),word(a+8),word(a+12)]};
      if crate::katsmithy_model::kat_trampoline(w,back,module,size){crate::log_info!("[better-skillinfo] KatSmithy jump found in trampoline {:08X}",a);return true}
     }
     a+=4;
    }
   }
  }
  if next>=0x100000000{break}at=next as u32;
 }
 false
}
#[forge::hook(offset=0x6f1ae8)]
fn smithy_update(owner:u32)->u32{
 let result=original!(owner);
 WINDOW.store(owner,Ordering::Release);result
}
/// KatSmithy installs its guarded hooks during its first onUpdate, not onInit.
/// Sample the entry at our first update, wait one complete update round regardless
/// of plugin order, and never modify its guarded entry before that. No retry or
/// repeated module scan is performed.
pub fn update(base:u32){
 let round=SETUP.load(Ordering::Acquire);if round>=2{return}
 SETUP.store(round+1,Ordering::Release);
 let entry=unsafe{[word(base+0x6f1ae8),word(base+0x6f1aec)]};
 if round==0{discover_once();INITIAL0.store(entry[0],Ordering::Release);INITIAL1.store(entry[1],Ordering::Release);return}
 let module=MODULE.load(Ordering::Acquire);if module==0{return}
 let initial=[INITIAL0.load(Ordering::Acquire),INITIAL1.load(Ordering::Acquire)];
 let size=MODULE_SIZE.load(Ordering::Acquire);
 let chained=crate::katsmithy_model::needs_chain_proof(initial,entry,module,size)&&kat_chained(base,module,size);
 if !crate::katsmithy_model::kat_hook_ready(initial,entry,module,size,chained){
  forge::log::prelude::warn!("[better-skillinfo] KatSmithy smithy hook inactive (first {:08X} {:08X}, now {:08X} {:08X}); native skill UI retained",initial[0],initial[1],entry[0],entry[1]);return
 }
 if !(entry[1]>=module&&entry[1]-module<MODULE_SIZE.load(Ordering::Acquire)){
  crate::log_info!("[better-skillinfo] KatSmithy smithy hook chained by another plugin ({:08X}); observer linked on top",entry[1]);
 }
 forge::install_hook!(base,smithy_update);
 ACTIVE.store(true,Ordering::Release);
 crate::log_info!("[better-skillinfo] KatSmithy observer linked after its initialization");
}
/// Outer None means a different screen; Some(None) hides the native fallback
/// while KatSmithy is showing the ordinary armor status instead of skills.
pub fn content(s:&State)->Option<Option<Content>>{
 let module=MODULE.load(Ordering::Acquire);let w=WINDOW.load(Ordering::Acquire);
 if !ACTIVE.load(Ordering::Acquire)||module==0||w<0x10000{return None}
 if unsafe{word(w)}!=s.base+0x177d988||unsafe{word(w+4)}!=3{return None}
 // The native kind still distinguishes the armor forge from other smithy
 // operations. It must not supply the displayed series/part for KatSmithy.
 let native_kind=unsafe{half(w+0x5c)};
 if !(1..=5).contains(&native_kind){return None}
 if !s.config.armor_enabled||unsafe{word(module+MODE_OFFSET)}!=1{return Some(None)}
 let count=unsafe{word(module+SET_COUNT)};let row=unsafe{word(module+SET_ROW)};
 let piece=unsafe{word(module+SET_PIECE)};
 if !crate::katsmithy_model::valid_cursor(count,row,piece){return Some(None)}
 let id=unsafe{half(module+SET_IDS+row*2)};let kind=(piece+1) as u16;
 if id==0{return Some(None)}
 let (points,count)=armor_points(id,kind);
 if count==0{return Some(None)}
 Some(Some(Content{points,compared:[None;32],total_before:[None;32],total_after:[None;32],count,
 key:0x4b53000000000000|((kind as u64)<<16)|id as u64,preview:false,center:false,incompatible:false,projection:Projection::ProductionEquipment}))
}
