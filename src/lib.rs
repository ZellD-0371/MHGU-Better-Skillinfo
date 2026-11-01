#![no_std]
extern crate alloc;
mod navigation;
mod pagination;
mod controller;
mod panel;
mod layout;
mod sound;
mod generated;
mod storage;
mod locales;
mod equipped;
mod details;
mod equipment_context;
mod comparison;
mod sockets;
use core::{cell::UnsafeCell,ffi::c_void,fmt::{self,Write},ptr,sync::atomic::{AtomicBool,Ordering}};
use forge::{imgui,log::prelude::{info,warn,error},sys::imgui::{ImVec2,ImVec4,ImGuiSliderFlags,ImGuiSelectableFlags}};
use generated::*;

#[derive(Clone,Copy,PartialEq)]
struct Config { enabled:bool, armor_enabled:bool, deco_enabled:bool, change_enabled:bool, center_x:i32,center_y:i32,center_width:i32,center_height:i32,center_font_size:i32,center_opacity:f32, language:usize, show_all:bool, descriptions:bool, x:i32,y:i32,width:i32,max_height:i32,font_size:i32,opacity:f32 }
impl Config {
    const DEFAULT:Self=Self{enabled:true,armor_enabled:true,deco_enabled:true,change_enabled:true,center_x:0,center_y:53,center_width:375,center_height:525,center_font_size:14,center_opacity:0.9,language:1,show_all:false,descriptions:true,x:42,y:53,width:375,max_height:525,font_size:14,opacity:0.9};
    fn parse(text:&str)->Result<Self,()> {
        let mut c=Self::DEFAULT;
        let version3=text.lines().any(|line|line.split_once('=').is_some_and(|(k,v)|k.trim()=="version"&&v.trim()=="3"));
        let current=version3||text.lines().any(|line|line.split_once('=').is_some_and(|(k,v)|k.trim()=="version"&&v.trim()=="2"));
        for line in text.lines() {
            let Some((k,v))=line.split_once('=') else {continue};let v=v.trim();
            match k.trim() {
                "enabled"=>c.enabled=v=="true", "armor_enabled"=>c.armor_enabled=v=="true", "language"=>c.language=v.parse().map_err(|_|())?,
                "deco_enabled"=>c.deco_enabled=v=="true","change_enabled"=>c.change_enabled=v=="true",
                "center_x"=>c.center_x=v.parse().map_err(|_|())?,"center_y"=>c.center_y=v.parse().map_err(|_|())?,
                "center_width"=>c.center_width=v.parse().map_err(|_|())?,"center_height"=>c.center_height=v.parse().map_err(|_|())?,
                "center_font_size"=>c.center_font_size=v.parse().map_err(|_|())?,"center_opacity"=>c.center_opacity=v.parse().map_err(|_|())?,
                "show_descriptions"=>c.descriptions=v=="true",
                // Migrate the first release to the requested layout and sign filter once.
                "show_all" if current=>c.show_all=v=="true",
                "x" if current=>c.x=v.parse().map_err(|_|())?,"y" if current=>c.y=v.parse().map_err(|_|())?,
                "width" if current=>c.width=v.parse().map_err(|_|())?,"font_size" if current=>c.font_size=v.parse().map_err(|_|())?,
                "max_height"=>c.max_height=v.parse().map_err(|_|())?,
                "opacity" if current=>c.opacity=v.parse().map_err(|_|())?,_=>{}
            }
        }
        if !version3&&c.x==40&&c.y==20{c.x=42;c.y=53;}
        c.language=c.language.min(8);c.x=c.x.clamp(0,85);c.y=c.y.clamp(0,200);
        c.max_height=c.max_height.clamp(200,680);c.width=c.width.clamp(180,500);c.font_size=c.font_size.clamp(5,20);
        c.center_x=c.center_x.clamp(-200,200);c.center_y=c.center_y.clamp(0,200);c.center_width=c.center_width.clamp(180,500);c.center_height=c.center_height.clamp(200,680);c.center_font_size=c.center_font_size.clamp(5,20);
        if !c.center_opacity.is_finite(){return Err(())}
        c.center_opacity=((c.center_opacity.clamp(0.1,1.)*10.+0.5) as i32) as f32/10.;
        if !c.opacity.is_finite(){return Err(())}
        c.opacity=((c.opacity.clamp(0.1,1.)*10.+0.5) as i32) as f32/10.;
        Ok(c)
    }
}

