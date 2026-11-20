//! Purple-lib: the settings under Options > Mod Options > Better Skillinfo.
//!
//! Optional. Purple-lib calls `purplelib_on_register` (found by name in the exported symbols)
//! once from its first update; without Purple-lib nothing here runs and the Forge menu stays
//! the only way to change the settings. Changes apply at once and are saved to the same INI
//! as the Forge menu's Save button when the Options window closes. Records and API follow
//! Purple-lib's purplelib.h, API version 4 (Switch layout: 4-byte pointers).
use core::ffi::{c_char,c_void};
use crate::{Config,STATE};

const API_VERSION:u32=4;
const ROW_LABELS:u32=0;const ROW_NUMBERS:u32=1;const ROW_ACTION:u32=3;
const NO_WRAP:u32=1;

#[repr(C)]
pub struct Row{
    name:*const c_char,help:*const c_char,labels:*const *const c_char,label_count:u32,kind:u32,
    value_count:u32,first:i32,step:i32,suffix:*const c_char,flags:u32,
}
#[repr(C)]
pub struct Options{
    size:u32,context:*mut c_void,
    title:Option<unsafe extern "C" fn(*mut c_void,u32)->*const c_char>,
    row_count:u32,
    row:Option<unsafe extern "C" fn(*mut c_void,u32,u32,*mut Row)->bool>,
    get:Option<unsafe extern "C" fn(*mut c_void,u32)->u32>,
    set:Option<unsafe extern "C" fn(*mut c_void,u32,u32)>,
    closed:Option<unsafe extern "C" fn(*mut c_void)>,
    value_text:Option<unsafe extern "C" fn(*mut c_void,u32,u32,u32)->*const c_char>,
    action:Option<unsafe extern "C" fn(*mut c_void,u32)>,
}
#[repr(C)]
pub struct Help{
    size:u32,context:*mut c_void,category:u32,page_count:u32,
    title:Option<unsafe extern "C" fn(*mut c_void,u32)->*const c_char>,
    page:Option<unsafe extern "C" fn(*mut c_void,u32,u32)->*const c_char>,
    open:Option<unsafe extern "C" fn(*mut c_void)>,
}
#[repr(C)]
pub struct Api{
    version:u32,
    add_options:Option<unsafe extern "C" fn(*const Options)->bool>,
    add_help:Option<unsafe extern "C" fn(*const Help)->bool>,
    hold_input:Option<unsafe extern "C" fn()>,
    options_shown:Option<unsafe extern "C" fn(*const Options)->bool>,
    options_row:Option<unsafe extern "C" fn(*const Options)->i32>,
    add_set_help:Option<unsafe extern "C" fn(*const Options,*const Help)->bool>,
    help_shown:Option<unsafe extern "C" fn(*const Options)->bool>,
}
#[cfg(target_arch="arm")]
const _:()={
    assert!(core::mem::size_of::<Options>()==40);assert!(core::mem::size_of::<Row>()==40);
    assert!(core::mem::size_of::<Help>()==28);assert!(core::mem::size_of::<Api>()==32);
};

/// A string the game may keep a pointer to: Purple-lib's rule is that it lives forever.
struct Str(*const c_char);
unsafe impl Sync for Str {}
macro_rules! s{($t:literal)=>{Str(concat!($t,"\0").as_ptr() as *const c_char)}}
struct Labels<const N:usize>([*const c_char;N]);
unsafe impl<const N:usize> Sync for Labels<N> {}
macro_rules! labels{($($t:literal),*)=>{Labels([$(concat!($t,"\0").as_ptr() as *const c_char),*])}}

static ON_OFF:Labels<2>=labels!("Off","On");
static RESET:Labels<1>=labels!("Reset");
/// The mod's language indices (Config::language) in the order the row shows them.
const LANGUAGE_ORDER:[usize;9]=[1,0,2,3,4,5,6,7,8];
static LANGUAGES:Labels<9>=labels!("English","Korean","Japanese","Chinese (S)","Chinese (T)","German","French","Italian","Spanish");

