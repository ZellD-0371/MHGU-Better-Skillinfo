//! Pure parts of the Skill-Simulator link (headless-tested).
//! Skill-Simulator writes a small versioned block every frame its window is drawn;
//! Better Skillinfo only reads it. Version 1 carries the window frame; version 2 adds
//! the screen, the cursor's column and what the cursor points at
//! (docs/skill-simulator-link-v2.md). Version 3 keeps that layout and refines what is
//! written: an armor piece carries all of its points in Skill-Simulator's own order,
//! a charm includes its decorations, the weapon row has its own subject and a skill may
//! name a single threshold (also inside a modal); it appends the detail column's page
//! (skills or crafting materials). Better Skillinfo decides what to show
//! and where.
pub const MAGIC:[u8;16]=*b"MHGU-SKSIM-WIN01";
/// Byte offsets (repr(C), little endian). Version 1 is the first 44 bytes.
pub const OPEN:usize=20;pub const LEFT:usize=24;pub const TOP:usize=28;pub const RIGHT:usize=32;pub const BOTTOM:usize=36;pub const FRAME:usize=40;
pub const SIZE_V1:usize=44;
pub const SCREEN:usize=44;pub const AREA:usize=48;pub const SUBJECT:usize=52;pub const A:usize=56;pub const B:usize=60;
pub const COUNT:usize=64;pub const POINTS:usize=68;pub const RECTS:usize=196;pub const RECT:usize=200;pub const SERIAL:usize=320;
pub const SIZE_V2:usize=324;
/// Version 3 appends the detail column's page (`detail` below).
pub const PAGE:usize=324;pub const SIZE_V3:usize=328;
/// Detail column pages: 0 none, 1 skills/stats, 2 crafting materials.
pub mod detail{#[allow(dead_code)]pub const NONE:u32=0;#[allow(dead_code)]pub const SKILLS:u32=1;pub const MATERIALS:u32=2;}
pub const MAX_POINTS:usize=32;pub const MAX_RECTS:usize=6;
/// Our frames without a new Skill-Simulator frame before its window counts as closed
/// (it stopped drawing without clearing `open`).
pub const STALE_FRAMES:u32=10;
pub fn size(version:u32)->usize{match version{1=>SIZE_V1,2=>SIZE_V2,_=>SIZE_V3}}
fn w32(b:&[u8],o:usize)->u32{u32::from_le_bytes([b[o],b[o+1],b[o+2],b[o+3]])}
fn f32_at(b:&[u8],o:usize)->f32{f32::from_bits(w32(b,o))}
/// A 4-aligned block of a supported version in Skill-Simulator's data segment:
/// (offset, version).
pub fn find(data:&[u8])->Option<(usize,u32)>{
 (0..data.len().saturating_sub(SIZE_V1-1)).step_by(4).find_map(|i|{
  if data[i..i+16]!=MAGIC{return None}
  let v=w32(data,i+16);(matches!(v,1..=3)&&i+size(v)<=data.len()).then_some((i,v))
 })
}
/// Skill-Simulator's window frame (or one of its columns) in screen pixels.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Window{pub left:f32,pub top:f32,pub right:f32,pub bottom:f32}
fn rect(l:f32,t:f32,r:f32,b:f32)->Option<Window>{
 ([l,t,r,b].iter().all(|v|v.is_finite())&&r>l&&b>t).then_some(Window{left:l,top:t,right:r,bottom:b})
}
pub fn window(open:u32,left:f32,top:f32,right:f32,bottom:f32)->Option<Window>{
 if open!=1{return None}rect(left,top,right,bottom)
}
/// Detects a frozen block: `frame` must keep advancing while the window is open.
#[derive(Clone,Copy,Default)]
pub struct Liveness{frame:u32,changed_at:u32,seen:bool}
impl Liveness{
 pub const NEW:Self=Self{frame:0,changed_at:0,seen:false};
 pub fn alive(&mut self,frame:u32,now:u32)->bool{
  if !self.seen||frame!=self.frame{self.seen=true;self.frame=frame;self.changed_at=now;return true}
  now.wrapping_sub(self.changed_at)<=STALE_FRAMES
 }
}
/// Automatic fit into the free band left of the window: (x, y, width, max height) in
/// screen pixels. None when that band is narrower than `min_width`.
pub fn auto_fit(w:Window,margin:f32,min_width:f32,max_width:f32)->Option<(f32,f32,f32,f32)>{
 let width=(w.left-margin*2.).min(max_width);
 (width>=min_width).then_some((margin,w.top,width,w.bottom-w.top))
}

/// Screen numbers (fixed; independent of Skill-Simulator's enums). The whole table is
/// kept as the block's documentation even where the rules do not name a value.
#[allow(dead_code)]
pub mod screen{pub const NONE:u32=0;pub const MAIN:u32=1;pub const SKILLS:u32=2;pub const RESULTS:u32=3;pub const SAVED:u32=4;pub const MANAGE:u32=5;pub const BOX:u32=6;pub const MODAL:u32=7;}
/// Column numbers, shared by the cursor field and the column rectangles.
#[allow(dead_code)]
pub mod area{pub const NONE:u32=0;pub const TARGETS:u32=1;pub const OPTIONS:u32=2;pub const FIXED:u32=3;pub const SETS:u32=4;pub const DETAIL:u32=5;pub const LIST:u32=6;}
/// What the cursor points at. IDs are the game's own (armor-skills.csv scheme).
#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Subject{
 None,
 /// Skill tree (GU tree ID): its thresholds and effects; `threshold` (version 3, 0 =
 /// none) names the one effect the cursor's row stands for.
 Skill{tree:u16,threshold:i16},
 /// One armor piece (series, part 1..5). Version 2: `points` holds its decorations'
 /// points only. Version 3: its own skills in Skill-Simulator's order, then decorations.
 Armor{series:u16,part:u8},
 /// A charm; `points` holds its skills (version 3: then its decorations).
 Charm,
 /// The weapon row (version 3); `points` holds its decorations' points.
 Weapon,
 /// A whole set; `points` holds the final totals.
 Set,
 /// A decoration (item ID).
 Decoration(u16),
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Sim{
 pub window:Window,pub version:u32,pub screen:u32,pub area:u32,pub subject:Subject,
 /// (GU tree ID, points); zero IDs and zero points are dropped.
 pub points:[(u16,i16);MAX_POINTS],pub count:usize,
 pub rects:[(u32,Window);MAX_RECTS],pub rect_count:usize,pub serial:u32,
 /// Detail column page (version 3; 0 before).
 pub page:u32,
}
const EMPTY_RECT:(u32,Window)=(0,Window{left:0.,top:0.,right:0.,bottom:0.});
/// One frame's block (`b` holds `size(version)` bytes). None while the window is closed.
pub fn parse(b:&[u8],version:u32)->Option<Sim>{
 if b.len()<size(version){return None}
 let window=window(w32(b,OPEN),f32_at(b,LEFT),f32_at(b,TOP),f32_at(b,RIGHT),f32_at(b,BOTTOM))?;
 let mut sim=Sim{window,version,screen:screen::NONE,area:area::NONE,subject:Subject::None,points:[(0,0);MAX_POINTS],count:0,rects:[EMPTY_RECT;MAX_RECTS],rect_count:0,serial:0,page:0};
 if version<2{return Some(sim)}
 sim.screen=w32(b,SCREEN);sim.area=w32(b,AREA);sim.serial=w32(b,SERIAL);
 if version>=3{sim.page=w32(b,PAGE);}
 let (a,bv)=(w32(b,A),w32(b,B));
 sim.subject=match w32(b,SUBJECT){
  1 if (1..=0xffff).contains(&a)=>Subject::Skill{tree:a as u16,threshold:if version>=3{bv as i32 as i16}else{0}},
  2 if (1..=0xffff).contains(&a)&&(1..=5).contains(&bv)=>Subject::Armor{series:a as u16,part:bv as u8},
  3=>Subject::Charm,4=>Subject::Set,
  5 if (1..=0xffff).contains(&a)=>Subject::Decoration(a as u16),
  6 if version>=3=>Subject::Weapon,
  _=>Subject::None,
 };
 for n in 0..(w32(b,COUNT) as usize).min(MAX_POINTS){
  let o=POINTS+n*4;let (id,p)=(u16::from_le_bytes([b[o],b[o+1]]),i16::from_le_bytes([b[o+2],b[o+3]]));
  if id!=0&&p!=0{sim.points[sim.count]=(id,p);sim.count+=1;}
 }
 for n in 0..(w32(b,RECTS) as usize).min(MAX_RECTS){
  let o=RECT+n*20;
  if let Some(r)=rect(f32_at(b,o+4),f32_at(b,o+8),f32_at(b,o+12),f32_at(b,o+16)){sim.rects[sim.rect_count]=(w32(b,o),r);sim.rect_count+=1;}
 }
 Some(sim)
}
/// The panel's rules for Skill-Simulator (version 2). Content follows the cursor's
/// subject; these screens and columns never show a panel.
pub fn shows_panel(s:&Sim)->bool{
 if s.version<2||s.screen==screen::NONE||s.area==area::OPTIONS{return false}
 // The detail column turned to crafting materials: nothing about skills is on screen.
 if s.page==detail::MATERIALS{return false}
 // Version 3 modals (the extra-skills list) may point at a skill; any other modal hides.
 if s.screen==screen::MODAL&&!(s.version>=3&&matches!(s.subject,Subject::Skill{..})){return false}
 match s.subject{Subject::None=>false,Subject::Charm|Subject::Set|Subject::Weapon=>s.count>0,_=>true}
}
/// A set total reaches a threshold of its own sign.
pub fn activated(points:i32,thresholds:&[i32])->bool{
 thresholds.iter().any(|&t|t!=0&&(t>0)==(points>0)&&points.abs()>=t.abs())
}
/// The one threshold a set total shows: the strongest it reaches, otherwise the next
/// one of its sign. None when its sign has no thresholds.
pub fn set_threshold(points:i32,thresholds:&[i32])->Option<i32>{
 let same=thresholds.iter().copied().filter(|&t|t!=0&&(t>0)==(points>0));
 let reached=same.clone().filter(|t|points.abs()>=t.abs()).max_by_key(|t|t.abs());
 reached.or_else(||same.min_by_key(|t|t.abs()))
}
/// When the band left of the window is too narrow: the column to cover instead, the
/// one beside the cursor that the user is not working in. Full-width lists have none.
pub fn cover_area(cursor:u32)->Option<u32>{
 match cursor{area::TARGETS=>Some(area::FIXED),area::FIXED=>Some(area::TARGETS),area::SETS=>Some(area::DETAIL),area::DETAIL=>Some(area::SETS),_=>None}
}
pub fn area_rect(s:&Sim,id:u32)->Option<Window>{s.rects[..s.rect_count].iter().find(|r|r.0==id).map(|r|r.1)}

#[cfg(test)]mod tests{
 use super::*;
 extern crate std;use std::vec::Vec;
 fn put(b:&mut [u8],o:usize,v:u32){b[o..o+4].copy_from_slice(&v.to_le_bytes());}
 fn putf(b:&mut [u8],o:usize,v:f32){put(b,o,v.to_bits());}
 fn block(version:u32)->Vec<u8>{
  let mut b=std::vec![0u8;size(version)];b[..16].copy_from_slice(&MAGIC);put(&mut b,16,version);
  put(&mut b,OPEN,1);putf(&mut b,LEFT,384.);putf(&mut b,TOP,115.);putf(&mut b,RIGHT,1844.);putf(&mut b,BOTTOM,965.);b
 }
 fn v2(screen:u32,area:u32,subject:u32,a:u32,bv:u32,points:&[(u16,i16)])->Vec<u8>{
  let mut b=block(2);for (o,v) in [(SCREEN,screen),(AREA,area),(SUBJECT,subject),(A,a),(B,bv),(COUNT,points.len() as u32)]{put(&mut b,o,v);}
  for (n,(id,p)) in points.iter().enumerate(){let o=POINTS+n*4;b[o..o+2].copy_from_slice(&id.to_le_bytes());b[o+2..o+4].copy_from_slice(&p.to_le_bytes());}
  for (n,(id,l,r)) in [(1u32,420f32,980f32),(3,1050.,1610.)].iter().enumerate(){let o=RECT+n*20;put(&mut b,o,*id);putf(&mut b,o+4,*l);putf(&mut b,o+8,190.);putf(&mut b,o+12,*r);putf(&mut b,o+16,830.);}
  // Room for version 3's page field; parse reads only what a version needs.
  put(&mut b,RECTS,2);put(&mut b,SERIAL,7);b.resize(SIZE_V3,0);b
 }
 #[test]fn finds_both_versions(){
  for v in [1,2,3]{let mut d=std::vec![0u8;64];d.extend(block(v));d.extend([0u8;8]);assert_eq!(find(&d),Some((64,v)));}
 }
 #[test]fn rejects_unknown_versions_truncation_and_misalignment(){
  let mut d=std::vec![0u8;64];let mut b=block(2);put(&mut b,16,4);d.extend(b);assert_eq!(find(&d),None);
  let b=block(2);assert_eq!(find(&b[..SIZE_V2-1]),None,"version 2 needs its whole block");
  let b=block(3);assert_eq!(b.len(),SIZE_V3);assert_eq!(find(&b[..SIZE_V3-1]),None,"version 3 needs its whole block");
  let mut odd=std::vec![0u8;66];odd.extend(block(1));assert_eq!(find(&odd),None,"only 4-aligned blocks");
 }
 #[test]fn closed_or_bad_geometry_is_no_window(){
  assert_eq!(window(0,300.,6.,1250.,714.),None);
  assert_eq!(window(1,f32::NAN,6.,1250.,714.),None);
  assert_eq!(window(1,1250.,6.,300.,714.),None);
  assert_eq!(window(1,300.,714.,1250.,6.),None);
  assert_eq!(window(1,300.,6.,1250.,714.),Some(Window{left:300.,top:6.,right:1250.,bottom:714.}));
 }
 #[test]fn frozen_frame_counter_closes_the_window(){
  let mut l=Liveness::default();
  assert!(l.alive(5,100));
  for now in 101..=110{assert!(l.alive(5,now),"within {STALE_FRAMES} frames");}
  assert!(!l.alive(5,111));
  assert!(l.alive(6,112),"a new frame revives it");
 }
 #[test]fn counters_wrap_around(){let mut l=Liveness::default();assert!(l.alive(u32::MAX,u32::MAX-1));assert!(l.alive(0,3));assert!(l.alive(0,13));assert!(!l.alive(0,14));}
 // Skill-Simulator at 0.8 size, right-aligned on 1280x720: about 230 px free on the left.
 #[test]fn auto_fit_uses_band_left_of_window(){
  let w=Window{left:246.,top:78.,right:1250.,bottom:642.};
  assert_eq!(auto_fit(w,8.,160.,500.),Some((8.,78.,230.,564.)));
  assert_eq!(auto_fit(Window{left:1000.,..w},8.,160.,500.).unwrap().2,500.,"capped on wide screens");
 }
 #[test]fn narrow_band_falls_back(){let w=Window{left:150.,top:6.,right:1250.,bottom:714.};assert_eq!(auto_fit(w,8.,160.,500.),None);}
 #[test]fn version_one_has_window_only(){let s=parse(&block(1),1).unwrap();assert_eq!((s.screen,s.subject,s.count),(0,Subject::None,0));assert!(!shows_panel(&s));}
 #[test]fn parses_subjects(){
  assert_eq!(parse(&v2(1,1,1,42,0,&[]),2).unwrap().subject,Subject::Skill{tree:42,threshold:0});
  assert_eq!(parse(&v2(1,1,1,42,10,&[]),2).unwrap().subject,Subject::Skill{tree:42,threshold:0},"version 2 has no threshold");
  assert_eq!(parse(&v2(7,0,1,42,(-10i32) as u32,&[]),3).unwrap().subject,Subject::Skill{tree:42,threshold:-10});
  assert_eq!(parse(&v2(3,5,6,0,0,&[(42,1)]),3).unwrap().subject,Subject::Weapon);
  assert_eq!(parse(&v2(3,5,6,0,0,&[(42,1)]),2).unwrap().subject,Subject::None,"weapon row is version 3");
  let s=parse(&v2(1,3,2,33,2,&[(48,1),(0,5),(9,0)]),2).unwrap();
  assert_eq!(s.subject,Subject::Armor{series:33,part:2});assert_eq!(&s.points[..s.count],&[(48,1)],"zero IDs and points dropped");
  assert_eq!(parse(&v2(3,4,4,0,0,&[(42,3),(1,-2)]),2).unwrap().count,2);
  assert_eq!(parse(&v2(6,6,5,2638,0,&[]),2).unwrap().subject,Subject::Decoration(2638));
  for (kind,a,b) in [(2,33,6),(2,0,2),(1,0,0),(9,1,1)]{assert_eq!(parse(&v2(1,3,kind,a,b,&[]),2).unwrap().subject,Subject::None,"{kind} {a} {b}");}
  let s=parse(&v2(1,1,1,42,0,&[]),2).unwrap();assert_eq!(s.serial,7);assert_eq!(area_rect(&s,3).unwrap().left,1050.);assert_eq!(area_rect(&s,5),None);
 }
 #[test]fn rules_follow_subject_and_hide_modal_and_options(){
  let show=|screen,area,kind,a,b,p:&[(u16,i16)]|shows_panel(&parse(&v2(screen,area,kind,a,b,p),2).unwrap());
  assert!(show(screen::MAIN,area::TARGETS,1,42,0,&[]));
  assert!(!show(screen::MAIN,area::OPTIONS,1,42,0,&[]),"search options never show a panel");
  assert!(show(screen::MAIN,area::FIXED,2,33,2,&[]));
  assert!(!show(screen::MAIN,area::FIXED,0,0,0,&[]),"\"not fixed\" shows nothing");
  assert!(show(screen::RESULTS,area::SETS,4,0,0,&[(42,3)]));
  assert!(!show(screen::RESULTS,area::SETS,4,0,0,&[]),"empty totals");
  assert!(!show(screen::MODAL,area::DETAIL,2,33,2,&[]),"modal hides");
  assert!(!show(screen::NONE,area::LIST,1,42,0,&[]));
  assert!(!show(screen::MODAL,area::NONE,1,42,10,&[]),"version 2 modal hides even a skill");
  let show3=|screen,area,kind,a,b,p:&[(u16,i16)]|shows_panel(&parse(&v2(screen,area,kind,a,b,p),3).unwrap());
  assert!(show3(screen::MODAL,area::NONE,1,42,10,&[]),"extra-skills list");
  assert!(!show3(screen::MODAL,area::DETAIL,2,33,2,&[(42,2)]),"other modals hide");
  assert!(!show3(screen::RESULTS,area::DETAIL,6,0,0,&[]),"weapon without decorations");
  assert!(show3(screen::RESULTS,area::DETAIL,6,0,0,&[(42,1)]));
  assert!(show3(screen::RESULTS,area::SETS,3,2,0,&[(42,1)]),"charm with decorations");
  let mut b=v2(screen::RESULTS,area::DETAIL,2,33,2,&[(42,2)]);put(&mut b,16,3);
  b.resize(SIZE_V3,0);assert!(shows_panel(&parse(&b,3).unwrap()),"skills page");
  put(&mut b,PAGE,detail::MATERIALS);assert!(!shows_panel(&parse(&b,3).unwrap()),"materials page hides");
  assert_eq!(parse(&b[..SIZE_V2],3),None,"version 3 needs its page field");
 }
 #[test]fn set_totals_show_one_threshold(){
  let ear=[10,15];let atk=[10,15,20];let stamina=[-10,10,15];
  assert_eq!(set_threshold(15,&ear),Some(15));assert_eq!(set_threshold(14,&ear),Some(10));
  assert_eq!(set_threshold(20,&atk),Some(20));assert_eq!(set_threshold(2,&atk),Some(10),"next one when not reached");
  assert_eq!(set_threshold(-1,&stamina),Some(-10));assert_eq!(set_threshold(-12,&stamina),Some(-10));
  assert_eq!(set_threshold(-3,&atk),None,"no negative thresholds");
  assert!(activated(15,&ear));assert!(!activated(9,&ear));assert!(activated(-10,&stamina));assert!(!activated(-9,&stamina));assert!(!activated(12,&[-10]));
 }
 #[test]fn narrow_band_covers_the_other_column(){
  assert_eq!(cover_area(area::TARGETS),Some(area::FIXED));assert_eq!(cover_area(area::FIXED),Some(area::TARGETS));
  assert_eq!(cover_area(area::SETS),Some(area::DETAIL));assert_eq!(cover_area(area::DETAIL),Some(area::SETS));
  assert_eq!(cover_area(area::LIST),None);
 }
}
