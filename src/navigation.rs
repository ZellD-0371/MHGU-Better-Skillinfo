//! Page stepping shared by the pad, keyboard and settings preview.
pub const L3:u64=1<<4;
/// Wraps around; 0 keeps the page (clamped to the current count).
pub fn page(current:usize,count:usize,direction:i32)->usize{
    if count<=1{return 0}
    if direction>0{(current+1)%count}else if direction<0{(current+count-1)%count}else{current.min(count-1)}
}
