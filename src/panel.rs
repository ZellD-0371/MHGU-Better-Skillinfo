use alloc::{string::{String,ToString},vec::Vec};
use core::{cell::UnsafeCell,fmt::Write};
use forge::{imgui,sys::imgui::{ImVec2,ImVec4,ImGuiCond,ImGuiWindowFlags}};
use crate::{Config,Content,Text,TREES,STATE,color,effect_visible,equipped,pagination};

#[derive(Clone)]
struct Row {left:String,right:Option<String>,color:ImVec4,left_values:Vec<Value>,right_values:Vec<Value>,separator:bool}
fn paginate(groups:&[(usize,Vec<Row>)],capacity:usize)->Vec<Vec<Row>>{
    let separator=Row{left:String::new(),right:None,color:color(1.,1.,1.),left_values:Vec::new(),right_values:Vec::new(),separator:true};
    pagination::paginate(groups,capacity,&separator)
}
#[derive(Clone)]
struct Value {start:usize,end:usize,points:i32}
fn point_color(points:i32)->ImVec4{
    if points>0{color(0.4,0.95,0.7)}else if points<0{color(1.,0.55,0.5)}else{color(0.9,0.9,0.9)}
}
fn value(text:&mut String,values:&mut Vec<Value>,points:i32,signed:bool){
    let start=text.len();
    if signed{let _=write!(text,"{}",crate::comparison::Point(points));}else{let _=write!(text,"{}",points);}
    values.push(Value{start,end:text.len(),points});
}
#[derive(PartialEq)]
struct Key {selection:u64,config:Config,width:u32,capacity:usize,totals:[Option<i32>;32],projected:[Option<i32>;32],projection:crate::Projection,incompatible:bool}
struct Cache {key:Key,pages:Vec<Vec<Row>>}
// Only forge_onImGuiFreeRender accesses this cache, on the UI thread.
struct GuiCache(UnsafeCell<Option<Cache>>);unsafe impl Sync for GuiCache{}
static CACHE:GuiCache=GuiCache(UnsafeCell::new(None));
pub fn clear_cache(){unsafe{*CACHE.0.get()=None;}}