fn effect_visible(points:i32,effect_points:i32,show_all:bool)->bool {
    show_all || (points>0&&effect_points>0) || (points<0&&effect_points<0)
}

struct Text { bytes:[u8;1536],len:usize }
impl Text { fn new()->Self{Self{bytes:[0;1536],len:0}} fn str(&self)->&str{unsafe{core::str::from_utf8_unchecked(&self.bytes[..self.len])}} }
impl Write for Text { fn write_str(&mut self,s:&str)->fmt::Result{if self.len+s.len()>self.bytes.len(){return Err(fmt::Error)}self.bytes[self.len..self.len+s.len()].copy_from_slice(s.as_bytes());self.len+=s.len();Ok(())} }

#[derive(Clone,Copy)]
struct Candidate { owner:u32,vt:u32,group:u32,widget:u32,deco:usize,age:u32,score:u32 }
const EMPTY:Candidate=Candidate{owner:0,vt:0,group:0,widget:0,deco:0,age:0,score:0};
#[derive(Clone,Copy)]
struct Armor {identity:u64,owner:u32,points:[(u16,i32);11],pack_totals:[i16;206],count:usize,change:bool,eligible:bool,selected:bool,slot:u8,upgrade:bool,relocate:bool,inspect:bool,production:bool,incompatible:bool}
impl Armor{const EMPTY:Self=Self{identity:0,owner:0,points:[(0,0);11],pack_totals:[equipped::UNKNOWN;206],count:0,change:false,eligible:false,selected:false,slot:255,upgrade:false,relocate:false,inspect:false,production:false,incompatible:false};}
struct State {socket_owner:u32,socket_info_owner:u32,page:usize,page_count:usize,page_key:u64,navigation:navigation::Navigation,input_ready:bool,armors:[Armor;8],config:Config,base:u32,installed:bool,frame:u32,candidates:[Candidate;8],preview:bool,preview_id:i32,preview_center:bool,status:u8,logs:u32 }
impl State { const fn new()->Self{Self{socket_owner:0,socket_info_owner:0,page:0,page_count:1,page_key:0,navigation:navigation::Navigation::new(),input_ready:false,armors:[Armor::EMPTY;8],config:Config::DEFAULT,base:0,installed:false,frame:0,candidates:[EMPTY;8],preview:false,preview_id:1,preview_center:false,status:0,logs:0}} }
struct Shared { locked:AtomicBool,data:UnsafeCell<State> }
unsafe impl Sync for Shared {}
impl Shared {
    const fn new()->Self{Self{locked:AtomicBool::new(false),data:UnsafeCell::new(State::new())}}
    fn with<R>(&self,f:impl FnOnce(&mut State)->R)->R{
        while self.locked.compare_exchange_weak(false,true,Ordering::Acquire,Ordering::Relaxed).is_err(){core::hint::spin_loop()}
        let result=f(unsafe{&mut *self.data.get()});self.locked.store(false,Ordering::Release);result
    }
}
static STATE:Shared=Shared::new();
static mut FONT:Option<imgui::ImFont>=None;

