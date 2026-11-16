//! Pure parts of the Wishlist detail-page link (headless-tested).
pub const MAGIC:[u8;16]=*b"MHGU-WL-SKILLV01";
/// Version 2 adds the draw callback slot (Wishlist calls it after its own window).
pub const VERSION:u32=2;
/// Byte offsets of Wishlist's `skill_view::View` (repr(C), version 2).
pub const OPEN:u32=20;pub const ACTIVE:u32=24;pub const KIND:u32=28;pub const ID:u32=32;
pub const RIGHT:u32=40;pub const TOP:u32=44;pub const BOTTOM:u32=48;pub const DRAW:u32=56;pub const SIZE:u32=60;
/// The block is a 4-aligned static; the magic only exists in Wishlist's data segment.
pub fn find(data:&[u8])->Option<usize>{
 (0..data.len().saturating_sub(SIZE as usize-1)).step_by(4).find(|&i|data[i..i+16]==MAGIC&&u32::from_le_bytes([data[i+16],data[i+17],data[i+18],data[i+19]])==VERSION)
}
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Piece{pub kind:u16,pub id:u16,pub right:f32,pub top:f32,pub bottom:f32}
/// None: no Wishlist window. Some(None): its window is open without an armor detail page 2.
pub fn piece(open:u32,active:u32,kind:u32,id:u32,right:f32,top:f32,bottom:f32)->Option<Option<Piece>>{
 if open!=1{return None}
 let geometry=right.is_finite()&&top.is_finite()&&bottom.is_finite()&&right>0.&&bottom>top;
 if active!=1||!(1..=5).contains(&kind)||id==0||id>0xffff||!geometry{return Some(None)}
 Some(Some(Piece{kind:kind as u16,id:id as u16,right,top,bottom}))
}
/// Panel box left of Wishlist's detail frame: (x, y, max height) in screen pixels.
pub fn placement(p:Piece,width:f32,max_height:f32)->(f32,f32,f32){((p.right-width).max(0.),p.top,max_height.min(p.bottom-p.top))}
/// Height of the panel drawn directly on a draw list, matching the ImGui window layout:
/// padding, one line per row, then either the separator and page counter or no trailing gap.
pub fn panel_height(rows:usize,pages:usize,padding:f32,font:f32,spacing:f32)->f32{
 let body=rows as f32*(font+spacing);
 let footer=if pages>1{1.+spacing+font}else{-spacing};
 (body+footer).max(font)+padding*2.
}
#[cfg(test)]mod tests{
 use super::*;
 fn block(version:u32)->[u8;60]{let mut b=[0u8;60];b[..16].copy_from_slice(&MAGIC);b[16..20].copy_from_slice(&version.to_le_bytes());b}
 #[test]fn finds_aligned_block(){let mut d=[0u8;200];d[64..124].copy_from_slice(&block(2));assert_eq!(find(&d),Some(64));}
 #[test]fn rejects_other_versions_and_truncation(){let mut d=[0u8;200];d[64..124].copy_from_slice(&block(1));assert_eq!(find(&d),None);let b=block(2);assert_eq!(find(&b[..56]),None);}
 #[test]fn closed_window_is_not_wishlist(){assert_eq!(piece(0,1,2,33,900.,80.,700.),None);}
 #[test]fn open_without_armor_page_hides_other_panels(){assert_eq!(piece(1,0,2,33,900.,80.,700.),Some(None));assert_eq!(piece(1,1,7,33,900.,80.,700.),Some(None));assert_eq!(piece(1,1,2,0,900.,80.,700.),Some(None));}
 #[test]fn armor_page_two_is_a_piece(){assert_eq!(piece(1,1,2,33,900.,80.,700.),Some(Some(Piece{kind:2,id:33,right:900.,top:80.,bottom:700.})));}
 #[test]fn bad_geometry_is_rejected(){assert_eq!(piece(1,1,2,33,f32::NAN,80.,700.),Some(None));assert_eq!(piece(1,1,2,33,900.,700.,80.),Some(None));}
 #[test]fn placement_sits_left_of_detail_frame(){let p=Piece{kind:2,id:33,right:1446.,top:104.,bottom:930.};assert_eq!(placement(p,563.,788.),(883.,104.,788.));assert_eq!(placement(p,563.,2000.).2,826.);assert_eq!(placement(Piece{right:100.,..p},563.,500.).0,0.);}
 #[test]fn height_matches_window_layout(){
  // 3 rows of 21px text with 4px spacing, one page: no trailing spacing.
  assert_eq!(panel_height(3,1,18.,21.,4.),3.*25.-4.+36.);
  // Two pages add a 1px separator, its spacing and the counter line.
  assert_eq!(panel_height(3,2,18.,21.,4.),3.*25.+1.+4.+21.+36.);
 }
}
