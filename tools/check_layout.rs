// Headless geometry checks. Never starts the game or dereferences game memory.
#[path="../src/layout.rs"] mod layout;
fn near(a:f32,b:f32){assert!((a-b).abs()<0.02,"{a} != {b}");}
#[test]fn original_fhd_coordinates_are_preserved(){
 let v=layout::hud_viewport(1920.,1080.,1920,1080,1280).unwrap();
 let(x,y)=v.panel_position(false,42.,53.,375.);near(x,63.);near(y,79.5);near(v.scale,1.5);
 let(x,y)=v.panel_position(true,0.,53.,375.);near(x,678.75);near(y,79.5);
}
#[test]fn wide_aspects_follow_each_hud_anchor(){
 for (w,h) in [(2560.,1080.),(3440.,1440.),(3840.,1080.),(5120.,1440.)]{
  for anchor in [16./9.,21./9.,32./9.,w/h]{
   let aspect:f32=anchor;let uw=((720.*aspect.min(w/h)+0.5) as u32)&!1;
   let v=layout::hud_viewport(w,h,w as u32,h as u32,uw).unwrap();
   near(v.scale,h/720.);near(v.left,(w-uw as f32*v.scale)*0.5);
   let(x,y)=v.panel_position(false,42.,53.,375.);near(x,v.left+42.*v.scale);near(y,53.*v.scale);
   let(cx,_)=v.panel_position(true,0.,53.,375.);near(cx+375.*v.scale/2.,w/2.);
   assert!(x>=0.&&x+375.*v.scale<w);
  }
 }
}
#[test]fn center_and_edges_have_expected_wide_positions(){
 let center=layout::hud_viewport(5120.,1440.,5120,1440,1280).unwrap();
 let edges=layout::hud_viewport(5120.,1440.,5120,1440,2560).unwrap();
 near(center.panel_position(false,42.,53.,375.).0,1364.);
 near(edges.panel_position(false,42.,53.,375.).0,84.);
 near(center.panel_position(true,0.,53.,375.).0,edges.panel_position(true,0.,53.,375.).0);
}
#[test]fn letterboxing_uses_render_picture_not_window_height(){
 let v=layout::hud_viewport(2560.,1440.,2560,1080,1280).unwrap();
 near(v.scale,1.5);near(v.top,180.);near(v.left,320.);
 let(x,y)=v.panel_position(false,42.,53.,375.);near(x,383.);near(y,259.5);
}
#[test]fn narrow_and_emulator_fallbacks_are_finite(){
 for(w,h)in [(1280.,720.),(1920.,1080.),(2560.,1440.),(3840.,2160.),(1440.,900.),(1920.,1200.),(1680.,720.),(5120.,1440.)]{
  let v=layout::native_viewport(w,h).unwrap();assert!(v.scale>0.&&v.left>=0.&&v.top>=0.);
  let(x,y)=v.panel_position(false,42.,53.,375.);assert!(x.is_finite()&&y.is_finite());
  if w/h<16./9.{near(v.scale,w/1280.);near(v.top,0.);}
 }
}
#[test]fn invalid_metadata_is_rejected_without_memory_reads(){
 for(w,h)in [(0.,720.),(1280.,0.),(-1.,720.),(f32::NAN,720.),(f32::INFINITY,720.)]{assert!(layout::native_viewport(w,h).is_none());}
 for(rw,rh,uw)in [(0,720,1280),(1280,0,1280),(20000,1080,1280),(1280,720,0),(1280,720,8000),(1280,720,2560)]{assert!(layout::hud_viewport(1920.,1080.,rw,rh,uw).is_none());}
 assert!(layout::hud_size(0).is_none());assert!(layout::hud_size(0xeffff000).is_none());
}