fn fnv(bytes:&[u8])->u32{bytes.iter().fold(2166136261,|h,b|(h^*b as u32).wrapping_mul(16777619))}
fn find_name(bytes:&[u8])->Option<usize>{
    let h=fnv(bytes);let mut lo=0;let mut hi=ALIASES.len();
    while lo<hi {let mid=(lo+hi)/2;if ALIASES[mid].hash<h{lo=mid+1}else{hi=mid}}
    while lo<ALIASES.len()&&ALIASES[lo].hash==h{if ALIASES[lo].name.as_bytes()==bytes{return Some(ALIASES[lo].deco)}lo+=1}None
}
unsafe fn rd(p:u32)->u32{unsafe{ptr::read_unaligned(p as *const u32)}}
unsafe fn byte(p:u32)->u8{unsafe{ptr::read(p as *const u8)}}
unsafe fn capture(owner:u32,group:u32,widget:u32,text:u32){
    if owner<0x10000{return}
    if text<0x10000{STATE.with(|s|{for c in &mut s.candidates{if c.owner==owner&&c.group==group&&c.widget==widget{*c=EMPTY}}});return}
    let mut bytes=[0u8;128];let mut len=0;
    while len<bytes.len(){let ch=unsafe{byte(text+len as u32)};if ch==0{break}bytes[len]=ch;len+=1}
    if len==bytes.len(){return}
    let found=find_name(&bytes[..len]);
    STATE.with(|s|{
        let vt=unsafe{rd(owner)};
        if vt==s.base+0x177613c{s.socket_info_owner=owner;}
        if let Some(index)=s.candidates.iter().position(|c|c.owner==owner&&c.group==group&&c.widget==widget){
            if let Some(deco)=found{s.candidates[index].deco=deco;s.candidates[index].age=s.frame;}else{s.candidates[index]=EMPTY;}
            return
        }
        let Some(deco)=found else{return};
        let off=vt.wrapping_sub(s.base);
        let score=if [0x176e344,0x176e738,0x177613c].contains(&off){100}else{10};
        if s.logs<40{info!("[better-skillinfo] item={} vt=+0x{:X} group={} widget={}",DECOS[deco].gu_item,off,group,widget);s.logs+=1;}
        let index=s.candidates.iter().position(|c|c.owner==0).unwrap_or_else(||s.candidates.iter().enumerate().min_by_key(|(_,c)|(c.score,c.age)).unwrap().0);
        s.candidates[index]=Candidate{owner,vt,group,widget,deco,age:s.frame,score};
    });
}

#[forge::hook(offset=0x56e808)]
fn set_text(owner:u32,group:u32,widget:u32,text:u32)->u32{
    unsafe{capture(owner,group,widget,text)}
    // Native wrapper: ldr r0,[r0,#0x24]; b main+0x547564.
    // Forge copies these two instructions verbatim into its trampoline. The B
    // is PC-relative and would jump elsewhere from the trampoline's RX address.
    // Reproduce the wrapper directly instead of executing that copied branch.
    let base=STATE.with(|s|s.base);
    let render:unsafe extern "C" fn(u32,u32,u32,u32)->u32=unsafe{core::mem::transmute(base+0x547564)};
    unsafe{render(rd(owner+0x24),group,widget,text)}
}
#[forge::hook(offset=0x56ea08)]
fn show_text(owner:u32,group:u32,widget:u32,text:u32)->u32{unsafe{capture(owner,group,widget,text)}original!(owner,group,widget,text)}
#[forge::hook(offset=0x56dc88)]
fn close_ui(owner:u32)->u32{STATE.with(|s|{if s.socket_owner==owner{s.socket_owner=0;s.socket_info_owner=0;}for a in &mut s.armors{if a.owner==owner{*a=Armor::EMPTY;}}for c in &mut s.candidates{if c.owner==owner{*c=EMPTY}}});original!(owner)}
#[forge::hook(offset=0x56da30)]
fn destroy_ui(owner:u32)->u32{STATE.with(|s|{if s.socket_owner==owner{s.socket_owner=0;}if s.socket_info_owner==owner{s.socket_info_owner=0;}for a in &mut s.armors{if a.owner==owner{*a=Armor::EMPTY;}}for c in &mut s.candidates{if c.owner==owner{*c=EMPTY}}});original!(owner)}

