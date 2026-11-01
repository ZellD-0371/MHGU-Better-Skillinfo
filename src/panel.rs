use alloc::{string::{String,ToString},vec::Vec};
use core::{cell::UnsafeCell,fmt::Write};
use forge::{imgui,sys::imgui::{ImVec2,ImVec4,ImGuiCond,ImGuiWindowFlags}};
use crate::{Config,Content,Text,TREES,STATE,color,colored,effect_visible,equipped,pagination};

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
fn draw_values(text:&str,values:&[Value],default:ImVec4){
    let mut start=0;let mut drawn=false;
    for v in values{
        for (part,tint) in [(&text[start..v.start],default),(&text[v.start..v.end],point_color(v.points))]{
            if part.is_empty(){continue}
            if drawn{imgui::same_line(0.,0.);}
            colored(part,tint);drawn=true;
        }
        start=v.end;
    }
    if start<text.len(){if drawn{imgui::same_line(0.,0.);}colored(&text[start..],default);}
}
#[derive(PartialEq)]
struct Key {selection:u64,config:Config,width:u32,capacity:usize,totals:[Option<i32>;32],projected:[Option<i32>;32],projection:crate::Projection,incompatible:bool}
struct Cache {key:Key,pages:Vec<Vec<Row>>}
// Only forge_onImGuiFreeRender accesses this cache, on the UI thread.
struct GuiCache(UnsafeCell<Option<Cache>>);unsafe impl Sync for GuiCache{}
static CACHE:GuiCache=GuiCache(UnsafeCell::new(None));

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
fn build(c:Config,selected:&Content,width:f32,capacity:usize,totals:[Option<i32>;32])->Vec<Vec<Row>>{
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
    for (i,&(id,points)) in selected.points[..selected.count].iter().enumerate(){
        let Some(tree)=TREES.iter().find(|t|t.id==id) else{continue};
        let mut rows=Vec::new();let mut title=String::new();let mut current=String::new();
        let mut left_values=Vec::new();let mut right_values=Vec::new();
        title.push_str(tree.names[l]);
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
                if tree.gu_id==203{let _=write!(effect_line,"{}",effect.names[l]);}else{let _=write!(effect_line,"{} {:+}: {}",if negative{"≤"}else{"≥"},effect.points,effect.names[l]);}
                lines(&mut rows,effect_line.str(),width,if negative{color(1.,0.65,0.6)}else{color(0.85,0.92,1.)});
                if c.descriptions{lines(&mut rows,effect.descriptions[l],width,color(1.,1.,1.));}
            }
        }
        groups.push((headers,rows));
    }
    paginate(&groups,capacity)
}

