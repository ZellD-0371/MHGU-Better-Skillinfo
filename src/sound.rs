//! MHGU 1.4.0's own shoulder-button UI page sound.
use core::{ptr,sync::atomic::{AtomicBool,Ordering}};
static REPORTED:AtomicBool=AtomicBool::new(false);
static PENDING:AtomicBool=AtomicBool::new(false);
pub fn request(){PENDING.store(true,Ordering::Release);}
pub fn update(base:u32){
    if !PENDING.swap(false,Ordering::AcqRel){return}
    unsafe{
        // The generic UI page handlers at +553834 / +553960 both enqueue SE 7.
        let entry=base+0x4eeb78;
        let words=core::array::from_fn(|i|ptr::read_unaligned((entry+i as u32*4) as *const u32));
        if !crate::sound_guard::valid(words,entry){
            if !REPORTED.swap(true,Ordering::AcqRel){forge::log::prelude::warn!("[better-skillinfo] page sound disabled: main+0x4EEB78 expected={:08X?}, actual={:08X?}",[0xe1a02000u32,0xe59200c0,0xe3500000,0x012fff1e],words);}
            return
        }
        let manager=ptr::read_unaligned((base+0x1896f5c) as *const u32);
        if manager<0x10000{return}
        let play:unsafe extern "C" fn(u32,u32)=core::mem::transmute(entry);
        play(manager,7);
    }
}
