//! Bounded MHGU 1.4.0 entry checks. Only Forge's two-word ARM jump is chainable.
use core::sync::atomic::{AtomicU32,Ordering};
#[derive(Debug,PartialEq,Eq)]pub enum Entry{Original,Forge,Incompatible}
pub fn classify(actual:&[u32],expected:&[u32],site:u32)->Entry{
 if actual.len()!=expected.len()||expected.len()<2{return Entry::Incompatible}
 if actual==expected{return Entry::Original}
 let target=actual[1];
 if actual[0]==0xe51ff004&&(0x10000..0xffff0000).contains(&target)&&target&3==0&&target!=site&&actual[2..]==expected[2..]{Entry::Forge}else{Entry::Incompatible}
}
pub struct Spec{pub name:&'static str,pub offset:u32,pub words:&'static [u32]}
pub const TEXT:usize=0;pub const RENDER:usize=1;pub const SHOW:usize=2;pub const CLOSE:usize=3;pub const DESTROY:usize=4;pub const EQUIP:usize=5;pub const DECO:usize=6;pub const SOCKET:usize=7;pub const BLADE:usize=8;pub const GUNNER:usize=9;pub const WEAPON:usize=10;
pub const SPECS:[Spec;11]=[
 Spec{name:"text wrapper",offset:0x56e808,words:&[0xe5900024,0xeaff6354]},
 Spec{name:"text renderer",offset:0x547564,words:&[0xe92d4070,0xe1a06002,0xe5902298,0xe1a04003]},
 Spec{name:"text show",offset:0x56ea08,words:&[0xe92d4070,0xe1a06000,0xe1a04002,0xe1a05001]},
 Spec{name:"UI close",offset:0x56dc88,words:&[0xe92d48f0,0xe1a04000,0xe3a07000,0xe5c47018]},
 Spec{name:"UI destroy",offset:0x56da30,words:&[0xe92d4010,0xe1a04000,0xe59f0084,0xe79f0000]},
 Spec{name:"equipment details",offset:0x57f5ac,words:&[0xe92d4070,0xe24dd008,0xe1a04000,0xe59d001c]},
 Spec{name:"decoration lookup",offset:0xdb7b8,words:&[0xe0800081,0xe1d010ba,0xe3510000,0x03a00000]},
 Spec{name:"socket panel",offset:0x6353fc,words:&[0xe92d4ff0,0xe24dd03c,0xe1a04000,0xe59f05e4]},
 Spec{name:"blade predicate",offset:0xdb0b0,words:&[0xe92d4070,0xe1a05000,0xe59f0098,0xe1a01005]},
 Spec{name:"gunner predicate",offset:0xdb160,words:&[0xe92d4070,0xe1a05000,0xe59f0098,0xe1a01005]},
 Spec{name:"weapon predicate",offset:0xdaf18,words:&[0xe1d000b4,0xe200001f,0xe240000b,0xe3500006]},
];
pub struct Plan{pub lifecycle:bool,pub text:bool,pub show:bool,pub equipment:bool,pub sockets:bool}
impl Plan{pub fn from_bits(bits:u32)->Self{
 let has=|id:usize|bits&(1<<id)!=0;let lifecycle=has(CLOSE)&&has(DESTROY);
 Self{lifecycle,text:lifecycle&&has(TEXT)&&has(RENDER),show:lifecycle&&has(SHOW),equipment:lifecycle&&has(EQUIP),sockets:lifecycle&&has(SOCKET)}
}}
pub fn plan()->Plan{Plan::from_bits(READY.load(Ordering::Acquire))}
static READY:AtomicU32=AtomicU32::new(0);
pub fn enabled(id:usize)->bool{READY.load(Ordering::Acquire)&(1<<id)!=0}
#[cfg(target_arch="arm")]
pub fn check(base:u32,name:&str,offset:u32,expected:&[u32])->Entry{
 let mut words=[0u32;4];for(i,w)in words[..expected.len()].iter_mut().enumerate(){*w=unsafe{core::ptr::read_unaligned((base+offset+i as u32*4) as *const u32)}}
 let actual=&words[..expected.len()];let result=classify(actual,expected,base+offset);
 if result==Entry::Incompatible{forge::log::prelude::warn!("[better-skillinfo] {} disabled: main+0x{:X}; expected={:08X?}, actual={:08X?}",name,offset,expected,actual);}
 else{crate::log_info!("[better-skillinfo] {} main+0x{:X}: {:?}",name,offset,result);}
 result
}
#[cfg(target_arch="arm")]
pub fn initialize(base:u32){let mut ready=0;for(i,s)in SPECS.iter().enumerate(){if check(base,s.name,s.offset,s.words)!=Entry::Incompatible{ready|=1<<i;}}READY.store(ready,Ordering::Release);}
/// Separate immutable layout anchors from mutable code entries. These vtable
/// entries are not changed when Forge patches the pointed-to function bodies.
pub const LAYOUT:[(u32,u32);3]=[(0x176e344,0x57e3ac),(0x177613c,0x62abf8),(0x1747bb4,0x26e50c)];
#[cfg(target_arch="arm")]
pub fn game_layout(base:u32)->bool{
 let mut ok=true;for(offset,target)in LAYOUT{let actual=unsafe{core::ptr::read_unaligned((base+offset) as *const u32)};let expected=base+target;
 if actual!=expected{forge::log::prelude::error!("[better-skillinfo] MHGU 1.4.0 layout mismatch: main+0x{:X}, expected={:08X}, actual={:08X}",offset,expected,actual);ok=false;}}
 ok
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn every_original_and_forge_entry_is_accepted(){for s in SPECS{assert_eq!(classify(s.words,s.words,0x200000+s.offset),Entry::Original);let mut a=s.words.to_vec();a[0]=0xe51ff004;a[1]=0x3900000;assert_eq!(classify(&a,s.words,0x200000+s.offset),Entry::Forge);}}
 #[test]fn unknown_changes_and_invalid_targets_reject(){for s in SPECS{let mut a=s.words.to_vec();a[0]=0;assert_eq!(classify(&a,s.words,0x200000+s.offset),Entry::Incompatible);for target in [0,0x3900001,0x200000+s.offset,0xfffffff0]{a[0]=0xe51ff004;a[1]=target;assert_eq!(classify(&a,s.words,0x200000+s.offset),Entry::Incompatible);}}}
 #[test]fn wrapper_does_not_inspect_next_function(){assert_eq!(SPECS[TEXT].words.len(),2);let image=[0xe5900024,0xeaff6354,0xe51ff004,0x3900000];assert_eq!(classify(&image[..2],SPECS[TEXT].words,0x76e808),Entry::Original);}
 #[test]fn three_word_function_does_not_depend_on_next_hook(){let image=[0xe3a00001,0xe3a01000,0xe12fff1e,0xe51ff004,0x3900000];assert_eq!(classify(&image[..3],&image[..3],0x700000),Entry::Original);}
 #[test]fn body_mismatch_under_forge_jump_rejects(){let s=&SPECS[EQUIP];let mut a=s.words.to_vec();a[0]=0xe51ff004;a[1]=0x3900000;a[2]^=1;assert_eq!(classify(&a,s.words,0x200000+s.offset),Entry::Incompatible);}
}

#[cfg(test)]mod dependency_tests{
 use super::*;const ALL:u32=(1<<11)-1;
 #[test]fn helper_failures_do_not_disable_unrelated_observers(){for id in [DECO,BLADE,GUNNER,WEAPON]{let p=Plan::from_bits(ALL&!(1<<id));assert!(p.text&&p.show&&p.equipment&&p.sockets);}}
 #[test]fn one_observer_failure_is_local(){let p=Plan::from_bits(ALL&!(1<<SOCKET));assert!(p.text&&p.show&&p.equipment&&!p.sockets);let p=Plan::from_bits(ALL&!(1<<EQUIP));assert!(p.text&&p.show&&!p.equipment&&p.sockets);}
 #[test]fn renderer_failure_only_removes_direct_wrapper(){let p=Plan::from_bits(ALL&!(1<<RENDER));assert!(!p.text&&p.show&&p.equipment&&p.sockets);}
 #[test]fn owner_lifecycle_is_a_real_shared_dependency(){for id in [CLOSE,DESTROY]{let p=Plan::from_bits(ALL&!(1<<id));assert!(!p.lifecycle&&!p.text&&!p.show&&!p.equipment&&!p.sockets);}}
}
