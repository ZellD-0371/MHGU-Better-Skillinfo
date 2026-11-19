//! Pure parts of the page-arrow mouse support (headless-tested).
/// The game's picture inside the window: (left, top, width, height) in display pixels,
/// and the share of the picture's width the pointer's x spans.
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Picture{pub left:f32,pub top:f32,pub width:f32,pub height:f32,pub canvas:f32}
/// Recomp letterboxes the picture to the render aspect (`rw`/`rh`), and maps the pointer's
/// x over the HUD's width only when the HUD (`uw` at height 720) is narrower than the
/// picture. Unknown renderer sizes keep the whole window.
pub fn picture(w:f32,h:f32,hud:Option<(u32,u32,u32)>)->Picture{
 let whole=Picture{left:0.,top:0.,width:w,height:h,canvas:1.};
 let Some((rw,rh,uw))=hud else{return whole};
 if !(64..=16384).contains(&rw)||!(64..=16384).contains(&rh){return whole}
 let aspect=rw as f32/rh as f32;let ph=h.min(w/aspect);let pw=ph*aspect;
 let span=uw as f32/720.;let canvas=if (1280..=16384).contains(&uw)&&span<aspect{span/aspect}else{1.};
 Picture{left:(w-pw)*0.5,top:(h-ph)*0.5,width:pw,height:ph,canvas}
}
/// Recomp reports the pointer in console pixels (1280x720) over the game's picture;
/// ImGui draws in display pixels over the whole window.
pub fn to_display(x:f32,y:f32,p:Picture)->(f32,f32){
 let u=x/1280.*p.canvas+(1.-p.canvas)*0.5;
 (p.left+u*p.width,p.top+y/720.*p.height)
}
/// Buttons pressed since the last frame (bit 1 left, 2 middle, 4 right). After the
/// pointer was unavailable `previous` is `HELD_UNKNOWN`: a button already down when the
/// pointer is freed (the click that opened another mod's window) is not a new press.
pub const HELD_UNKNOWN:u32=u32::MAX;
/// Buttons pressed since the last frame.
pub fn pressed(now:u32,previous:u32)->u32{now&!previous}
/// Hit boxes of the two page arrows of "◀ n / m ▶", drawn from `left`:
/// (left, top, right, bottom), widened by `pad` so a small glyph is easy to click.
pub fn arrow_boxes(left:f32,top:f32,height:f32,arrow_w:f32,total_w:f32,pad:f32)->[(f32,f32,f32,f32);2]{
 [(left-pad,top-pad,left+arrow_w+pad,top+height+pad),(left+total_w-arrow_w-pad,top-pad,left+total_w+pad,top+height+pad)]
}
pub fn inside(x:f32,y:f32,b:(f32,f32,f32,f32))->bool{x>=b.0&&x<b.2&&y>=b.1&&y<b.3}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn scales_console_pixels(){let p=picture(1920.,1080.,None);assert_eq!(to_display(640.,360.,p),(960.,540.));assert_eq!(to_display(0.,720.,picture(1280.,720.,None)),(0.,720.));}
 #[test]fn sixteen_nine_fills_the_window(){let p=picture(1920.,1080.,Some((1920,1080,1280)));assert_eq!(p,Picture{left:0.,top:0.,width:1920.,height:1080.,canvas:1.});assert_eq!(to_display(1280.,0.,p),(1920.,0.));}
 #[test]fn forced_sixteen_ten_has_side_bands(){
  // Measured: render 1728x1080 in a 1920x1080 window, HUD width 1280.
  let p=picture(1920.,1080.,Some((1728,1080,1280)));
  assert_eq!((p.left,p.top,p.width,p.height,p.canvas),(96.,0.,1728.,1080.,1.));
  assert_eq!(to_display(0.,0.,p),(96.,0.));assert_eq!(to_display(1280.,720.,p),(1824.,1080.));assert_eq!(to_display(640.,360.,p),(960.,540.));
 }
 #[test]fn narrow_hud_on_wide_picture(){
  // 21:9 picture with a 16:9 HUD: x spans the HUD's middle part of the picture.
  let p=picture(2520.,1080.,Some((2520,1080,1280)));
  assert!((p.canvas-(1280./720.)/(2520./1080.)).abs()<1e-6);
  let (l,_)=to_display(0.,0.,p);let (r,_)=to_display(1280.,0.,p);
  assert!((l-300.).abs()<0.01&&(r-2220.).abs()<0.01);
 }
 #[test]fn bad_renderer_keeps_whole_window(){assert_eq!(picture(1920.,1080.,Some((0,1080,1280))),picture(1920.,1080.,None));}
 #[test]fn held_button_after_pointer_returns_is_not_a_click(){assert_eq!(pressed(1,HELD_UNKNOWN),0);assert_eq!(pressed(0,HELD_UNKNOWN),0);}
 #[test]fn only_new_presses(){assert_eq!(pressed(1,0),1);assert_eq!(pressed(1,1),0);assert_eq!(pressed(5,1),4);assert_eq!(pressed(0,5),0);}
 #[test]fn arrows_sit_at_both_ends(){
  let [l,r]=arrow_boxes(100.,50.,20.,12.,80.,4.);
  assert!(inside(100.,60.,l));assert!(inside(97.,47.,l),"padding");assert!(!inside(117.,60.,l));
  assert!(inside(175.,60.,r));assert!(!inside(150.,60.,r),"the page numbers are not arrows");assert!(!inside(185.,60.,r));
 }
}