#[derive(Clone,Copy)]
enum Kind{
    Switch(fn(&mut Config)->&mut bool),
    Number{first:i32,step:i32,count:u32,suffix:Option<&'static Str>,field:fn(&mut Config)->&mut i32},
    Opacity(fn(&mut Config)->&mut f32),
    Language,
    Defaults,
}
struct Entry{name:Str,help:Str,kind:Kind}
unsafe impl Sync for Entry {}
const fn size(field:fn(&mut Config)->&mut i32)->Kind{Kind::Number{first:5,step:1,count:16,suffix:None,field}}
const fn width(field:fn(&mut Config)->&mut i32)->Kind{Kind::Number{first:180,step:5,count:65,suffix:None,field}}
const fn height(field:fn(&mut Config)->&mut i32)->Kind{Kind::Number{first:200,step:5,count:97,suffix:None,field}}
const fn top(field:fn(&mut Config)->&mut i32)->Kind{Kind::Number{first:0,step:1,count:201,suffix:None,field}}

// Sizes and positions are in the game's 1280x720 screen units, as in the Forge menu.
static ROWS:[Entry;21]=[
    Entry{name:s!("Better Skillinfo"),help:s!("Show the skill information panel."),kind:Kind::Switch(|c|&mut c.enabled)},
    Entry{name:s!("Decorations"),help:s!("Skill info for decorations."),kind:Kind::Switch(|c|&mut c.deco_enabled)},
    Entry{name:s!("Armor"),help:s!("Skill info for armor at the Smithy."),kind:Kind::Switch(|c|&mut c.armor_enabled)},
    Entry{name:s!("Equipment change"),help:s!("Compare skills when changing equipment."),kind:Kind::Switch(|c|&mut c.change_enabled)},
    Entry{name:s!("Show all effects"),help:s!("Also list effects the points don't reach."),kind:Kind::Switch(|c|&mut c.show_all)},
    Entry{name:s!("Descriptions"),help:s!("Show what each skill effect does."),kind:Kind::Switch(|c|&mut c.descriptions)},
    Entry{name:s!("Language"),help:s!("The language of the skill panel."),kind:Kind::Language},
    Entry{name:s!("Left: Text size"),help:s!("Text size of the left panel (Smithy)."),kind:size(|c|&mut c.font_size)},
    Entry{name:s!("Left: Width"),help:s!("Width of the left panel."),kind:width(|c|&mut c.width)},
    Entry{name:s!("Left: Max height"),help:s!("Height before the left panel adds pages."),kind:height(|c|&mut c.max_height)},
    Entry{name:s!("Left: X"),help:s!("Distance of the left panel from the left."),kind:Kind::Number{first:0,step:1,count:86,suffix:None,field:|c|&mut c.x}},
    Entry{name:s!("Left: Y"),help:s!("Distance of the left panel from the top."),kind:top(|c|&mut c.y)},
    Entry{name:s!("Left: Opacity"),help:s!("Background opacity of the left panel."),kind:Kind::Opacity(|c|&mut c.opacity)},
    Entry{name:s!("Center: Text size"),help:s!("Text size of the center panel (Item Box)."),kind:size(|c|&mut c.center_font_size)},
    Entry{name:s!("Center: Width"),help:s!("Width of the center panel."),kind:width(|c|&mut c.center_width)},
    Entry{name:s!("Center: Max height"),help:s!("Height before the center panel adds pages."),kind:height(|c|&mut c.center_height)},
    Entry{name:s!("Center: X"),help:s!("Offset of the center panel from the middle."),kind:Kind::Number{first:-200,step:5,count:81,suffix:None,field:|c|&mut c.center_x}},
    Entry{name:s!("Center: Y"),help:s!("Distance of the center panel from the top."),kind:top(|c|&mut c.center_y)},
    Entry{name:s!("Center: Opacity"),help:s!("Background opacity of the center panel."),kind:Kind::Opacity(|c|&mut c.center_opacity)},
    Entry{name:s!("Skill-Sim text size"),help:s!("Text size while Skill-Simulator is open."),kind:size(|c|&mut c.sim_font_size)},
    Entry{name:s!("Defaults"),help:s!("Put every setting back to its default."),kind:Kind::Defaults},
];
static PERCENT:Str=s!("%");

unsafe extern "C" fn title(_:*mut c_void,_:u32)->*const c_char{c"Better Skillinfo".as_ptr()}

unsafe extern "C" fn row(_:*mut c_void,_:u32,index:u32,out:*mut Row)->bool{
    let (Some(e),Some(out))=(ROWS.get(index as usize),unsafe{out.as_mut()}) else{return false};
    out.name=e.name.0;out.help=e.help.0;
    match e.kind{
        Kind::Switch(_)=>{out.kind=ROW_LABELS;out.labels=ON_OFF.0.as_ptr();out.label_count=2;}
        Kind::Language=>{out.kind=ROW_LABELS;out.labels=LANGUAGES.0.as_ptr();out.label_count=9;}
        Kind::Number{first,step,count,suffix,..}=>{
            out.kind=ROW_NUMBERS;out.value_count=count;out.first=first;out.step=step;
            out.suffix=suffix.map_or(core::ptr::null(),|s|s.0);out.flags=NO_WRAP;
        }
        Kind::Opacity(_)=>{out.kind=ROW_NUMBERS;out.value_count=10;out.first=10;out.step=10;out.suffix=PERCENT.0;out.flags=NO_WRAP;}
        Kind::Defaults=>{out.kind=ROW_ACTION;out.labels=RESET.0.as_ptr();out.label_count=1;}
    }
    true
}

unsafe extern "C" fn get(_:*mut c_void,index:u32)->u32{
    let Some(e)=ROWS.get(index as usize) else{return 0};
    let mut c=STATE.with(|s|s.config);
    match e.kind{
        Kind::Switch(field)=>*field(&mut c) as u32,
        Kind::Language=>LANGUAGE_ORDER.iter().position(|&l|l==c.language).unwrap_or(0) as u32,
        Kind::Number{first,step,count,field,..}=>{
            // Nearest step, so a value set in the Forge menu between two steps still shows.
            let v=*field(&mut c);(((v-first)*2+step)/(2*step)).clamp(0,count as i32-1) as u32
        }
        Kind::Opacity(field)=>((*field(&mut c)*10.+0.5) as i32-1).clamp(0,9) as u32,
        Kind::Defaults=>0,
    }
}

unsafe extern "C" fn set(_:*mut c_void,index:u32,value:u32){
    let Some(e)=ROWS.get(index as usize) else{return};
    STATE.with(|s|{
        let c=&mut s.config;
        match e.kind{
            Kind::Switch(field)=>*field(c)=value!=0,
            Kind::Language=>if let Some(&l)=LANGUAGE_ORDER.get(value as usize){c.language=l;},
            Kind::Number{first,step,count,field,..}=>if value<count{*field(c)=first+value as i32*step;},
            Kind::Opacity(field)=>*field(c)=(value.min(9)+1) as f32/10.,
            Kind::Defaults=>{}
        }
        // A new layout or language starts its pages over.
        s.page=0;s.page_key=0;
    });
}

fn save(){
    let c=STATE.with(|s|s.config);
    if let Err(e)=crate::save_config(c){forge::log::prelude::warn!("[better-skillinfo] Mod Options: {}",e);}
}
unsafe extern "C" fn closed(_:*mut c_void){save()}
unsafe extern "C" fn action(_:*mut c_void,index:u32){
    if !matches!(ROWS.get(index as usize).map(|e|e.kind),Some(Kind::Defaults)){return}
    STATE.with(|s|{s.config=Config::DEFAULT;s.page=0;s.page_count=1;s.page_key=0;});
    save();
}

struct Record<T>(T);
unsafe impl<T> Sync for Record<T> {}
static OPTIONS:Record<Options>=Record(Options{
    size:core::mem::size_of::<Options>() as u32,context:core::ptr::null_mut(),title:Some(title),
    row_count:ROWS.len() as u32,row:Some(row),get:Some(get),set:Some(set),closed:Some(closed),value_text:None,action:Some(action),
});

static HELP_PAGES:[Str;3]=[
    s!("Shows each skill's activation points,\r\nthe skills they turn on and what those\r\ndo, beside decorations and equipment."),
    s!("Left is the panel at the Smithy, Center\r\nthe one over the Item Box. Sizes and\r\npositions use the game's 1280x720 units."),
    s!("Changes show at once. They are saved\r\nto better_skillinfo.ini next to your\r\nsave when you close the window."),
];
unsafe extern "C" fn help_page(_:*mut c_void,_:u32,index:u32)->*const c_char{
    HELP_PAGES.get(index as usize).map_or(core::ptr::null(),|p|p.0)
}
static SET_HELP:Record<Help>=Record(Help{
    size:core::mem::size_of::<Help>() as u32,context:core::ptr::null_mut(),category:0,
    page_count:HELP_PAGES.len() as u32,title:Some(title),page:Some(help_page),open:None,
});

#[unsafe(no_mangle)]
pub unsafe extern "C" fn purplelib_on_register(api:*const Api){
    let Some(api)=(unsafe{api.as_ref()}) else{return};
    if api.version<API_VERSION{forge::log::prelude::warn!("[better-skillinfo] Purple-lib is too old: no Mod Options page");return}
    let Some(add)=api.add_options else{return};
    if unsafe{add(&OPTIONS.0)}{
        if let Some(help)=api.add_set_help{unsafe{help(&OPTIONS.0,&SET_HELP.0);}}
        crate::log_info!("[better-skillinfo] settings under Options > Mod Options");
    }else{
        forge::log::prelude::warn!("[better-skillinfo] Purple-lib refused the Mod Options page");
    }
}
