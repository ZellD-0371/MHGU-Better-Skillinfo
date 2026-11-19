//! Mouse paging on Recomp. Recomp's own Forge extensions report the pointer only while
//! some plugin freed it (Skill-Simulator or Wishlist window open); otherwise the game
//! holds the mouse and nothing here reacts. Hardware and emulators never call them.
use core::sync::atomic::{AtomicU32,Ordering};
use crate::mouse_model as m;
unsafe extern "C"{
 fn forge_input_getMouse(x:*mut f32,y:*mut f32,buttons:*mut u32)->bool;
 fn forge_graphics_isMenuVisible()->bool;
}
static PREV:AtomicU32=AtomicU32::new(m::HELD_UNKNOWN);
static X:AtomicU32=AtomicU32::new(0);static Y:AtomicU32=AtomicU32::new(0);
/// 1: pointer known this frame; 2: new left press this frame.
static FLAGS:AtomicU32=AtomicU32::new(0);
fn is_recomp()->bool{
 crate::menu_input::camera_uses_forge_original(forge_graphics_isMenuVisible as *const () as usize,forge::sys::imgui::igImFileOpen as *const () as usize,false)
}
/// Once per frame before the panel is drawn. `base`/`installed`: the game module, for
/// the renderer's picture size (the same source the HUD panel layout uses).
pub fn update(base:u32,installed:bool){
 FLAGS.store(0,Ordering::Relaxed);
 // While the pointer is unavailable a held button is unknown: the first frame it is
 // freed again must not turn a button that is already down into a click.
 if !is_recomp()||forge::graphics::Graphics::is_menu_visible(){PREV.store(m::HELD_UNKNOWN,Ordering::Relaxed);return}
 let (mut x,mut y,mut buttons)=(0f32,0f32,0u32);
 if !unsafe{forge_input_getMouse(&mut x,&mut y,&mut buttons)}{PREV.store(m::HELD_UNKNOWN,Ordering::Relaxed);return}
 let Some((w,h))=crate::panel::logical_size() else{PREV.store(m::HELD_UNKNOWN,Ordering::Relaxed);return};
 let picture=m::picture(w,h,if installed{crate::layout::hud_size(base)}else{None});
 let (x,y)=m::to_display(x,y,picture);
 let new=m::pressed(buttons,PREV.swap(buttons,Ordering::Relaxed));
 X.store(x.to_bits(),Ordering::Relaxed);Y.store(y.to_bits(),Ordering::Relaxed);
 FLAGS.store(1|((new&1!=0) as u32)<<1,Ordering::Relaxed);
}
/// The free pointer this frame and whether the left button was just pressed.
pub fn pointer()->Option<(f32,f32,bool)>{
 let f=FLAGS.load(Ordering::Relaxed);
 (f&1!=0).then(||(f32::from_bits(X.load(Ordering::Relaxed)),f32::from_bits(Y.load(Ordering::Relaxed)),f&2!=0))
}
