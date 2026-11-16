//! Wishlist's detail page 2 (its own ImGui window). Wishlist publishes a small
//! versioned block; it is located once during module discovery and only read.
use core::{ptr,sync::atomic::{AtomicU32,Ordering}};
use crate::{Content,Projection,State,wishlist_model as m};
static VIEW:AtomicU32=AtomicU32::new(0);
pub fn found()->bool{VIEW.load(Ordering::Acquire)!=0}
pub fn locate(start:u32,len:u32){
 let bytes=unsafe{core::slice::from_raw_parts(start as *const u8,len as usize)};
 if let Some(at)=m::find(bytes){
  let a=start+at as u32;VIEW.store(a,Ordering::Release);
  // Wishlist calls this right after drawing its window (see wishlist_model::DRAW).
  unsafe{ptr::write_volatile((a+m::DRAW) as *mut u32,overlay as extern "C" fn() as usize as u32);}
  crate::log_info!("[better-skillinfo] Wishlist skill view detected at {:08X}",a);
 }
}
/// Called by Wishlist on the UI thread after its own foreground drawing.
extern "C" fn overlay(){crate::panel::render_wishlist();}
fn word(view:u32,offset:u32)->u32{unsafe{ptr::read_volatile((view+offset) as *const u32)}}
fn float(view:u32,offset:u32)->f32{f32::from_bits(word(view,offset))}
pub fn piece()->Option<Option<m::Piece>>{
 let v=VIEW.load(Ordering::Acquire);if v==0{return None}
 m::piece(word(v,m::OPEN),word(v,m::ACTIVE),word(v,m::KIND),word(v,m::ID),float(v,m::RIGHT),float(v,m::TOP),float(v,m::BOTTOM))
}
/// Outer None: Wishlist's window is not on screen. Some(None): it is, but not on an
/// armor detail page 2; other screens' panels underneath stay hidden meanwhile.
pub fn content(s:&State)->Option<Option<Content>>{
 let piece=piece()?;
 let Some(p)=piece else{return Some(None)};
 if !s.config.armor_enabled{return Some(None)}
 let (points,count)=crate::katsmithy::armor_points(p.id,p.kind);
 if count==0{return Some(None)}
 Some(Some(Content{points,compared:[None;32],total_before:[None;32],total_after:[None;32],count,
  key:0x574c000000000000|((p.kind as u64)<<16)|p.id as u64,preview:false,center:false,incompatible:false,projection:Projection::WishlistEquipment}))
}
