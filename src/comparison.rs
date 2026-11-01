use core::fmt;

pub fn replacement(current:i32,before:i32,after:i32)->i32{current-before+after}

/// Native body-column points already include every Torso Up copy.
pub fn remove_slot(total:i32,own:i32,body:i32,torso:i32,removed_torso:i32,is_body:bool)->Option<i32>{
    if is_body||removed_torso==0{return Some(total-own)}
    let multiplier=torso+1;
    if multiplier<=0||removed_torso<0||removed_torso>torso||body%multiplier!=0{return None}
    Some(total-own-body/multiplier*removed_torso)
}

pub fn matches_equipment_type(blademaster:bool,allows_blade:bool,allows_gunner:bool)->bool{
    if blademaster{allows_blade}else{allows_gunner}
}

/// Missing skills are zero on the respective side; never add Before to After.
pub fn insert<const N:usize>(before:&mut [(u16,i32);N],after:&mut [Option<i32>;N],count:&mut usize,id:u16,value:i32,is_after:bool){
    let i=match before[..*count].iter().position(|r|r.0==id){
        Some(i)=>i,
        None=>{
            if *count==N{return}
            let i=*count;*count+=1;before[i]=(id,0);after[i]=Some(0);i
        }
    };
    if is_after{after[i]=Some(value);}else{before[i].1=value;}
}

pub struct Point(pub i32);
impl fmt::Display for Point{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        if self.0==0{f.write_str("0")}else{write!(f,"{:+}",self.0)}
    }
}

/// A displayed set total takes precedence over the individual item's signs.
/// At zero, keep the Before total's direction. Unknown After also uses Before; only unknown totals keep the item fallback.
pub fn effect_total_basis(before:Option<i32>,after:Option<i32>)->Option<i32>{
    match after{Some(0)=>Some(before.unwrap_or(0)),Some(n)=>Some(n),None=>before}
}
