pub fn identifies(ro:&[u8])->bool{
 [b"[KatSmithy]".as_slice(),b"katsmithy.ini".as_slice(),b"Skill points by piece".as_slice()]
 .iter().all(|s|ro.windows(s.len()).any(|w|w==*s))
}
pub fn in_armor_list(screen:u32,kind:u16,id:u16)->bool{screen==3&&(1..=5).contains(&kind)&&id!=0}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn ordinary_and_other_menus_are_excluded(){assert!(!in_armor_list(1,1,3));assert!(!in_armor_list(3,6,3));assert!(!in_armor_list(3,1,0));assert!(in_armor_list(3,2,4));}
 #[test]fn module_identity_requires_all_markers(){assert!(identifies(b"[KatSmithy] katsmithy.ini Skill points by piece"));assert!(!identifies(b"[KatSmithy]"));assert!(!identifies(b"katsmithy.ini"));}
}

/// Loaded alone is not active. Require the guarded entry to belong to KatSmithy.
pub fn kat_hook_ready(instruction:u32,target:u32,module:u32,size:u32)->bool{
 instruction==0xe51ff004&&module!=0&&target&3==0&&target>=module&&target.checked_sub(module).is_some_and(|n|n<size)
}
#[cfg(test)]mod initialization_tests{
 use super::*;
 #[test]fn loaded_but_unhooked_keeps_native_ui(){assert!(!kat_hook_ready(0xe92d41f0,0xe24dd018,0x33e0000,0x488000));}
 #[test]fn foreign_hook_does_not_enable_override(){assert!(!kat_hook_ready(0xe51ff004,0x3879188,0x33e0000,0x488000));}
 #[test]fn kat_hook_enables_observer(){assert!(kat_hook_ready(0xe51ff004,0x33e4e50,0x33e0000,0x488000));}
}

/// Bound all indices before reading the optional module's set-ID array.
pub fn valid_cursor(count:u32,row:u32,piece:u32)->bool{count>0&&count<=1024&&row<count&&piece<5}
#[cfg(test)]mod cursor_tests{
 use super::*;
 #[test]fn locked_torso_and_craftable_arms_keep_same_set(){
 assert!(valid_cursor(87,18,1));assert!(valid_cursor(87,18,2));
 let ids=[31u16,33,37];assert_eq!(ids[1],33);
 // The native cursor was stale on 31; it is deliberately not an input here.
 }
 #[test]fn invalid_module_cursors_are_rejected(){for c in [(0,0,0),(1025,0,0),(87,87,1),(87,u32::MAX,1),(87,18,5)]{assert!(!valid_cursor(c.0,c.1,c.2));}}
}
