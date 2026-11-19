//! Physical binding detection plus post-remapping engine menu ownership.
use core::{ptr,sync::atomic::{AtomicU32,AtomicBool,Ordering}};
use crate::{STATE,content,navigation::L3};
const LAZY_RESOLVER:u32=0x13efd44;
const GOT:[u32;5]=[0x1838494,0x1838498,0x183849c,0x18384a0,0x18384a4];
// Compact diagnostic snapshot: raw buttons/X/Y, delivered buttons/X/Y, capture, scope, sequence.
#[unsafe(no_mangle)]
pub static better_skillinfo_input_audit:[AtomicU32;9]=[const{AtomicU32::new(0)};9];
// Once linked, never recapture a wrapper installed above us.
static INSTALLED:AtomicBool=AtomicBool::new(false);
static ORIGINAL:[AtomicU32;5]=[const{AtomicU32::new(0)};5];
static MENU_OWNED:AtomicU32=AtomicU32::new(0);
static mut MENU_GATE:crate::menu_input::Gate=crate::menu_input::Gate::NEW;
#[unsafe(no_mangle)]
pub static better_skillinfo_menu_audit:[AtomicU32;4]=[const{AtomicU32::new(0)};4];
// NSO +0x3de4f8 / runtime f_5de4f8: the same final translator as BMI.
#[forge::hook(offset=0x3de4f8)]
fn menu_translate(object:u32,word:u32)->u32{
    let mapped=original!(object,word);
    let overlay=forge::graphics::Graphics::is_menu_visible();
    let active=STATE.with(|s|s.input_ready&&content(s,false).is_some());
    let owned=(active&&MENU_OWNED.load(Ordering::Acquire)!=0)||crate::bindings::capturing();
    let filtered=unsafe{(&mut *core::ptr::addr_of_mut!(MENU_GATE)).step(mapped,owned,overlay)};
    for(i,value)in [word,mapped,filtered,owned as u32].into_iter().enumerate(){better_skillinfo_menu_audit[i].store(value,Ordering::Relaxed);}
    filtered
}
// The model preview/camera reads the separate gameplay word, reconstructed by
// Recomp independently of the menu word. Keep its release latch independent.
// A copied `ldr r0,[pc,#0x34]` is NOT position independent. Replay the
// displaced prologue with its original literal, then resume at the original PC.
// Native Forge needs this replay. Recomp must instead call Forge original:
// its dispatcher accepts function entries, not our direct jump to entry + 8.
static CAMERA_FORGE_ORIGINAL:AtomicU32=AtomicU32::new(0);
static mut CAMERA_LITERAL:u32=0;
static mut CAMERA_RESUME:u32=0;
#[unsafe(naked)]
unsafe extern "C" fn camera_original(_object:u32,_word:u32,_literal:u32,_resume:u32)->u32{
    // r2/r3 are scratch and are overwritten by the original body before use.
    core::arch::naked_asm!("push {{r11, lr}}", "mov r0, r2", "bx r3");
}
static mut CAMERA_GATE:crate::menu_input::Gate=crate::menu_input::Gate::NEW;
#[forge::hook(offset=0x3de4b0)]
fn camera_translate(object:u32,word:u32)->u32{
    // Recomp originals use the host dispatcher and preserve the hook chain.
    // Only a native, unchained original may use the relocated ARM prologue.
    let mapped=if CAMERA_FORGE_ORIGINAL.load(Ordering::Acquire)!=0{original!(object,word)}
        else{unsafe{camera_original(object,word,CAMERA_LITERAL,CAMERA_RESUME)}};
    let overlay=forge::graphics::Graphics::is_menu_visible();
    let active=STATE.with(|s|s.input_ready&&content(s,false).is_some());
    let owned=(active&&MENU_OWNED.load(Ordering::Acquire)!=0)||crate::bindings::capturing();
    unsafe{(&mut *core::ptr::addr_of_mut!(CAMERA_GATE)).step(mapped,owned,overlay)}
}
static PAGE_HELD:AtomicU32=AtomicU32::new(0);
static PAD_RELEASE:AtomicU32=AtomicU32::new(0);
static SHORTCUT_HELD:AtomicU32=AtomicU32::new(0);
unsafe fn read(a:u32)->u32{unsafe{ptr::read_unaligned(a as *const u32)}}
unsafe fn write(a:u32,v:u32){unsafe{ptr::write_unaligned(a as *mut u32,v)}}

