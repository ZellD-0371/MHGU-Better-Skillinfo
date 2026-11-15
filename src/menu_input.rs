//! Ownership at the final menu translator, after optional host action remapping.
#[derive(Default)]
pub struct Gate{captured:u32}
impl Gate{
 pub const NEW:Self=Self{captured:0};
 pub fn step(&mut self,word:u32,owned:bool,overlay:bool)->u32{
  // Forge must keep its own navigation. Retain only held consumed bits so
  // reopening/closing settings cannot reintroduce a held game action.
  self.captured&=word;
  if overlay{return word}
  if owned{self.captured|=word;}
  word&!self.captured
 }
}
/// A late GOT re-chain can put our wrapper into its own predecessor chain.
/// Bound recursion at the independent SDK import and remember the safe route.
pub struct PollChain{
 active:core::sync::atomic::AtomicU32,
 cyclic:core::sync::atomic::AtomicU32,
}
impl PollChain{
 pub const NEW:Self=Self{active:core::sync::atomic::AtomicU32::new(0),cyclic:core::sync::atomic::AtomicU32::new(0)};
 pub fn poll(&self,index:usize,chained:impl FnOnce(),direct:impl FnOnce())->(bool,bool){
  use core::sync::atomic::Ordering::*;
  let bit=1u32<<index;
  if self.active.fetch_or(bit,AcqRel)&bit!=0{
   self.cyclic.fetch_or(bit,AcqRel);direct();return(false,false)
  }
  let bypass=self.cyclic.load(Acquire)&bit!=0;
  if bypass{direct()}else{chained()}
  self.active.fetch_and(!bit,Release);
  (true,!bypass&&self.cyclic.load(Acquire)&bit!=0)
 }
}
/// Reject self/cross-style links before installation; foreign predecessors
/// are allowed and must remain immutable after this wrapper is published.
pub fn valid_pad_predecessor(address:u32,resolver:u32,own:[u32;5])->bool{
 address>=0x10000&&address&3==0&&address!=resolver&&!own.contains(&address)
}
/// Recomp resolves Forge imports to aligned host-call stubs; native Forge
/// resolves them to guest executable code. Check two independent imports.
pub fn camera_uses_forge_original(graphics:usize,file:usize,chained:bool)->bool{
 chained||[graphics,file].iter().all(|&p|(0x03000000..0x03100000).contains(&p)&&p&15==0)
}
/// Permit Forge's normal chained entry, with the verified original body intact.
pub fn valid_entry(words:[u32;4],site:u32)->bool{
 valid_signature(words,site,[0xe3a02010,0xe3a00040,0xe0022821,0xe0000821])
}
pub fn valid_camera_entry(words:[u32;4],site:u32)->bool{
 valid_signature(words,site,[0xe92d4800,0xe59f0034,0xe3a02000,0xe79fe000])
}
fn valid_signature(words:[u32;4],site:u32,original:[u32;4])->bool{
 words==original||(words[0]==0xe51ff004&&(0x10000..0xeffff000).contains(&words[1])&&words[1]&3==0&&words[1]!=site&&words[2..]==original[2..])
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn pad_install_rejects_self_links_and_unresolved_entries(){
 let own=[0x3200000,0x3200010,0x3200020,0x3200030,0x3200040];
 for p in own{assert!(!valid_pad_predecessor(p,0x15efd44,own));}
 for p in [0,3,0x15efd44,0x3400001]{assert!(!valid_pad_predecessor(p,0x15efd44,own));}
 assert!(valid_pad_predecessor(0x3400000,0x15efd44,own));
 assert!(valid_pad_predecessor(0x3000100,0x15efd44,own));
 }

 #[test]fn cyclic_poll_chain_is_bounded_and_permanently_bypassed(){
 let chain=PollChain::NEW;let sdk=core::cell::Cell::new(0);let filters=core::cell::Cell::new(0);
 let (outer,warn)=chain.poll(0,||{
   let (nested,_)=chain.poll(0,||panic!("recursion"),||sdk.set(sdk.get()+1));
   if nested{filters.set(filters.get()+1)}
 },||panic!("first call should preserve predecessor"));
 if outer{filters.set(filters.get()+1)}
 assert!(warn);assert_eq!(sdk.get(),1);assert_eq!(filters.get(),1);
 for _ in 0..100{assert_eq!(chain.poll(0,||panic!("cyclic predecessor reused"),||sdk.set(sdk.get()+1)),(true,false));}
 assert_eq!(sdk.get(),101);
 assert_eq!(chain.poll(1,||{},||panic!("another style must keep its chain")),(true,false));
 }
 #[test]fn acyclic_poll_chain_keeps_predecessor_every_time(){
 let chain=PollChain::NEW;let calls=core::cell::Cell::new(0);
 for _ in 0..100{assert_eq!(chain.poll(0,||calls.set(calls.get()+1),||panic!("unnecessary bypass")),(true,false));}
 assert_eq!(calls.get(),100);
 }

 #[test]fn recomp_always_uses_forge_original_not_middle_of_function(){
 assert!(camera_uses_forge_original(0x03000100,0x03000200,false));
 assert!(camera_uses_forge_original(0x03000100,0x03000200,true));
 }
 #[test]fn native_camera_keeps_relocated_prologue_and_existing_chains(){
 for (a,b) in [(0x2400100,0x2400200),(0,0),(0x03000101,0x03000200),(0x03000100,0x03200100)]{
 assert!(!camera_uses_forge_original(a,b,false));
 assert!(camera_uses_forge_original(a,b,true));
 }
 }

 #[test]fn camera_signature_and_chain_are_checked(){
 let original=[0xe92d4800,0xe59f0034,0xe3a02000,0xe79fe000];
 assert!(valid_camera_entry(original,0x5de4b0));
 assert!(valid_camera_entry([0xe51ff004,0x3200000,original[2],original[3]],0x5de4b0));
 assert!(!valid_camera_entry([0xe51ff004,0x3200000,0,original[3]],0x5de4b0));
 }
 #[test]fn camera_and_menu_latches_do_not_share_translated_bits(){
 let mut camera=Gate::NEW;let mut menu=Gate::NEW;
 assert_eq!(camera.step(0x40,true,false),0);
 assert_eq!(menu.step(0x800,true,false),0);
 assert_eq!(camera.step(0x40,false,false),0);
 assert_eq!(menu.step(0x800,false,false),0);
 camera.step(0,false,false);menu.step(0,false,false);
 assert_eq!(camera.step(0x40,false,false),0x40);
 assert_eq!(menu.step(0x800,false,false),0x800);
 }

 #[test]fn original_and_forge_chain_entries_are_accepted(){
  let original=[0xe3a02010,0xe3a00040,0xe0022821,0xe0000821];assert!(valid_entry(original,0x5de4f8));
  assert!(valid_entry([0xe51ff004,0x3200000,original[2],original[3]],0x5de4f8));
  for target in [0,0x3200001,0x5de4f8,0xfffffff0]{assert!(!valid_entry([0xe51ff004,target,original[2],original[3]],0x5de4f8));}
  assert!(!valid_entry([0xe51ff004,0x3200000,0,original[3]],0x5de4f8));
 }
 #[test]fn remapped_actions_are_blocked_instead_of_physical_bits(){let mut g=Gate::NEW;assert_eq!(g.step(0x4000|0x10000,true,false),0);}
 #[test]fn ordinary_right_stick_is_untouched(){let mut g=Gate::NEW;assert_eq!(g.step(0x20000,false,false),0x20000);}
 #[test]fn modifier_alone_does_not_capture_menu(){let mut g=Gate::NEW;assert_eq!(g.step(2,false,false),2);}
 #[test]fn held_input_does_not_leak_when_chord_modifier_is_released_first(){let mut g=Gate::NEW;assert_eq!(g.step(2|0x20000,true,false),0);assert_eq!(g.step(0x20000,false,false),0);assert_eq!(g.step(0,false,false),0);assert_eq!(g.step(0x20000,false,false),0x20000);}
 #[test]fn unrelated_new_actions_pass_after_chord_ends(){let mut g=Gate::NEW;g.step(0x20000,true,false);assert_eq!(g.step(0x20000|0x4000,false,false),0x4000);}
 #[test]fn forge_navigation_is_never_blocked(){let mut g=Gate::NEW;g.step(0x2000,true,false);assert_eq!(g.step(0x2000,true,true),0x2000);}
 #[test]fn binding_capture_owns_all_final_actions_including_unknown_bits(){let mut g=Gate::NEW;assert_eq!(g.step(0x0102ffff,true,false),0);}
 #[test]fn repeated_chord_stays_consumed(){let mut g=Gate::NEW;for _ in 0..60{assert_eq!(g.step(0x800,true,false),0);}g.step(0,false,false);assert_eq!(g.step(0x800,false,false),0x800);}
}