fn wrapped(text:&str,width:f32)->Vec<String>{
    let mut result=Vec::new();
    for paragraph in text.lines(){
        let mut rest=paragraph.trim();
        while !rest.is_empty(){
            if imgui::calc_text_size(rest,false,0.).x<=width{result.push(rest.to_string());break}
            let mut cut=0;let mut space=0;
            for (start,ch) in rest.char_indices(){
                let end=start+ch.len_utf8();
                if imgui::calc_text_size(&rest[..end],false,0.).x>width{break}
                cut=end;if ch.is_whitespace(){space=end;}
            }
            if cut==0{cut=rest.chars().next().unwrap().len_utf8()}
            if space>0{cut=space}
            result.push(rest[..cut].trim_end().to_string());rest=rest[cut..].trim_start();
        }
    }
    result
}
fn lines(rows:&mut Vec<Row>,text:&str,width:f32,c:ImVec4){
    value_lines(rows,text,&[],width,c)
}
fn value_lines(rows:&mut Vec<Row>,text:&str,values:&[Value],width:f32,c:ImVec4){
    let mut offset=0;
    for line in wrapped(text,width){
        let start=offset+text[offset..].find(&line).unwrap_or(0);let end=start+line.len();
        let left_values=values.iter().filter_map(|v|{
            let a=v.start.max(start);let b=v.end.min(end);
            (a<b).then(||Value{start:a-start,end:b-start,points:v.points})
        }).collect();
        offset=end;rows.push(Row{left:line,right:None,color:c,left_values,right_values:Vec::new(),separator:false});
    }
}
fn header(rows:&mut Vec<Row>,title:String,left_values:Vec<Value>,current:String,right_values:Vec<Value>,width:f32,tint:ImVec4){
    if imgui::calc_text_size(&title,false,0.).x+imgui::calc_text_size(&current,false,0.).x+16.<width{
        rows.push(Row{left:title,right:Some(current),color:tint,left_values,right_values,separator:false});
    }else{
        value_lines(rows,&title,&left_values,width,tint);
        value_lines(rows,&current,&right_values,width,color(0.9,0.9,0.9));
    }
}
fn build(c:Config,selected:&Content,width:f32,capacity:usize,totals:[Option<i32>;32],language:Option<&crate::language::Data>)->Vec<Vec<Row>>{
    let mut groups=Vec::new();let l=c.language;
    let label=["스킬 포인트","Skill Points","スキルポイント","技能点数","技能點數","Fähigkeitspunkte","Points de talent","Punti abilità","Puntos de habilidad"][l];
    if selected.preview{
        for i in 0..selected.count{
            let mut rows=Vec::new();let mut title=String::new();let mut current=String::new();
            let mut left_values=Vec::new();let mut right_values=Vec::new();
            let _=write!(title,"Preview {} ",i+1);value(&mut title,&mut left_values,1,true);
            let _=write!(current,"{}: ",label);value(&mut current,&mut right_values,0,false);
            current.push_str(" → ");value(&mut current,&mut right_values,1,false);
            let tint=color(0.4,0.95,0.7);
            header(&mut rows,title,left_values,current,right_values,width,tint);
            let headers=rows.len();
            for points in [10,15]{
                let mut effect=Text::new();let _=write!(effect,"≥ +{}: Preview",points);
                lines(&mut rows,effect.str(),width,color(0.85,0.92,1.));
                if c.descriptions{lines(&mut rows,"Preview",width,color(1.,1.,1.));}
            }
            if c.show_all{
                lines(&mut rows,"≤ -10: Preview",width,color(1.,0.65,0.6));
                if c.descriptions{lines(&mut rows,"Preview",width,color(1.,1.,1.));}
            }
            groups.push((headers,rows));
        }
        return paginate(&groups,capacity)
    }
    let Some(language)=language else{let mut rows=Vec::new();lines(&mut rows,"Language file missing or invalid",width,color(1.,0.3,0.3));return alloc::vec![rows]};
    for (i,&(id,points)) in selected.points[..selected.count].iter().enumerate(){
        let Some(tree)=TREES.iter().find(|t|t.id==id) else{continue};
        let mut rows=Vec::new();let mut title=String::new();let mut current=String::new();
        let mut left_values=Vec::new();let mut right_values=Vec::new();
        title.push_str(language.text(tree.name));
        if let Some(other)=selected.compared[i]{
            title.push(' ');value(&mut title,&mut left_values,points,true);
            title.push_str(" → ");value(&mut title,&mut left_values,other,true);
        }else if tree.gu_id!=203{title.push(' ');value(&mut title,&mut left_values,points,true);}
        if !selected.projection.shows_totals(){
            // Informational views omit hunter totals but retain the item skill values.
            value_lines(&mut rows,&title,&left_values,width,point_color(points));
        }else{
            let _=write!(current,"{}: ",label);
            match totals[i]{Some(n)=>value(&mut current,&mut right_values,n,false),None=>current.push_str("--")}
            if selected.projection!=crate::Projection::CurrentOnly{
                current.push_str(" → ");
                match selected.total_after[i]{Some(n)=>value(&mut current,&mut right_values,n,false),None=>current.push_str("--")}
            }
            header(&mut rows,title,left_values,current,right_values,width,point_color(points));
        }
        let headers=rows.len();
        let basis=if selected.projection.shows_totals()&&tree.gu_id!=203{
            let after=if selected.projection==crate::Projection::CurrentOnly{totals[i]}else{selected.total_after[i]};
            crate::comparison::effect_total_basis(totals[i],after)
        }else{None};
        for negative in [false,true]{
            let sign=if negative{-1}else{1};
            let visible=if let Some(n)=basis{effect_visible(n,sign,c.show_all)}else{
                effect_visible(if tree.gu_id==203{1}else{points},sign,c.show_all)
                    ||selected.compared[i].is_some_and(|p|effect_visible(p,sign,c.show_all))
            };
            if !visible{continue}
            let mut effects:Vec<_>=tree.effects.iter().filter(|effect|(effect.points<0)==negative).collect();
            // Negative thresholds progress from the mildest penalty to the strongest.
            // Sort complete effects so each localized description stays with its threshold.
            if negative{effects.sort_by_key(|effect|effect.points.unsigned_abs());}
            for effect in effects{
                let mut effect_line=Text::new();
                if tree.gu_id==203{let _=write!(effect_line,"{}",language.text(effect.name));}else{let _=write!(effect_line,"{} {:+}: {}",if negative{"≤"}else{"≥"},effect.points,language.text(effect.name));}
                lines(&mut rows,effect_line.str(),width,if negative{color(1.,0.65,0.6)}else{color(0.85,0.92,1.)});
                if c.descriptions{lines(&mut rows,language.text(effect.description),width,color(1.,1.,1.));}
            }
        }
        groups.push((headers,rows));
    }
    paginate(&groups,capacity)
}

