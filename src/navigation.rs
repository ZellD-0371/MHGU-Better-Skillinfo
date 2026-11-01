//! One page per deliberate tilt. Releasing L3 or leaving the panel cancels capture.
pub const L3:u64=1<<4;
pub const RIGHT_STICK:u64=(1<<5)|(15<<20);
#[derive(Clone,Copy)]
pub struct Navigation {pub armed:bool}
impl Navigation {
    pub const fn new()->Self{Self{armed:false}}
    pub fn sample(&mut self,active:bool,buttons:u64,x:i32,y:i32)->(bool,i32){
        if !active||buttons&L3==0{self.armed=false;return (false,0)}
        if x.abs()<8000&&y.abs()<8000{self.armed=true;return (true,0)}
        if self.armed&&x.abs()>=18000&&x.abs()>y.abs(){self.armed=false;return (true,if x>0{1}else{-1})}
        (true,0)
    }
    pub fn page(current:usize,count:usize,direction:i32)->usize{
        if count<=1{return 0}
        if direction>0{(current+1)%count}else if direction<0{(current+count-1)%count}else{current.min(count-1)}
    }
}