// Shared equipment-page dispatch; six AAPCS arguments, page kind is the sixth.
#[forge::hook(offset=0x57f5ac)]
fn equipment_page(owner:u32,equip:u32,compare:u32,arg3:u32,arg4:u32,page_kind:u32)->u32{
    let result=original!(owner,equip,compare,arg3,arg4,page_kind);
    details::capture(owner,equip,page_kind);
    result
}

#[forge::hook(offset=0x6353fc)]
fn socket_page(owner:u32)->u32{
    let result=original!(owner);
    STATE.with(|s|s.socket_owner=owner);
    result
}

#[forge::entry]
fn main(){
    let base=forge::mem::text_addr();
    let compatible=unsafe{rd(base+0x56e808)==0xe5900024&&rd(base+0x56e80c)==0xeaff6354&&rd(base+0x547564)==0xe92d4070&&rd(base+0x56ea08)==0xe92d4070&&rd(base+0x56dc88)==0xe92d48f0&&rd(base+0x57f5ac)==0xe92d4070&&rd(base+0xdb7b8)==0xe0800081&&rd(base+0x6353fc)==0xe92d4ff0&&rd(base+0xdb0b0)==0xe92d4070&&rd(base+0xdb160)==0xe92d4070&&rd(base+0xdaf18)==0xe1d000b4};
    let config=storage::SETTINGS.load(Config::DEFAULT,Config::parse);
    STATE.with(|s|{s.base=base;s.config=config;s.installed=compatible});
    if !compatible{error!("[better-skillinfo] MHGU 1.4.0 code signature mismatch; hooks disabled");return}
    forge::install_hook!(base,set_text);forge::install_hook!(base,show_text);forge::install_hook!(base,close_ui);forge::install_hook!(base,destroy_ui);forge::install_hook!(base,equipment_page);forge::install_hook!(base,socket_page);
    let input_ready=controller::install(base);STATE.with(|s|s.input_ready=input_ready);
    if input_ready{info!("[better-skillinfo] conditional L3 + right-stick import filters installed");}else{error!("[better-skillinfo] controller paging unavailable: input imports rejected");}
    info!("[better-skillinfo] MHGU Better_Skillinfo v0.8.32: {} items / 9 languages, main=0x{:X}",DECOS.len(),base);
}

#[unsafe(no_mangle)]
pub extern "C" fn forge_onUpdate(_dt:f32){let base=STATE.with(|s|{s.frame=s.frame.wrapping_add(1);s.base});sound::update(base);}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onImGuiInit(_ctx:*mut c_void,_alloc:*mut c_void,_free:*mut c_void){
    let f=imgui::ImFont::load(c"app:/nativeNX/plugins/better-skillinfo/NotoSansCJKkr-Regular.otf");
    if f.is_none(){warn!("[better-skillinfo] multilingual font could not be loaded");}
    unsafe{FONT=f;}
}

