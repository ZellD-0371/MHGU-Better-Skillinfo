fn has(ro:&[u8],s:&[u8])->bool{ro.windows(s.len()).any(|w|w==s)}
/// Log prefixes of our own plugins. They also carry KatSmithy's marker strings
/// (to detect it), so a module holding any of them is never KatSmithy itself.
const OUR_PREFIXES:[&[u8];4]=[b"[better-skillinfo]",b"[korean-",b"[wishlist]",b"[skill-sim"];
pub fn identifies(ro:&[u8])->bool{
 [b"[KatSmithy]".as_slice(),b"katsmithy.ini".as_slice(),b"Skill points by piece".as_slice()]
 .iter().all(|s|has(ro,s))&&!OUR_PREFIXES.iter().any(|p|has(ro,p))
}
#[cfg(test)]mod tests{
 use super::*;
 extern crate std;
 #[test]fn module_identity_requires_all_markers(){assert!(identifies(b"[KatSmithy] katsmithy.ini Skill points by piece"));assert!(!identifies(b"[KatSmithy]"));assert!(!identifies(b"katsmithy.ini"));}
 // Ryujinx log without KatSmithy: our other plugins hold the same markers and were
 // mistaken for it. Any of our log prefixes rules a module out.
 #[test]fn our_plugins_are_not_katsmithy(){
  for p in ["[better-skillinfo]","[korean-patch]","[wishlist]","[skill-sim]"]{
   let ro=std::format!("[KatSmithy] katsmithy.ini Skill points by piece {p} ready");
   assert!(!identifies(ro.as_bytes()),"{p}");
  }
  assert!(identifies(b"\0[KatSmithy] loaded\0katsmithy.ini\0Skill points by piece\0"));
 }
}

/// The smithy updater's first two words in MHGU 1.4.0; KatSmithy's guard expects them.
pub const SMITHY_ENTRY:[u32;2]=[0xe92d41f0,0xe24dd018];
/// Forge's hook form: `ldr pc, [pc, #-4]; .word detour`.
pub const FORGE_JUMP:u32=0xe51ff004;
fn inside(target:u32,module:u32,size:u32)->bool{module!=0&&target&3==0&&target.checked_sub(module).is_some_and(|n|n<size)}
/// Sampled at the first update, after every plugin's onInit. KatSmithy installs its
/// guarded hook in its first update, so it can succeed only if the entry is still
/// the original here, or it already ran this frame and the jump is its own.
pub fn kat_can_hook(entry:[u32;2],module:u32,size:u32)->bool{
 entry==SMITHY_ENTRY||(entry[0]==FORGE_JUMP&&inside(entry[1],module,size))
}
/// Second update: KatSmithy's guard passed, so the entry now carries a Forge jump.
/// Other plugins may chain on top of KatSmithy in the same frame (their detour then
/// owns the second word); that keeps KatSmithy in the chain and must stay accepted.
/// `kat_chained`: a Forge trampoline holding KatSmithy's jump was found, which proves
/// KatSmithy hooked first even when our first sample came after the chaining.
pub fn kat_hook_ready(initial:[u32;2],current:[u32;2],module:u32,size:u32,kat_chained:bool)->bool{
 module!=0&&current[0]==FORGE_JUMP&&current[1]&3==0&&current[1]>=0x10000&&(kat_can_hook(initial,module,size)||kat_chained)
}
/// Whether the entry already proves KatSmithy without looking for a trampoline.
pub fn needs_chain_proof(initial:[u32;2],current:[u32;2],module:u32,size:u32)->bool{
 current[0]==FORGE_JUMP&&!kat_can_hook(initial,module,size)&&!inside(current[1],module,size)
}
/// Forge's trampoline for a hook placed over KatSmithy's: the two copied entry words
/// (KatSmithy's jump) followed by a jump back to the entry + 8.
pub fn kat_trampoline(words:[u32;4],jump_back:u32,module:u32,size:u32)->bool{
 words[0]==FORGE_JUMP&&inside(words[1],module,size)&&words[2]==FORGE_JUMP&&words[3]==jump_back
}
#[cfg(test)]mod initialization_tests{
 use super::*;
 const KAT:u32=0x3440000;const SIZE:u32=0x488000;
 const BACK:u32=0x200000+0x6f1af0;
 #[test]fn loaded_but_unhooked_keeps_native_ui(){assert!(!kat_hook_ready(SMITHY_ENTRY,SMITHY_ENTRY,KAT,SIZE,false));}
 #[test]fn kat_hook_enables_observer(){assert!(kat_hook_ready(SMITHY_ENTRY,[FORGE_JUMP,0x3479964],KAT,SIZE,false));}
 // Recomp log: KatSmithy hooked -> 03479964, then a later plugin chained -> 0390e2b0 in f=1.
 #[test]fn plugin_chained_after_kat_keeps_observer(){assert!(kat_hook_ready(SMITHY_ENTRY,[FORGE_JUMP,0x390e2b0],KAT,SIZE,false));}
 // KatSmithy updated before us and already hooked: also fine, with or without a chain.
 #[test]fn kat_ran_first_is_accepted(){let k=[FORGE_JUMP,0x3479964];assert!(kat_hook_ready(k,k,KAT,SIZE,false));assert!(kat_hook_ready(k,[FORGE_JUMP,0x390e2b0],KAT,SIZE,false));}
 // We update after both KatSmithy and the chaining plugin: the first sample is already
 // foreign, so only a trampoline holding KatSmithy's jump can prove it.
 #[test]fn late_sample_needs_trampoline_proof(){
  let f=[FORGE_JUMP,0x390e2b0];
  assert!(needs_chain_proof(f,f,KAT,SIZE));
  assert!(!kat_hook_ready(f,f,KAT,SIZE,false));
  assert!(kat_trampoline([FORGE_JUMP,0x3479964,FORGE_JUMP,BACK],BACK,KAT,SIZE));
  assert!(kat_hook_ready(f,f,KAT,SIZE,true));
 }
 // Taken by another plugin before KatSmithy: its guard fails and no trampoline holds
 // KatSmithy's jump (the copied words are the original prologue), keep the native UI.
 #[test]fn entry_taken_before_kat_keeps_native_ui(){
  let f=[FORGE_JUMP,0x337fee4];assert!(!kat_can_hook(f,KAT,SIZE));
  assert!(!kat_trampoline([SMITHY_ENTRY[0],SMITHY_ENTRY[1],FORGE_JUMP,BACK],BACK,KAT,SIZE));
  assert!(!kat_hook_ready(f,f,KAT,SIZE,false));
 }
 #[test]fn trampoline_must_return_to_smithy_entry(){assert!(!kat_trampoline([FORGE_JUMP,0x3479964,FORGE_JUMP,BACK+4],BACK,KAT,SIZE));}
 #[test]fn missing_module_is_rejected(){assert!(!kat_hook_ready(SMITHY_ENTRY,[FORGE_JUMP,0x3479964],0,SIZE,true));}
}

