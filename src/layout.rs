//! Shared logical viewport coordinates; no emulator-specific DPI multiplier.
pub fn viewport_size(viewport:(f32,f32),fallback:(f32,f32))->Option<(f32,f32)>{
    [viewport,fallback].into_iter().find(|&(x,y)|x.is_finite()&&y.is_finite()&&x>0.&&y>0.)
}

/// The native render picture and HUD are separate from the host window.
#[derive(Clone,Copy,Debug)]
pub struct HudViewport{pub scale:f32,pub left:f32,pub top:f32,pub center:f32}
pub fn native_viewport(w:f32,h:f32)->Option<HudViewport>{
    viewport_size((w,h),(0.,0.))?;
    let scale=(w/1280.).min(h/720.);
    if w/h>=16./9.-0.01{Some(HudViewport{scale,left:(w-1280.*scale)*0.5,top:(h-720.*scale)*0.5,center:w*0.5})}
    else{Some(HudViewport{scale:w/1280.,left:0.,top:(h-h.min(w/1.6))*0.5,center:w*0.5})}
}
pub fn hud_viewport(w:f32,h:f32,rw:u32,rh:u32,uw:u32)->Option<HudViewport>{
    native_viewport(w,h)?;
    if !(64..=16384).contains(&rw)||!(64..=16384).contains(&rh)||!(1280..=7200).contains(&uw){return None}
    let aspect=rw as f32/rh as f32;
    let ph=h.min(w/aspect);let pw=ph*aspect;
    let scale=(ph/720.).min(pw/1280.);
    // A resize can expose mismatched dimensions for one frame; use fallback
    // rather than placing the panel outside the picture. Allow rounding by 2px.
    if uw as f32>pw/scale+2.{return None}
    Some(HudViewport{scale,left:(w-uw as f32*scale)*0.5,top:(h-ph)*0.5,center:w*0.5})
}
impl HudViewport{
    pub fn panel_position(self,center:bool,x:f32,y:f32,width:f32)->(f32,f32){
        let left=if center{self.center-width*self.scale*0.5}else{self.left};
        (left+x*self.scale,self.top+y*self.scale)
    }
}
/// Global 1.4.0 renderer metadata, shared with Better Monster Information.
/// Only reads values already reflected by the game's current HUD anchor.
pub fn hud_size(base:u32)->Option<(u32,u32,u32)>{
    if base==0||base>0xeffff000-0x212699c{return None}
    let rd=|p:u32|unsafe{core::ptr::read_unaligned(p as *const u32)};
    let render=rd(base+0x2126998);
    if !(0x10000..=0xeffff000-0x44bad0).contains(&render)||render&3!=0||rd(render)!=base+0x1754a58{return None}
    Some((rd(render+0x44bac0),rd(render+0x44bac4),rd(render+0x44bac8)))
}
