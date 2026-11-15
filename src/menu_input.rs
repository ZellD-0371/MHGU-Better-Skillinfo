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