fn hud(base:u32,installed:bool)->Option<crate::layout::HudViewport>{
    let viewport=unsafe{forge::sys::imgui::igGetMainViewport()};
    let size=if viewport.is_null(){(0.,0.)}else{unsafe{((*viewport).Size.x,(*viewport).Size.y)}};
    let display=imgui::get_io().DisplaySize;
    let (logical_width,logical_height)=crate::layout::viewport_size(size,(display.x,display.y))?;
    (if installed{crate::layout::hud_size(base)}else{None})
        .and_then(|(rw,rh,uw)|crate::layout::hud_viewport(logical_width,logical_height,rw,rh,uw))
        .or_else(||crate::layout::native_viewport(logical_width,logical_height))
}
/// Shared page state and keyboard paging for both panel paths.
fn current_page(selected:&Content,count:usize,c:Config)->usize{
    STATE.with(|s|{
        if s.page_key!=selected.key{s.page=0;s.page_key=selected.key;}
        s.page_count=count.max(1);s.page=s.page.min(count.saturating_sub(1));
        // Forge's keyboard API remains available while its settings menu is closed.
        // Preview/settings never consume these shortcuts; one arrow press moves one page.
        if !selected.preview&&!forge::graphics::Graphics::is_menu_visible(){
            static HELD:core::sync::atomic::AtomicU32=core::sync::atomic::AtomicU32::new(0);
            let now=crate::bindings::key_down(c.keyboard_previous) as u32|((crate::bindings::key_down(c.keyboard_next) as u32)<<1);
            let pressed=now&!HELD.swap(now,core::sync::atomic::Ordering::AcqRel);
            let left=pressed&1!=0;let right=pressed&2!=0;
            let direction=right as i32-left as i32;
            let next=crate::navigation::page(s.page,s.page_count,direction);
            if next!=s.page{crate::sound::request();s.page=next;}
        }
        s.page
    })
}
fn pack(c:ImVec4)->u32{
    let b=|v:f32|(v.clamp(0.,1.)*255.+0.5) as u32;
    (b(c.w)<<24)|(b(c.z)<<16)|(b(c.y)<<8)|b(c.x)
}
type Font=*mut forge::sys::imgui::ImFont;
fn text_at(d:imgui::DrawList,font:Font,size:f32,x:f32,y:f32,col:ImVec4,text:&str){
    if text.is_empty(){return}
    unsafe{forge::sys::imgui::ImDrawList_AddText_FontPtr(d.raw(),font,size,ImVec2{x,y},pack(col),text.as_ptr() as *const _,text.as_ptr().add(text.len()) as *const _,0.,core::ptr::null());}
}
/// Coloured value segments within one line, advanced by measured width.
fn values_at(d:imgui::DrawList,font:Font,size:f32,mut x:f32,y:f32,text:&str,values:&[Value],default:ImVec4){
    let mut start=0;
    for v in values{
        for (part,tint) in [(&text[start..v.start],default),(&text[v.start..v.end],point_color(v.points))]{
            text_at(d,font,size,x,y,tint,part);x+=imgui::calc_text_size(part,false,0.).x;
        }
        start=v.end;
    }
    text_at(d,font,size,x,y,default,&text[start..]);
}