pub fn render(mut c:Config,mut selected:Content,base:u32,installed:bool){
    let scale=imgui::get_io().DisplaySize.y/720.;if scale<=0.{return}
    if selected.center{
        c.x=((imgui::get_io().DisplaySize.x/scale-c.center_width as f32)/2.) as i32+c.center_x;
        c.y=c.center_y;c.width=c.center_width;c.max_height=c.center_height;c.font_size=c.center_font_size;c.opacity=c.center_opacity;
    }
    let max_height=(c.max_height as f32).min(720.-c.y as f32-16.)*scale;
    if selected.preview{
        // Draw the effective outer bounds independently of the auto-sized content window.
        // This does not create an input target or change layout/pagination.
        imgui::DrawList::foreground().add_rect(
            ImVec2{x:c.x as f32*scale,y:c.y as f32*scale},
            ImVec2{x:(c.x+c.width) as f32*scale,y:c.y as f32*scale+max_height},
            0xff40cfff,0.,(2.*scale).max(1.),forge::sys::imgui::ImDrawFlags::None);
    }
    let pushed=crate::push_font(c.font_size as f32*scale);
    imgui::set_next_window_pos(ImVec2{x:c.x as f32*scale,y:c.y as f32*scale},ImGuiCond::Always,ImVec2{x:0.,y:0.});
    imgui::set_next_window_size(ImVec2{x:c.width as f32*scale,y:0.},ImGuiCond::Always);
    imgui::set_next_window_bg_alpha(c.opacity);
    let flags=ImGuiWindowFlags::NoTitleBar|ImGuiWindowFlags::NoResize|ImGuiWindowFlags::NoMove|ImGuiWindowFlags::NoSavedSettings|ImGuiWindowFlags::AlwaysAutoResize|ImGuiWindowFlags::NoInputs|ImGuiWindowFlags::NoScrollbar|ImGuiWindowFlags::NoScrollWithMouse;
    if imgui::begin(c"MHGU Better_Skillinfo",None,flags){
        let width=(imgui::get_content_region_avail().x-2.).max(80.);
        let row_height=imgui::get_text_line_height_with_spacing();
        let padding=imgui::get_style().WindowPadding.y*2.;
        let hint=["L3 누름 + R ←/→","Hold L3 + R ←/→","L3押し込み + R ←/→","按住L3 + R ←/→","按住L3 + R ←/→","L3 halten + R ←/→","Maintenir L3 + R ←/→","Tieni L3 + R ←/→","Mantén L3 + R ←/→"][c.language];
        let keyboard_hint="Ctrl + ←/→";
        let keyboard_width=imgui::calc_text_size(keyboard_hint,false,0.).x;
        let inline_hints=imgui::calc_text_size(hint,false,0.).x+keyboard_width+16.<=width;
        // Include both hints when wrapping is necessary at narrow widths / large fonts.
        let footer_rows=if selected.preview{3}else if inline_hints{2}else{wrapped(hint,width).len()+2};
        // Repeat the warning on each page and reserve its text plus the gap below.
        let warning=if selected.incompatible{wrapped(crate::locales::INCOMPATIBLE[c.language],width)}else{Vec::new()};
        let warning_height=if warning.is_empty(){0.}else{(warning.len() as f32+0.5)*row_height};
        let capacity=(((max_height-padding-warning_height-footer_rows as f32*row_height-12.)/row_height) as usize).max(4);
        let mut totals=[None;32];
        if installed&&!selected.preview&&selected.projection.shows_totals(){for (i,(id,_)) in selected.points[..selected.count].iter().enumerate(){if let Some(tree)=TREES.iter().find(|t|t.id==*id){totals[i]=equipped::total(base,tree.gu_id);}}}
        if selected.center{for (i,total) in totals[..selected.count].iter_mut().enumerate(){if selected.total_before[i].is_some(){*total=selected.total_before[i];}}}
        if selected.projection==crate::Projection::ReplaceDecoration{
            for (i,(_,before)) in selected.points[..selected.count].iter().enumerate(){
                let after=selected.compared[i].unwrap_or(0);
                selected.total_after[i]=totals[i].map(|n|crate::comparison::replacement(n,*before,after));
            }
        }
        let key=Key{selection:selected.key,config:c,width:width.to_bits(),capacity,totals,projected:selected.total_after,projection:selected.projection,incompatible:selected.incompatible};
        let slot=unsafe{&mut *CACHE.0.get()};
        if slot.as_ref().is_none_or(|cache|cache.key!=key){let pages=build(c,&selected,width,capacity,totals);*slot=Some(Cache{key,pages});}
        let cache=slot.as_ref().unwrap();let count=cache.pages.len();
        let page=STATE.with(|s|{
            if s.page_key!=selected.key{s.page=0;s.page_key=selected.key;}
            s.page_count=count.max(1);s.page=s.page.min(count.saturating_sub(1));
            // Forge's keyboard API remains available while its settings menu is closed.
            // Preview/settings never consume these shortcuts; one arrow press moves one page.
            if !selected.preview&&!forge::graphics::Graphics::is_menu_visible()&&forge::input::Input::is_ctrl_down(){
                let left=forge::input::Input::is_key_pressed(forge::input::Key::Left);
                let right=forge::input::Input::is_key_pressed(forge::input::Key::Right);
                let direction=right as i32-left as i32;
                let next=crate::navigation::Navigation::page(s.page,s.page_count,direction);
                if next!=s.page{crate::sound::request();s.page=next;}
            }
            s.page
        });
        if !warning.is_empty(){
            let left=imgui::get_cursor_pos().x;
            for line in &warning{
                let text_width=imgui::calc_text_size(line,false,0.).x;
                imgui::set_cursor_pos_x(left+((width-text_width)*0.5).max(0.));
                colored(line,color(1.,0.3,0.3));
            }
            imgui::set_cursor_pos_x(left);
            // Draw inside the existing gap without advancing the layout cursor.
            let gap=(row_height*0.5-imgui::get_style().ItemSpacing.y).max(0.);
            let mut start=imgui::get_cursor_screen_pos();
            start.y+=gap*0.5;
            imgui::DrawList::window().add_line(start,ImVec2{x:start.x+width,y:start.y},0x66999999,scale.max(1.));
            imgui::dummy(ImVec2{x:0.,y:gap});
        }
        if let Some(rows)=cache.pages.get(page){
            for row in rows{
                if row.separator{
                    // A full row reserved by pagination gives the line balanced breathing room.
                    let gap=(row_height-imgui::get_style().ItemSpacing.y).max(1.);
                    let mut start=imgui::get_cursor_screen_pos();
                    start.y+=gap*0.5;
                    imgui::DrawList::window().add_line(start,ImVec2{x:start.x+width,y:start.y},0x66999999,scale.max(1.));
                    imgui::dummy(ImVec2{x:0.,y:gap});
                    continue
                }
                let pos=imgui::get_cursor_pos();draw_values(&row.left,&row.left_values,row.color);
                if let Some(right)=&row.right{let w=imgui::calc_text_size(right,false,0.).x;imgui::same_line(pos.x+width-w,0.);draw_values(right,&row.right_values,color(0.9,0.9,0.9));}
            }
        }
        if count>1 {
            imgui::separator();let mut counter=Text::new();let _=write!(counter,"◀ {} / {} ▶",page+1,count);colored(counter.str(),color(1.,0.85,0.3));
            if selected.preview{
                imgui::push_text_wrap_pos(0.);imgui::text(crate::locales::UI[c.language].page.to_str().unwrap_or("Preview page"));imgui::pop_text_wrap_pos();
            }else{
                let pos=imgui::get_cursor_pos();
                imgui::push_text_wrap_pos(0.);imgui::text(hint);imgui::pop_text_wrap_pos();
                if inline_hints{imgui::same_line(pos.x+width-keyboard_width,0.);}
                else{imgui::set_cursor_pos_x(pos.x+width-keyboard_width);}
                imgui::text(keyboard_hint);
            }
        }
    }
    imgui::end();if pushed{imgui::pop_font();}
}
