//! Filter the game's five nn::hid imports, leaving Forge and host input untouched.
use core::{ptr,sync::atomic::{AtomicU32,Ordering}};
use crate::{STATE,content,navigation::{L3,RIGHT_STICK,Navigation}};
const PLT:[u32;5]=[0x13f0af0,0x13f0afc,0x13f0b08,0x13f0b14,0x13f0b20];
const LAZY_RESOLVER:u32=0x13efd44;
const GOT:[u32;5]=[0x1838494,0x1838498,0x183849c,0x18384a0,0x18384a4];
// Compact diagnostic snapshot: raw buttons/X/Y, delivered buttons/X/Y, capture, scope, sequence.
#[unsafe(no_mangle)]
pub static better_skillinfo_input_audit:[AtomicU32;9]=[const{AtomicU32::new(0)};9];
static ORIGINAL:[AtomicU32;5]=[const{AtomicU32::new(0)};5];
static SHORTCUT_HELD:AtomicU32=AtomicU32::new(0);
unsafe fn read(a:u32)->u32{unsafe{ptr::read_unaligned(a as *const u32)}}
unsafe fn write(a:u32,v:u32){unsafe{ptr::write_unaligned(a as *mut u32,v)}}

unsafe fn filter(index:usize,state:u32,id:u32){
    let original=ORIGINAL[index].load(Ordering::Acquire);
    let call:unsafe extern "C" fn(u32,u32)=unsafe{core::mem::transmute(original)};
    unsafe{call(state,id)};
    if state<0x10000||unsafe{read(state+32)}&1==0{return}
    let buttons=unsafe{read(state+8)} as u64|((unsafe{read(state+12)} as u64)<<32);
    let x=unsafe{read(state+24)} as i32;let y=unsafe{read(state+28)} as i32;
    let menu=forge::graphics::Graphics::is_menu_visible();
    let (captured,active)=STATE.with(|s|{
        let selected=if menu{None}else{content(s,false)};
        let active=selected.is_some()&&s.input_ready;
        if let Some(c)=selected{if s.page_key!=c.key{s.page=0;s.page_count=1;s.page_key=c.key;}}
        let (captured,direction)=s.navigation.sample(active,buttons,x,y);
        if direction!=0{
            let next=Navigation::page(s.page,s.page_count,direction);
            if next!=s.page{crate::sound::request();s.page=next;}
        }
        (captured,active)
    });
    // Recomp maps Ctrl -> L3 and arrows -> D-pad in the SAME input snapshot.
    // Forge's keyboard cache is updated later, at render time: requiring its arrow
    // state here leaked the first poll of a press. Use the unmodified snapshot's
    // modifier/directions, with cached Ctrl only as a compatibility fallback.
    let horizontal=buttons&((1<<12)|(1<<14));
    let modifier=buttons&L3!=0||forge::input::Input::is_ctrl_down();
    let previous=SHORTCUT_HELD.load(Ordering::Relaxed) as u64;
    let keyboard_mask=if active{
        // Keep consumed arrows suppressed through modifier-first release. A new
        // ordinary arrow press passes through after the consumed key is released.
        (previous&horizontal)|if modifier{horizontal}else{0}
    }else{0};
    SHORTCUT_HELD.store(keyboard_mask as u32,Ordering::Relaxed);
    let filtered=(if captured{buttons&!(RIGHT_STICK|L3)}else{buttons})&!keyboard_mask;
    let delivered_x=if captured{0}else{x};let delivered_y=if captured{0}else{y};
    for (i,v) in [buttons as u32,x as u32,y as u32,filtered as u32,delivered_x as u32,delivered_y as u32,captured as u32,active as u32].iter().enumerate(){better_skillinfo_input_audit[i].store(*v,Ordering::Relaxed);}
    better_skillinfo_input_audit[8].fetch_add(1,Ordering::Release);
    if filtered!=buttons{unsafe{write(state+8,filtered as u32);write(state+12,(filtered>>32) as u32);}}
    if captured{unsafe{write(state+24,0);write(state+28,0);}}
}
unsafe extern "C" fn full(s:u32,id:u32){unsafe{filter(0,s,id)}}
unsafe extern "C" fn dual(s:u32,id:u32){unsafe{filter(1,s,id)}}
unsafe extern "C" fn left(s:u32,id:u32){unsafe{filter(2,s,id)}}
unsafe extern "C" fn right(s:u32,id:u32){unsafe{filter(3,s,id)}}
unsafe extern "C" fn handheld(s:u32,id:u32){unsafe{filter(4,s,id)}}

pub fn install(base:u32)->bool{
    let detours=[full as *const (),dual as *const (),left as *const (),right as *const (),handheld as *const ()];
    // Validate every slot before changing any. Existing import wrappers remain in the call chain.
    for (i,offset) in GOT.iter().enumerate(){
        let mut address=unsafe{read(base+offset)};
        if address==base+LAZY_RESOLVER{
            // Bind through the real PLT entry, which supplies the resolver's r12.
            // Calling/storing the shared resolver stub as a function pointer is
            // invalid, and later binding would overwrite our GOT filter.
            #[repr(C,align(8))] struct NpadState([u8;40]);
            let mut state=NpadState([0;40]);let port=0u32;
            let bind:unsafe extern "C" fn(*mut NpadState,*const u32)=unsafe{core::mem::transmute(base+PLT[i])};
            unsafe{bind(&mut state,&port)};
            address=unsafe{read(base+offset)};
        }
        if address<0x10000||address&3!=0||address==base+LAZY_RESOLVER{return false}
        ORIGINAL[i].store(address,Ordering::Release);
    }
    for (i,offset) in GOT.iter().enumerate(){unsafe{ptr::write_volatile((base+offset) as *mut u32,detours[i] as u32)};}
    true
}
