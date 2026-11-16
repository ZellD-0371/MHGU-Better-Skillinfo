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
