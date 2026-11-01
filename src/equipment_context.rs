//! Classify the operation before considering optional equipment-detail pages.
#[derive(Clone,Copy,PartialEq,Debug)]
pub enum EquipmentContext{Production,ProductionComparison,Upgrade,Move,Inspect,Change,Other}

pub fn classify(layout:u32,operation:u32,page_count:u32,has_sockets:bool)->EquipmentContext{
    match (layout,operation){
        // Menu -> Equipment Details is informational, even with five pages.
        (9,0)=>EquipmentContext::Inspect,
        // Forge production and upgrades can both gain an optional socket page.
        (9,1)=>EquipmentContext::Production,
        (11|12,1)=>EquipmentContext::ProductionComparison,
        (9,2)=>EquipmentContext::Upgrade,
        (0,2)=>EquipmentContext::Move,
        _ if page_count>=5&&has_sockets=>EquipmentContext::Change,
        _=>EquipmentContext::Other,
    }
}

pub fn relocation_skill_page(kind:u32,page_kind:u32)->bool{
    matches!(page_kind,7|8)||(kind==6&&page_kind==4)
}

pub fn upgrade_skill_page(kind:u32,page_kind:u32)->bool{
    ((1..=6).contains(&kind)&&matches!(page_kind,7|8))
        ||((7..=21).contains(&kind)&&page_kind==8)
}

pub fn attachment_browse(layout:u32,operation:u32,flags:u32)->bool{
    // Both locations use 0x8288. Actual upgrades use 0x8308 instead.
    operation==2&&matches!(layout,0|9)&&flags&0x80!=0&&flags&0x100==0
}

pub fn attachment_skill_page(kind:u32,page_kind:u32,has_decorations:bool)->bool{
    if !(1..=21).contains(&kind){return false}
    matches!(page_kind,7|8)||(kind==6&&page_kind==4)
        ||(has_decorations&&match kind{1..=5=>page_kind==3,7..=21=>matches!(page_kind,1|2),_=>false})
}

pub fn equipment_slot(kind:u32)->Option<usize>{
    match kind{1..=6=>Some(kind as usize),7..=21=>Some(0),_=>None}
}
pub fn equipment_grid_index(column:u32,row:u32,columns:u32,rows:u32,page:u32,pages:u32)->Option<u16>{
    if columns!=10||rows!=10||column>=columns||row>=rows||page>=pages||pages>655{return None}
    let index=page*columns*rows+row*columns+column;
    (index<65535).then_some(index as u16)
}