/// KatSmithy's filtered armor list, as byte offsets from its set-ID array (u16[0x514]).
/// The array moves between KatSmithy builds (+0x1E2664 -> +0x1F78C4); these distances
/// stayed the same, so finding the array locates everything else.
pub const CAPACITY:u32=0x514;
pub const COUNT:usize=0xa28;pub const ROW:usize=0xa2c;pub const PIECE:usize=0xa30;
/// 1 while KatSmithy shows the skill page (Y), 0 on the ordinary armor status.
pub const MODE:usize=0x23dc;
/// 1 while the cursor is on the set's full-set cell instead of one piece.
pub const SET_CELL:usize=0x23e0;
pub const SPAN:usize=0x23e4;
/// Module offsets of the builds seen so far: tried before any search.
pub const KNOWN:[u32;2]=[0x1e2664,0x1f78c4];
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Cursor{pub row:u32,pub piece:u32,pub skills:bool,pub set:bool,pub id:u16}
fn w32(b:&[u8],o:usize)->u32{u32::from_le_bytes([b[o],b[o+1],b[o+2],b[o+3]])}
fn w16(b:&[u8],o:usize)->u16{u16::from_le_bytes([b[o],b[o+1]])}
/// Per-frame check of the list at `b` (SPAN bytes from the ID array).
pub fn cursor(b:&[u8])->Option<Cursor>{
 if b.len()<SPAN{return None}
 let (count,row,piece,mode,set)=(w32(b,COUNT),w32(b,ROW),w32(b,PIECE),w32(b,MODE),w32(b,SET_CELL));
 if !(1..=CAPACITY).contains(&count)||row>=count||piece>=5||mode>1||set>1{return None}
 let id=w16(b,row as usize*2);
 if id==0||id>2000{return None}
 Some(Cursor{row,piece,skills:mode==1,set:set==1,id})
}
/// Search-time check: every listed entry is a distinct plausible armor series ID.
pub fn plausible(b:&[u8])->Option<u32>{
 cursor(b)?;let count=w32(b,COUNT);let mut seen=[0u64;32];
 for n in 0..count as usize{
  let id=w16(b,n*2) as usize;if id==0||id>2000{return None}
  let (word,bit)=(id/64,1u64<<(id%64));if seen[word]&bit!=0{return None}seen[word]|=bit;
 }
 Some(count)
}
/// A forge list is never this short (low rank alone lists 87 sets for one style);
/// short tables elsewhere in KatSmithy can look like one.
pub const MIN_LIST:u32=8;
/// Offset of KatSmithy's list in its data segment: the longest plausible list.
pub fn find(data:&[u8])->Option<usize>{
 let mut best:Option<(usize,u32)>=None;let mut i=0;
 while i+SPAN<=data.len(){
  let c=w32(data,i+COUNT);
  if (1..=CAPACITY).contains(&c)&&w32(data,i+ROW)<c&&w32(data,i+PIECE)<5{
   if let Some(n)=plausible(&data[i..i+SPAN]).filter(|&n|n>=MIN_LIST){if best.is_none_or(|b|n>b.1){best=Some((i,n));}}
  }
  i+=4;
 }
 best.map(|b|b.0)
}
#[cfg(test)]mod cursor_tests{
 use super::*;
 extern crate std;use std::vec::Vec;
 fn list(ids:&[u16],row:u32,piece:u32,mode:u32,set:u32)->Vec<u8>{
  let mut b=std::vec![0u8;SPAN];for (n,id) in ids.iter().enumerate(){b[n*2..n*2+2].copy_from_slice(&id.to_le_bytes());}
  for (o,v) in [(COUNT,ids.len() as u32),(ROW,row),(PIECE,piece),(MODE,mode),(SET_CELL,set)]{b[o..o+4].copy_from_slice(&v.to_le_bytes());}b
 }
 #[test]fn locked_torso_reads_kat_selection(){assert_eq!(cursor(&list(&[31,33,37],1,1,1,0)),Some(Cursor{row:1,piece:1,skills:true,set:false,id:33}));}
 #[test]fn set_cell_and_status_page_are_reported(){let c=cursor(&list(&[1,2,3,4,6,7,8,9,11],8,0,0,1)).unwrap();assert!(c.set&&!c.skills);assert_eq!(c.id,11);}
 #[test]fn invalid_cursors_are_rejected(){
  for (row,piece,mode,set) in [(3,0,1,0),(0,5,1,0),(0,0,2,0),(0,0,1,2)]{assert_eq!(cursor(&list(&[1,2,3],row,piece,mode,set)),None);}
  assert_eq!(cursor(&list(&[1,0,3],1,0,1,0)),None);assert_eq!(cursor(&list(&[],0,0,1,0)),None);
 }
 #[test]fn duplicate_ids_are_not_a_list(){assert_eq!(plausible(&list(&[1,2,2],0,0,1,0)),None);assert_eq!(plausible(&list(&[1,2,3],0,0,1,0)),Some(3));}
 #[test]fn longest_plausible_list_wins(){
  let mut data=std::vec![0u8;0x100];data.extend(list(&[774,1280,517],0,0,0,0));
  assert_eq!(find(&data),None,"a three-entry table is not a forge list");
  let at=data.len();data.extend(list(&[1,2,3,4,6,7,8,9,11],8,0,1,0));data.extend(std::vec![0u8;0x40]);
  assert_eq!(find(&data),Some(at));
 }
 /// Memory dumps of both KatSmithy builds (data segment from its start), when present.
 #[test]fn dumps_of_both_builds(){
  for (file,data,want,row,piece,skills,set) in [
   ("target/katsmithy_vespoid_arms_module.bin",0x1c3000usize,0x1e2664usize,18,2,true,false),
   ("target/katsmithy_r2_set_status.bin",0x1d8000,0x1f78c4,8,0,false,true),
   ("target/katsmithy_r2_head_skills.bin",0x1d8000,0x1f78c4,8,0,true,false)]{
   let Ok(b)=std::fs::read(file) else{std::println!("skip {file}");continue};
   let at=find(&b).expect(file);assert_eq!(at+data,want,"{file}");
   let c=cursor(&b[at..at+SPAN]).unwrap();assert_eq!((c.row,c.piece,c.skills,c.set),(row,piece,skills,set),"{file}");
   assert!(KNOWN.contains(&(want as u32)));
  }
 }
}
