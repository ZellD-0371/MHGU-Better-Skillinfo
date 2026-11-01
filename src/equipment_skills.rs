//! Merge one item's intrinsic skills and every installed decoration by skill ID.
use crate::generated::DECOS;

pub fn add(points:&mut [(u16,i32)],count:&mut usize,id:u16,value:i32){
    if id==0{return}
    if let Some(row)=points[..*count].iter_mut().find(|row|row.0==id){row.1+=value;}
    else if *count<points.len(){points[*count]=(id,value);*count+=1;}
}

pub fn decorations(points:&mut [(u16,i32)],count:&mut usize,items:[u16;3]){
    // Each slot is a separate contribution, including repeated jewel IDs.
    for item in items{
        if let Some(deco)=DECOS.iter().find(|deco|deco.gu_item==item){
            for &(id,value) in deco.points{add(points,count,id,value);}
        }
    }
}
