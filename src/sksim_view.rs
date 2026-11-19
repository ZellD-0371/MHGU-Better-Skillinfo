//! Skill-Simulator's window (its own ImGui window). Skill-Simulator publishes a small
//! versioned block; it is located once during module discovery and only read.
use core::{cell::UnsafeCell,ptr,sync::atomic::{AtomicU32,Ordering}};
use crate::{Content,Projection,State,TREES,generated::DECOS,sksim_model as m};
static VIEW:AtomicU32=AtomicU32::new(0);
static VERSION:AtomicU32=AtomicU32::new(0);
// Only the UI thread (forge_onImGuiFreeRender and Wishlist's callback) reads the block.
struct Frame{live:m::Liveness,now:u32,sim:Option<m::Sim>}
struct Shared(UnsafeCell<Frame>);unsafe impl Sync for Shared{}
static LAST:Shared=Shared(UnsafeCell::new(Frame{live:m::Liveness::NEW,now:u32::MAX,sim:None}));
pub fn found()->bool{VIEW.load(Ordering::Acquire)!=0}
pub fn locate(start:u32,len:u32){
 let bytes=unsafe{core::slice::from_raw_parts(start as *const u8,len as usize)};
 if let Some((at,version))=m::find(bytes){
  let a=start+at as u32;VERSION.store(version,Ordering::Release);VIEW.store(a,Ordering::Release);
  crate::log_info!("[better-skillinfo] Skill-Simulator block v{} detected at {:08X}",version,a);
 }
}
/// The open Skill-Simulator window this frame (`now`: our frame counter), read once per frame.
pub fn current(now:u32)->Option<m::Sim>{
 let v=VIEW.load(Ordering::Acquire);if v==0{return None}
 let last=unsafe{&mut *LAST.0.get()};
 if last.now==now{return last.sim}
 let version=VERSION.load(Ordering::Acquire);let size=m::size(version);
 let mut bytes=[0u8;m::SIZE_V3];
 for (n,b) in bytes[..size].iter_mut().enumerate(){*b=unsafe{ptr::read_volatile((v as usize+n) as *const u8)};}
 let frame=u32::from_le_bytes([bytes[m::FRAME],bytes[m::FRAME+1],bytes[m::FRAME+2],bytes[m::FRAME+3]]);
 let sim=m::parse(&bytes[..size],version).filter(|_|last.live.alive(frame,now));
 last.now=now;last.sim=sim;sim
}
fn thresholds(id:u16)->([i32;16],usize){
 let mut out=[0;16];let mut n=0;
 if let Some(t)=TREES.iter().find(|t|t.id==id){for e in t.effects.iter().take(16){out[n]=e.points;n+=1;}}
 (out,n)
}
/// Activated skills first; Skill-Simulator's order is kept within both groups.
fn activated_first(points:&mut [(u16,i32);32],count:usize){
 let on=|&(id,v):&(u16,i32)|{let (t,n)=thresholds(id);m::activated(v,&t[..n])};
 let mut out=[(0u16,0i32);32];let mut n=0;
 for p in points[..count].iter().filter(|p|on(p)){out[n]=*p;n+=1;}
 for p in points[..count].iter().filter(|p|!on(p)){out[n]=*p;n+=1;}
 points[..count].copy_from_slice(&out[..count]);
}
fn add_gu(points:&mut [(u16,i32);32],count:&mut usize,gu:u16,value:i32){
 if let Some(t)=TREES.iter().find(|t|t.gu_id==gu){crate::equipment_skills::add(points,count,t.id,value);}
}
/// Outer None: no version-2 Skill-Simulator window. Some(None): it is open and covers
/// the game screens underneath, but the cursor's subject shows no panel.
pub fn content(s:&State)->Option<Option<Content>>{
 let sim=current(s.frame).filter(|sim|sim.version>=2)?;
 if !m::shows_panel(&sim){return Some(None)}
 let mut points=[(0u16,0i32);32];let mut count=0;
 let projection=match sim.subject{
  // A nonzero value on a simulator skill names the single threshold to show.
  m::Subject::Skill{tree,threshold}=>{let Some(t)=TREES.iter().find(|t|t.gu_id==tree) else{return Some(None)};points[0]=(t.id,threshold as i32);count=1;Projection::SimulatorSkill}
  m::Subject::Armor{series,part}=>{
   // Version 3 writes the piece's own skills in its display order; version 2 only
   // its decorations, so the piece's own skills come from our armor table.
   if sim.version<3{let (p,n)=crate::katsmithy::armor_points(series,part as u16);points=p;count=n;}
   for &(gu,v) in &sim.points[..sim.count]{add_gu(&mut points,&mut count,gu,v as i32);}
   Projection::SimulatorEquipment
  }
  m::Subject::Charm|m::Subject::Weapon=>{for &(gu,v) in &sim.points[..sim.count]{add_gu(&mut points,&mut count,gu,v as i32);}Projection::SimulatorEquipment}
  m::Subject::Set=>{
   for &(gu,v) in &sim.points[..sim.count]{add_gu(&mut points,&mut count,gu,v as i32);}
   activated_first(&mut points,count);Projection::SimulatorSet
  }
  m::Subject::Decoration(item)=>{
   let Some(d)=DECOS.iter().find(|d|d.gu_item==item) else{return Some(None)};
   for &(id,v) in d.points{crate::equipment_skills::add(&mut points,&mut count,id,v);}Projection::SimulatorEquipment
  }
  m::Subject::None=>return Some(None),
 };
 if count==0{return Some(None)}
 Some(Some(Content{points,compared:[None;32],total_before:[None;32],total_after:[None;32],count,
  key:0x53494d0000000000|((sim.screen as u64)<<32)|sim.serial as u64,preview:false,center:false,incompatible:false,projection}))
}