// Game-GOT wrappers may be re-chained by another plugin after startup.
// If that creates A -> B -> A, terminate at our independent SDK import and
// permanently omit the cyclic predecessor. Ordinary acyclic chains stay intact.
static IMPORT_CHAIN:crate::menu_input::PollChain=crate::menu_input::PollChain::NEW;
unsafe extern "C"{
 #[link_name="_ZN2nn3hid12GetNpadStateEPNS0_16NpadFullKeyStateERKj"] fn sdk_full(state:u32,id:u32);
 #[link_name="_ZN2nn3hid12GetNpadStateEPNS0_16NpadJoyDualStateERKj"] fn sdk_dual(state:u32,id:u32);
 #[link_name="_ZN2nn3hid12GetNpadStateEPNS0_16NpadJoyLeftStateERKj"] fn sdk_left(state:u32,id:u32);
 #[link_name="_ZN2nn3hid12GetNpadStateEPNS0_17NpadJoyRightStateERKj"] fn sdk_right(state:u32,id:u32);
 #[link_name="_ZN2nn3hid12GetNpadStateEPNS0_17NpadHandheldStateERKj"] fn sdk_handheld(state:u32,id:u32);
}
unsafe fn sdk_poll(index:usize,state:u32,id:u32){
 let polls:[unsafe extern "C" fn(u32,u32);5]=[sdk_full,sdk_dual,sdk_left,sdk_right,sdk_handheld];
 unsafe{polls[index](state,id)};
}
unsafe fn filter(index:usize,state:u32,id:u32){
    let (outer,cycle_detected)=IMPORT_CHAIN.poll(index,||{
        let original=ORIGINAL[index].load(Ordering::Acquire);
        let call:unsafe extern "C" fn(u32,u32)=unsafe{core::mem::transmute(original)};
        unsafe{call(state,id)};
    },||unsafe{sdk_poll(index,state,id)});
    // Only the outer invocation applies bindings and consumes game input.
    if !outer{return}
    if cycle_detected{
        forge::log::prelude::warn!("[better-skillinfo] cyclic pad import {} detected; independent SDK polling enabled",index);
    }
    if state<0x10000||unsafe{read(state+32)}&1==0{return}
    let buttons=unsafe{read(state+8)} as u64|((unsafe{read(state+12)} as u64)<<32);
    let x=unsafe{read(state+24)} as i32;let y=unsafe{read(state+28)} as i32;
    let raw=crate::bindings::snapshot(buttons as u32,unsafe{read(state+16)} as i32,unsafe{read(state+20)} as i32,x,y);
    crate::bindings::publish(raw);
    let menu=forge::graphics::Graphics::is_menu_visible();
    if crate::bindings::capturing()&&!menu{
        // Capture mode consumes the game's state on every native poll, on any screen.
        unsafe{write(state+8,0);write(state+12,0);for offset in [16,20,24,28]{write(state+offset,0);}}
        MENU_OWNED.store(1,Ordering::Release);
        PAGE_HELD.store(0,Ordering::Release);return
    }
    let (captured,active,shortcut_directions,chord_down,keyboard_down)=STATE.with(|s|{
        let selected=if menu{None}else{content(s,false)};
        let active=selected.is_some()&&s.input_ready;
        if let Some(c)=selected{if s.page_key!=c.key{s.page=0;s.page_count=1;s.page_key=c.key;}}
        let bindings=[s.config.previous_binding,s.config.next_binding];
        let hits=if active{(crate::bindings::matches(raw,bindings[0]) as u32)|((crate::bindings::matches(raw,bindings[1]) as u32)<<1)}else{0};
        let old=PAGE_HELD.swap(hits,Ordering::AcqRel);let pressed=hits&!old;
        let direction=((pressed&2!=0) as i32)-((pressed&1!=0) as i32);
        let mut captured=0u32;
        if active{for chord in bindings{
            let modifier=chord&0xffff;
            if crate::bindings::matches(raw,chord)||(chord&0xff0000!=0&&modifier!=0&&raw&modifier==modifier){captured|=chord;}
        }}
        if direction!=0{
            let next=crate::navigation::page(s.page,s.page_count,direction);
            if next!=s.page{crate::sound::request();s.page=next;}
        }
        let old=PAD_RELEASE.load(Ordering::Acquire);
        captured|=old&raw;
        PAD_RELEASE.store(if active{captured}else{0},Ordering::Release);
        (if active{captured}else{0},active,[s.config.keyboard_previous,s.config.keyboard_next].iter().fold(0u64,|mask,key|mask|match *key{crate::bindings::KEY_PREV=>1<<12,crate::bindings::KEY_NEXT=>1<<14,_=>0}),hits!=0,active&&(crate::bindings::key_down(s.config.keyboard_previous)||crate::bindings::key_down(s.config.keyboard_next)))
    });
    // Recomp maps Ctrl -> L3 and arrows -> D-pad in the SAME input snapshot.
    // Forge's keyboard cache is updated later, at render time: requiring its arrow
    // state here leaked the first poll of a press. Use the unmodified snapshot's
    // modifier/directions, with cached Ctrl only as a compatibility fallback.
    let horizontal=buttons&shortcut_directions;
    let modifier=buttons&L3!=0||forge::input::Input::is_ctrl_down();
    let previous=SHORTCUT_HELD.load(Ordering::Relaxed) as u64;
    let keyboard_mask=if active{
        // Keep consumed arrows suppressed through modifier-first release. A new
        // ordinary arrow press passes through after the consumed key is released.
        (previous&horizontal)|if modifier{horizontal}else{0}
    }else{0};
    SHORTCUT_HELD.store(keyboard_mask as u32,Ordering::Relaxed);
    MENU_OWNED.store((active&&(chord_down||keyboard_down||keyboard_mask!=0)) as u32,Ordering::Release);
    let filtered=buttons&!(captured as u64)&!keyboard_mask;
    let delivered_x=if captured&0xf00000!=0{0}else{x};let delivered_y=if captured&0xf00000!=0{0}else{y};
    for (i,v) in [buttons as u32,x as u32,y as u32,filtered as u32,delivered_x as u32,delivered_y as u32,captured as u32,active as u32].iter().enumerate(){better_skillinfo_input_audit[i].store(*v,Ordering::Relaxed);}
    better_skillinfo_input_audit[8].fetch_add(1,Ordering::Release);
    if filtered!=buttons{unsafe{write(state+8,filtered as u32);write(state+12,(filtered>>32) as u32);}}
    if captured&0xf00000!=0{unsafe{write(state+24,0);write(state+28,0);}}
    if captured&0xf0000!=0{unsafe{write(state+16,0);write(state+20,0);}}
}
unsafe extern "C" fn full(s:u32,id:u32){unsafe{filter(0,s,id)}}
unsafe extern "C" fn dual(s:u32,id:u32){unsafe{filter(1,s,id)}}
unsafe extern "C" fn left(s:u32,id:u32){unsafe{filter(2,s,id)}}
unsafe extern "C" fn right(s:u32,id:u32){unsafe{filter(3,s,id)}}
unsafe extern "C" fn handheld(s:u32,id:u32){unsafe{filter(4,s,id)}}