fn active_candidate(s:&State)->Option<usize>{
    s.candidates.iter().filter(|c|c.owner!=0&&c.score>=100&&unsafe{rd(c.owner)==c.vt&&byte(c.owner+0x18)!=0})
        .max_by_key(|c|(c.score,c.age)).map(|c|c.deco)
}
#[derive(Clone,Copy,PartialEq)]
enum Projection{Calculated,AddDecoration,ReplaceDecoration,CurrentOnly,UpgradeEquipment,MoveEquipment,InspectEquipment,UnwornDecoration,ProductionEquipment}
impl Projection{
    fn shows_totals(self)->bool{!matches!(self,Self::AddDecoration|Self::UpgradeEquipment|Self::MoveEquipment|Self::InspectEquipment|Self::UnwornDecoration|Self::ProductionEquipment)}
}
#[derive(Clone,Copy)]
struct Content {points:[(u16,i32);32],compared:[Option<i32>;32],total_before:[Option<i32>;32],total_after:[Option<i32>;32],count:usize,key:u64,preview:bool,center:bool,incompatible:bool,projection:Projection}
fn content(s:&State,allow_preview:bool)->Option<Content>{
    if !s.config.enabled||!s.installed{return None}
    if s.preview{
        if !allow_preview{return None}
        let count=s.preview_id.clamp(1,8) as usize;
        return Some(Content{points:[(0,1);32],compared:[None;32],total_before:[None;32],total_after:[None;32],count,key:0x5052455600000000|count as u64|if s.preview_center{1<<16}else{0},preview:true,center:s.preview_center,incompatible:false,projection:Projection::Calculated})
    }
    if sockets::active(s){return sockets::content(s)}
    let live=|a:&&Armor|a.owner>=0x10000&&unsafe{rd(a.owner)==s.base+0x176e344&&byte(a.owner+0x18)!=0};
    // Relocation and the equipment-details menu only inspect existing items.
    // Recognize the window even on non-skill pages to prevent stale jewel-name fallback.
    if let Some(a)=s.armors.iter().filter(live).find(|a|a.relocate||a.inspect){
        if !s.config.change_enabled||!a.eligible||a.count==0{return None}
        let mut points=[(0,0);32];points[..a.count].copy_from_slice(&a.points[..a.count]);
        let mut key=a.identity^((a.owner as u64)<<32)^if a.inspect{0x494e535000000000}else{0x4d4f564500000000};
        for &(id,p) in &points[..a.count]{key=key.wrapping_mul(1099511628211)^id as u64^((p as u32 as u64)<<16);}
        return Some(Content{points,compared:[None;32],total_before:[None;32],total_after:[None;32],count:a.count,key,preview:false,center:!a.inspect,incompatible:false,projection:if a.inspect{Projection::InspectEquipment}else{Projection::MoveEquipment}})
    }
    // A recognized equipment-change detail page takes priority over decoration-name matches.
    if s.armors.iter().filter(live).any(|a|a.change){
        let production=s.armors.iter().filter(live).any(|a|a.change&&a.production);
        if !(if production{s.config.armor_enabled}else{s.config.change_enabled}){return None}
        let mut selected=Content{points:[(0,0);32],compared:[None;32],total_before:[None;32],total_after:[None;32],count:0,key:0x4348414e47450000^if production{1<<48}else{0},preview:false,center:!production,incompatible:false,projection:Projection::Calculated};
        for bottom in [false,true]{for a in s.armors.iter().filter(live).filter(|a|a.change&&a.production==production&&a.eligible&&a.selected==bottom){
            selected.key=selected.key.wrapping_mul(1099511628211)^a.identity;
            if bottom{selected.incompatible|=a.incompatible;}
            for &(id,p) in &a.points[..a.count]{
                comparison::insert(&mut selected.points,&mut selected.compared,&mut selected.count,id,p,bottom);
            }
        }}
        // The complete calculated packs include contributions from the other slots.
        // Do not treat a skill missing on this one item as zero for the full set.
        for bottom in [false,true]{
            if let Some(a)=s.armors.iter().filter(live).find(|a|a.change&&a.production==production&&a.eligible&&a.selected==bottom&&a.pack_totals[0]!=equipped::UNKNOWN){
                for (i,(id,_)) in selected.points[..selected.count].iter().enumerate(){
                    if let Some(tree)=TREES.iter().find(|t|t.id==*id){
                        let value=a.pack_totals[tree.gu_id as usize];
                        if value!=equipped::UNKNOWN{if bottom{selected.total_after[i]=Some(value as i32);}else{selected.total_before[i]=Some(value as i32);}}
                    }
                }
            }
        }
        // Unequipping renders the lower detail as native page 27 ("No equipment").
        // It has no predicted pack and does not carry the usual 0x4000 After flag.
        // Pair the two windows by their shared pager; never apply this to incompatible gear.
        if let Some(before)=s.armors.iter().filter(live).find(|a|a.change&&!a.production&&a.eligible&&!a.selected&&(1..=6).contains(&a.slot)&&
            s.armors.iter().filter(live).any(|empty|unsafe{
                rd(empty.owner+0xf8)==27&&rd(empty.owner+0x54)==2&&rd(empty.owner+0x4c)==rd(a.owner+0x4c)
            })){
            let after=equipped::after_removal(s.base,before.slot as usize);
            for (i,(id,_)) in selected.points[..selected.count].iter().enumerate(){
                if let Some(tree)=TREES.iter().find(|t|t.id==*id){
                    let n=after[tree.gu_id as usize];
                    selected.total_after[i]=(n!=equipped::UNKNOWN).then_some(n as i32);
                }
            }
            selected.key^=0x52454d4f56450000;
        }
        for &(id,p) in &selected.points[..selected.count]{selected.key=selected.key.wrapping_mul(1099511628211)^id as u64^((p as u32 as u64)<<16);}
        for p in &selected.compared[..selected.count]{selected.key=selected.key.wrapping_mul(1099511628211)^p.unwrap_or(i32::MIN) as u32 as u64;}
        return (selected.count>0).then_some(selected)
    }
    if s.config.armor_enabled {
        if let Some(a)=s.armors.iter().filter(live).find(|a|!a.change&&a.eligible&&a.count>0&&unsafe{rd(a.owner+0xf8)==7||(a.upgrade&&rd(a.owner+0xf8)==8)}){
            let mut key=a.identity^((a.owner as u64)<<32)^0x41524d4f52000000;
            for &(id,p) in &a.points[..a.count]{key=key.wrapping_mul(1099511628211)^(id as u64)^((p as u32 as u64)<<16);}
            let mut points=[(0,0);32];points[..a.count].copy_from_slice(&a.points[..a.count]);
            let mut total_after=[None;32];
            for (i,(id,_)) in points[..a.count].iter().enumerate(){if let Some(tree)=TREES.iter().find(|t|t.id==*id){
                let n=a.pack_totals[tree.gu_id as usize];if n!=equipped::UNKNOWN{total_after[i]=Some(n as i32);}
            }}
            return Some(Content{points,compared:[None;32],total_before:[None;32],total_after,count:a.count,key,preview:false,center:false,incompatible:a.production&&a.incompatible,projection:if a.upgrade{Projection::UpgradeEquipment}else if a.production{Projection::ProductionEquipment}else{Projection::Calculated}})
        }
    }
    // A live equipment-detail window must never be treated as decoration production.
    // Jewel names can remain in its hidden widgets after switching pages or equipment.
    if s.armors.iter().filter(live).next().is_some(){return None}
    if !s.config.deco_enabled{return None}
    let index=active_candidate(s)?;if index>=DECOS.len(){return None}
    let d=&DECOS[index];let mut points=[(0,0);32];points[..d.points.len()].copy_from_slice(d.points);
    Some(Content{points,compared:[None;32],total_before:[None;32],total_after:[None;32],count:d.points.len(),key:0x4445434f00000000|index as u64,preview:false,center:false,incompatible:false,projection:Projection::AddDecoration})
}
fn push_font(size:f32)->bool{if let Some(f)=unsafe{FONT}{imgui::push_font(&f,Some(size));true}else{false}}