/// Everything one frame of the panel needs; the page rows stay in CACHE.
/// Both paths (ImGui window, Wishlist's foreground list) draw it with `paint`.
struct Prepared{font:Font,size:f32,spacing:f32,padding:f32,width:f32,panel_width:f32,height:f32,warning:Vec<String>,warning_gap:f32,page:usize,count:usize,scale:f32,opacity:f32}
/// Call with the panel font pushed: measurement and drawing share its size.
fn prepare(c:Config,selected:&Content,totals:[Option<i32>;32],scale:f32,max_height:f32)->Option<Prepared>{
    let font=unsafe{forge::sys::imgui::igGetFont()};let size=unsafe{forge::sys::imgui::igGetFontSize()};
    if font.is_null()||!(size>0.){return None}
    let padding=12.*scale;let spacing=imgui::get_style().ItemSpacing.y;let row_height=size+spacing;
    let panel_width=c.width as f32*scale;let width=(panel_width-padding*2.-2.).max(80.);
    // Repeat the warning on each page and reserve its text plus the gap below.
    let warning=if selected.incompatible{wrapped(crate::locales::INCOMPATIBLE[c.language],width)}else{Vec::new()};
    let warning_gap=(row_height*0.5-spacing).max(0.);
    let warning_height=if warning.is_empty(){0.}else{warning.len() as f32*row_height+warning_gap+spacing};
    // Reserve only the centered page counter; bindings remain in settings.
    let capacity=(((max_height-padding*2.-warning_height-row_height-12.)/row_height) as usize).max(4);
    let key=Key{selection:selected.key,config:c,width:width.to_bits(),capacity,totals,projected:selected.total_after,projection:selected.projection,incompatible:selected.incompatible};
    let slot=unsafe{&mut *CACHE.0.get()};
    if slot.as_ref().is_none_or(|cache|cache.key!=key){let pages=crate::language::with(|language|build(c,selected,width,capacity,totals,language));*slot=Some(Cache{key,pages});}
    let count=slot.as_ref().map_or(0,|cache|cache.pages.len());
    let page=current_page(selected,count,c);
    let rows=slot.as_ref().and_then(|cache|cache.pages.get(page)).map_or(0,Vec::len);
    let height=crate::wishlist_model::panel_height(rows,count,padding,size,spacing)+warning_height;
    Some(Prepared{font,size,spacing,padding,width,panel_width,height,warning,warning_gap,page,count,scale,opacity:c.opacity})
}
fn paint(d:imgui::DrawList,x:f32,y:f32,p:&Prepared){
    crate::skin::paint(d,x,y,p.panel_width,p.height,p.scale,p.opacity);
    let (font,size)=(p.font,p.size);let row_height=size+p.spacing;let rule=0x66999999;let thick=p.scale.max(1.);
    let left=x+p.padding;let mut top=y+p.padding;
    if !p.warning.is_empty(){
        for line in &p.warning{
            let w=imgui::calc_text_size(line,false,0.).x;
            text_at(d,font,size,left+((p.width-w)*0.5).max(0.),top,color(1.,0.3,0.3),line);top+=row_height;
        }
        let line=top+p.warning_gap*0.5;
        d.add_line(ImVec2{x:left,y:line},ImVec2{x:left+p.width,y:line},rule,thick);
        top+=p.warning_gap+p.spacing;
    }
    let cache=unsafe{&*CACHE.0.get()};
    let rows=cache.as_ref().and_then(|cache|cache.pages.get(p.page)).map_or(&[][..],|rows|&rows[..]);
    for row in rows{
        if row.separator{
            // A full row reserved by pagination gives the line balanced breathing room.
            let line=top+(row_height-p.spacing).max(1.)*0.5;
            d.add_line(ImVec2{x:left,y:line},ImVec2{x:left+p.width,y:line},rule,thick);
        }else{
            values_at(d,font,size,left,top,&row.left,&row.left_values,row.color);
            if let Some(right)=&row.right{let w=imgui::calc_text_size(right,false,0.).x;values_at(d,font,size,left+p.width-w,top,right,&row.right_values,color(0.9,0.9,0.9));}
        }
        top+=row_height;
    }
    if p.count>1{
        d.add_line(ImVec2{x:left,y:top},ImVec2{x:left+p.width,y:top},rule,thick);top+=1.+p.spacing;
        let mut counter=Text::new();let _=write!(counter,"◀ {} / {} ▶",p.page+1,p.count);
        let w=imgui::calc_text_size(counter.str(),false,0.).x;
        text_at(d,font,size,left+((p.width-w)*0.5).max(0.),top,color(1.,0.85,0.3),counter.str());
    }
}

/// Wishlist's detail page 2. Wishlist and other plugins (KatSmithy) draw on ImGui's
/// foreground list, which covers every ImGui window; Wishlist calls this right after
/// its own window, so the panel is drawn on that list just above Wishlist.
pub fn render_wishlist(){
    if crate::bindings::capturing(){return}
    let (c,selected,base,installed)=STATE.with(|s|(s.config,crate::content(s,false),s.base,s.installed));
    let Some(selected)=selected else{return};
    if selected.projection!=crate::Projection::WishlistEquipment{return}
    let Some(Some(piece))=crate::wishlist_view::piece() else{return};
    let Some(hud)=hud(base,installed) else{return};
    let scale=hud.scale;
    let d=imgui::DrawList::foreground();if d.raw().is_null(){return}
    let (x,y,max_height)=crate::wishlist_model::placement(piece,c.width as f32*scale,c.max_height as f32*scale);
    let pushed=crate::push_font(c.font_size as f32*scale);
    if let Some(p)=prepare(c,&selected,[None;32],scale,max_height){paint(d,x,y,&p);}
    if pushed{imgui::pop_font();}
}