pub fn install(base:u32)->bool{
    if INSTALLED.load(Ordering::Acquire){return refresh_pending(base)}
    let site=base+0x3de4f8;
    let words=core::array::from_fn(|i|unsafe{read(site+i as u32*4)});
    if !crate::menu_input::valid_entry(words,site){return false}
    let camera_site=base+0x3de4b0;
    let camera_words=core::array::from_fn(|i|unsafe{read(camera_site+i as u32*4)});
    if !crate::menu_input::valid_camera_entry(camera_words,camera_site){return false}
    unsafe{
        CAMERA_LITERAL=read(base+0x3de4f0);
        CAMERA_RESUME=camera_site+8;
    }
    unsafe extern "C"{fn forge_graphics_isMenuVisible()->bool;}
    let forge_original=crate::menu_input::camera_uses_forge_original(
        forge_graphics_isMenuVisible as *const () as usize,
        forge::sys::imgui::igImFileOpen as *const () as usize,
        camera_words[0]==0xe51ff004,
    );
    CAMERA_FORGE_ORIGINAL.store(forge_original as u32,Ordering::Release);
    forge::install_hook!(base,menu_translate);
    forge::install_hook!(base,camera_translate);
    INSTALLED.store(true,Ordering::Release);
    refresh_pending(base)
}

/// Native SDK imports bind lazily and unsupported controller styles may never
/// resolve. Attach each style once when it becomes available; never re-chain a
/// slot that another mod subsequently wraps above us.
/// The menu and camera entry hooks are in place (pad imports may still be pending).
pub fn entry_hooked()->bool{INSTALLED.load(Ordering::Acquire)}
pub fn refresh_pending(base:u32)->bool{
 if !INSTALLED.load(Ordering::Acquire){return false}
 let detours=[full as *const (),dual as *const (),left as *const (),right as *const (),handheld as *const ()];
 for(i,offset)in GOT.iter().enumerate(){
  if ORIGINAL[i].load(Ordering::Acquire)!=0{continue}
  let address=unsafe{read(base+offset)};
  if !crate::menu_input::valid_pad_predecessor(address,base+LAZY_RESOLVER,detours.map(|p|p as u32)){continue}
  ORIGINAL[i].store(address,Ordering::Release);
  unsafe{ptr::write_volatile((base+offset) as *mut u32,detours[i] as u32)};
  crate::log_info!("[better-skillinfo] controller paging attached to pad style {} ({:08X})",i,address);
 }
 ORIGINAL.iter().any(|a|a.load(Ordering::Acquire)!=0)
}