fn color(r:f32,g:f32,b:f32)->ImVec4{ImVec4{x:r,y:g,z:b,w:1.}}
fn colored(text:&str,c:ImVec4){imgui::push_style_color(forge::sys::imgui::ImGuiCol::Text,c);imgui::text(text);imgui::pop_style_color(1);}

fn save_config(c:Config)->Result<(),&'static str>{
    let mut t=Text::new();let _=write!(t,"version=3\nenabled={}\narmor_enabled={}\nlanguage={}\nshow_all={}\nshow_descriptions={}\nx={}\ny={}\nwidth={}\nmax_height={}\nfont_size={}\nopacity={:.1}\n",c.enabled,c.armor_enabled,c.language,c.show_all,c.descriptions,c.x,c.y,c.width,c.max_height,c.font_size,c.opacity);
    let _=write!(t,"deco_enabled={}\nchange_enabled={}\ncenter_x={}\ncenter_y={}\ncenter_width={}\ncenter_height={}\ncenter_font_size={}\ncenter_opacity={:.1}\n",c.deco_enabled,c.change_enabled,c.center_x,c.center_y,c.center_width,c.center_height,c.center_font_size,c.center_opacity);
    storage::SETTINGS.save(t.str())
}

fn confirmation(title:&core::ffi::CStr,question:&str,actions:&locales::Actions)->bool{
    let mut confirmed=false;
    if imgui::begin_popup_modal(title,None,forge::sys::imgui::ImGuiWindowFlags::AlwaysAutoResize|forge::sys::imgui::ImGuiWindowFlags::NoSavedSettings){
        imgui::text(question);
        imgui::separator();
        if imgui::button(actions.confirm,ImVec2{x:0.,y:0.}){confirmed=true;imgui::close_current_popup();}
        imgui::same_line(0.,-1.);
        if imgui::button(actions.cancel,ImVec2{x:0.,y:0.}){imgui::close_current_popup();}
        imgui::end_popup();
    }
    confirmed
}