pub fn render(mut c:Config,mut selected:Content,base:u32,installed:bool){
    // Drawn by render_wishlist from Wishlist's own render instead.
    if selected.projection==crate::Projection::WishlistEquipment{return}
    let viewport=unsafe{forge::sys::imgui::igGetMainViewport()};
    let Some(hud)=hud(base,installed) else{return};
    let scale=hud.scale;
    if selected.center{
        c.x=c.center_x;
        c.y=c.center_y;c.width=c.center_width;c.max_height=c.center_height;c.font_size=c.center_font_size;c.opacity=c.center_opacity;
    }
    let (mut panel_x,mut panel_y)=hud.panel_position(selected.center,c.x as f32,c.y as f32,c.width as f32);
    // ImGui uses absolute viewport coordinates, while HUD geometry is local.
    if !viewport.is_null(){unsafe{panel_x+=(*viewport).Pos.x;panel_y+=(*viewport).Pos.y;}}
    let max_height=(c.max_height as f32).min(720.-c.y as f32-16.)*scale;
    if selected.preview{
        // Draw the effective outer bounds independently of the panel itself.
        // This does not create an input target or change layout/pagination.
        imgui::DrawList::foreground().add_rect(
            ImVec2{x:panel_x,y:panel_y},
            ImVec2{x:panel_x+c.width as f32*scale,y:panel_y+max_height},
            0xff40cfff,0.,(2.*scale).max(1.),forge::sys::imgui::ImDrawFlags::None);
    }
    let mut totals=[None;32];
    if installed&&!selected.preview&&selected.projection.shows_totals(){for (i,(id,_)) in selected.points[..selected.count].iter().enumerate(){if let Some(tree)=TREES.iter().find(|t|t.id==*id){totals[i]=equipped::total(base,tree.gu_id);}}}
    if selected.center{for (i,total) in totals[..selected.count].iter_mut().enumerate(){if selected.total_before[i].is_some(){*total=selected.total_before[i];}}}
    if selected.projection==crate::Projection::ReplaceDecoration{
        for (i,(_,before)) in selected.points[..selected.count].iter().enumerate(){
            let after=selected.compared[i].unwrap_or(0);
            selected.total_after[i]=totals[i].map(|n|crate::comparison::replacement(n,*before,after));
        }
    }
    let pushed=crate::push_font(c.font_size as f32*scale);
    let Some(p)=prepare(c,&selected,totals,scale,max_height) else{if pushed{imgui::pop_font();}return};
    // A plain window keeps native-screen panels on the usual ImGui window layer.
    imgui::set_next_window_pos(ImVec2{x:panel_x,y:panel_y},ImGuiCond::Always,ImVec2{x:0.,y:0.});
    imgui::set_next_window_size(ImVec2{x:p.panel_width,y:p.height},ImGuiCond::Always);
    unsafe {
        forge::sys::imgui::igPushStyleVar_Vec2(forge::sys::imgui::ImGuiStyleVar::WindowPadding,ImVec2{x:0.,y:0.});
        forge::sys::imgui::igPushStyleVar_Float(forge::sys::imgui::ImGuiStyleVar::WindowBorderSize,0.);
    }
    let flags=ImGuiWindowFlags::NoTitleBar|ImGuiWindowFlags::NoResize|ImGuiWindowFlags::NoMove|ImGuiWindowFlags::NoSavedSettings|ImGuiWindowFlags::NoInputs|ImGuiWindowFlags::NoScrollbar|ImGuiWindowFlags::NoScrollWithMouse|ImGuiWindowFlags::NoBackground;
    if imgui::begin(c"MHGU Better_Skillinfo",None,flags){
        let d=imgui::DrawList::window();
        let at=unsafe{forge::sys::imgui::igGetWindowPos()};
        unsafe{forge::sys::imgui::ImDrawList_PushClipRectFullScreen(d.raw());}
        paint(d,at.x,at.y,&p);
        unsafe{forge::sys::imgui::ImDrawList_PopClipRect(d.raw());}
    }
    imgui::end();unsafe{forge::sys::imgui::igPopStyleVar(2);}if pushed{imgui::pop_font();}
}
