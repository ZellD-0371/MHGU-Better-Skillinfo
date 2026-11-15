//! Optional KatSmithy integration. Discover once, then read only live selection.
use core::{ptr,sync::atomic::{AtomicBool,AtomicU32,Ordering}};
use crate::{Content,Projection,State,TREES};
static SCANNED:AtomicBool=AtomicBool::new(false);
static MODULE:AtomicU32=AtomicU32::new(0);
static SETUP:AtomicU32=AtomicU32::new(0);
static ACTIVE:AtomicBool=AtomicBool::new(false);
static MODULE_SIZE:AtomicU32=AtomicU32::new(0);
static WINDOW:AtomicU32=AtomicU32::new(0);
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
      if crate::katsmithy_model::identifies(bytes)&&MODE_OFFSET.checked_add(4).is_some_and(|n|n<=total)&&readable(base+MODE_OFFSET,4){
       MODULE_SIZE.store(total,Ordering::Release);MODULE.store(base,Ordering::Release);crate::log_info!("[better-skillinfo] KatSmithy detected at {:08X}",base);return
      }
     }}
    }}
   }
  }
  if next>=0x100000000{break}at=next as u32;
 }
 crate::log_info!("[better-skillinfo] KatSmithy not detected; discovery complete");
}
#[forge::hook(offset=0x6f1ae8)]
fn smithy_update(owner:u32)->u32{
 let result=original!(owner);
 WINDOW.store(owner,Ordering::Release);result
}
/// KatSmithy installs its guarded hooks during its first onUpdate, not onInit.
/// Wait one complete update round regardless of plugin order; never modify its
/// guarded entry before that. No retry or repeated module scan is performed.
pub fn update(base:u32){
 let round=SETUP.load(Ordering::Acquire);if round>=2{return}
 SETUP.store(round+1,Ordering::Release);
 if round==0{discover_once();return}
 let module=MODULE.load(Ordering::Acquire);if module==0{return}
 let a=unsafe{word(base+0x6f1ae8)};let target=unsafe{word(base+0x6f1aec)};
 if !crate::katsmithy_model::kat_hook_ready(a,target,module,MODULE_SIZE.load(Ordering::Acquire)){
  forge::log::prelude::warn!("[better-skillinfo] KatSmithy smithy hook inactive; native skill UI retained");return
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
 if count==0{return Some(None)}
 Some(Some(Content{points,compared:[None;32],total_before:[None;32],total_after:[None;32],count,
 key:0x4b53000000000000|((kind as u64)<<16)|id as u64,preview:false,center:false,incompatible:false,projection:Projection::ProductionEquipment}))
}