#[unsafe(no_mangle)]
pub extern "C" fn forge_onImGuiRender(){
    if imgui::tree_node(c"Better_Skillinfo"){
        let pushed=push_font(18.);
        let (mut c,installed,mut preview,mut p,status)=STATE.with(|s|(s.config,s.installed,s.preview,s.preview_id,s.status));
        let ui=&locales::UI[c.language];
        if !installed{imgui::text(ui.unsupported);}
        imgui::checkbox(ui.enabled,&mut c.enabled);
        let armor_labels=[c"방어구 스킬 정보",c"Armor skill info",c"防具のスキル情報",c"防具技能信息",c"防具技能資訊",c"Rüstungsfähigkeiten",c"Infos des talents d’armure",c"Abilità delle armature",c"Habilidades de armadura"];
        imgui::checkbox(locales::EXTRA[c.language].deco,&mut c.deco_enabled);
        imgui::checkbox(armor_labels[c.language],&mut c.armor_enabled);
        imgui::checkbox(locales::EXTRA[c.language].change,&mut c.change_enabled);
        let labels=[c"한국어",c"English",c"日本語",c"简体中文",c"繁體中文",c"Deutsch",c"Français",c"Italiano",c"Español"];
        if imgui::begin_combo(ui.language,labels[c.language],forge::sys::imgui::ImGuiComboFlags::None){
            for i in [1,0,2,3,4,5,6,7,8]{if imgui::selectable(labels[i],c.language==i,ImGuiSelectableFlags::None,ImVec2{x:0.,y:0.}){c.language=i}}
            imgui::end_combo();
        }
        let ui=&locales::UI[c.language];
        imgui::checkbox(ui.all,&mut c.show_all);
        imgui::checkbox(ui.descriptions,&mut c.descriptions);
        imgui::text(locales::EXTRA[c.language].left.to_str().unwrap_or("Left layout"));
        imgui::slider_int(c"X",&mut c.x,0,85,ImGuiSliderFlags::AlwaysClamp);
        imgui::slider_int(c"Y",&mut c.y,0,200,ImGuiSliderFlags::AlwaysClamp);
        imgui::slider_int(ui.width,&mut c.width,180,500,ImGuiSliderFlags::AlwaysClamp);
        imgui::slider_int(ui.height,&mut c.max_height,200,680,ImGuiSliderFlags::AlwaysClamp);
        imgui::slider_int(ui.size,&mut c.font_size,5,20,ImGuiSliderFlags::AlwaysClamp);
        unsafe{forge::sys::imgui::igSliderFloat(ui.opacity.as_ptr(),&mut c.opacity,0.1,1.,c"%.1f".as_ptr(),ImGuiSliderFlags::AlwaysClamp);}
        c.opacity=((c.opacity*10.+0.5) as i32).clamp(1,10) as f32/10.;
        let extra=&locales::EXTRA[c.language];
        imgui::text(extra.center.to_str().unwrap_or("Center layout"));
        imgui::slider_int(c"X##center",&mut c.center_x,-200,200,ImGuiSliderFlags::AlwaysClamp);
        imgui::slider_int(c"Y##center",&mut c.center_y,0,200,ImGuiSliderFlags::AlwaysClamp);
        // IDs are scoped so the two layout profiles can share translated labels.
        unsafe{forge::sys::imgui::igPushID_Str(c"center-layout".as_ptr());}
        imgui::slider_int(ui.width,&mut c.center_width,180,500,ImGuiSliderFlags::AlwaysClamp);
        imgui::slider_int(ui.height,&mut c.center_height,200,680,ImGuiSliderFlags::AlwaysClamp);
        imgui::slider_int(ui.size,&mut c.center_font_size,5,20,ImGuiSliderFlags::AlwaysClamp);
        unsafe{forge::sys::imgui::igSliderFloat(ui.opacity.as_ptr(),&mut c.center_opacity,0.1,1.,c"%.1f".as_ptr(),ImGuiSliderFlags::AlwaysClamp);forge::sys::imgui::igPopID();}
        c.center_opacity=((c.center_opacity*10.+0.5) as i32).clamp(1,10) as f32/10.;
        imgui::checkbox(ui.preview,&mut preview);
        if preview{
            STATE.with(|s|{imgui::checkbox(extra.preview_center,&mut s.preview_center);});
            imgui::slider_int(ui.item,&mut p,1,8,ImGuiSliderFlags::AlwaysClamp);
            let (page,count)=STATE.with(|s|(s.page,s.page_count));let mut number=page as i32+1;
            if count>1&&imgui::slider_int(ui.page,&mut number,1,count as i32,ImGuiSliderFlags::AlwaysClamp){
                STATE.with(|s|{let next=number as usize-1;if next!=s.page{sound::request();s.page=next;}});
            }
        }
        let mut status=status;
        let actions=&locales::ACTIONS[c.language];
        if imgui::button(actions.defaults,ImVec2{x:0.,y:0.}){
            imgui::open_popup(actions.reset_title,forge::sys::imgui::ImGuiPopupFlags::None);
        }
        imgui::same_line(0.,-1.);
        if imgui::button(ui.save,ImVec2{x:0.,y:0.}){
            imgui::open_popup(actions.save_title,forge::sys::imgui::ImGuiPopupFlags::None);
        }
        if confirmation(actions.reset_title,actions.reset_question,actions){
            c=Config::DEFAULT;preview=false;p=1;status=3;
            STATE.with(|s|{s.preview_center=false;s.page=0;s.page_count=1;s.page_key=0;s.navigation=navigation::Navigation::new();});
        }
        if confirmation(actions.save_title,actions.save_question,actions){
            status=match save_config(c){Ok(())=>1,Err(e)=>{warn!("[better-skillinfo] {}",e);2}};
        }
        let ui=&locales::UI[c.language];
        match status{1=>imgui::text(ui.saved),2=>imgui::text(ui.failed),3=>imgui::text(locales::ACTIONS[c.language].reset_done),_=>{}}
        STATE.with(|s|{s.config=c;s.preview=preview;s.preview_id=p;s.status=status;});
        if pushed{imgui::pop_font();}
        imgui::tree_pop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn forge_onImGuiFreeRender(){
    let (c,selected,base,installed)=STATE.with(|s|(s.config,content(s,true),s.base,s.installed));
    let Some(selected)=selected else{STATE.with(|s|{s.page=0;s.page_count=1;s.page_key=0;s.navigation=navigation::Navigation::new();});return};
    panel::render(c,selected,base,installed);
}
