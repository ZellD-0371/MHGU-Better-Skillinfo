//! User-selected Npad chords. No duplicate/conflict policy is imposed.
use core::sync::atomic::{AtomicU32,Ordering};
use alloc::string::String;
use crate::{Config,imgui};
use forge::sys::imgui::ImVec2;
pub const DEFAULT_PREV:u32=(1<<4)|(1<<20);
pub const DEFAULT_NEXT:u32=(1<<4)|(1<<22);
pub const MASK:u32=0x00ffffff;
static RAW:AtomicU32=AtomicU32::new(0);
pub fn snapshot(buttons:u32,lx:i32,ly:i32,rx:i32,ry:i32)->u32{
    let mut v=buttons&0xffff;
    for (x,y,bit) in [(lx,ly,16),(rx,ry,20)] {
        if x<=-18000{v|=1<<bit} if x>=18000{v|=1<<(bit+2)}
        if y>=18000{v|=1<<(bit+1)} if y<=-18000{v|=1<<(bit+3)}
    }
    v
}
pub fn publish(v:u32){RAW.store(v,Ordering::Release);}
pub fn matches(raw:u32,chord:u32)->bool{chord!=0&&raw&chord==chord}
pub fn label(chord:u32)->String{
    let names=["A","B","X","Y","L3","R3","L","R","ZL","ZR","+","-","D Left","D Up","D Right","D Down","L Stick Left","L Stick Up","L Stick Right","L Stick Down","R Stick Left","R Stick Up","R Stick Right","R Stick Down"];
    let mut out=String::new();
    for (i,n) in names.iter().enumerate(){if chord&(1<<i)!=0{if !out.is_empty(){out.push_str(" + ");}out.push_str(n);}}
    out
}

// Keyboard bindings contain one HID key plus Ctrl/Shift/Alt modifiers.
pub const CTRL:u32=1<<16;
pub const SHIFT:u32=1<<17;
pub const ALT:u32=1<<18;
pub const KEY_PREV:u32=CTRL|80;
pub const KEY_NEXT:u32=CTRL|79;
pub fn validate_key(v:u32)->u32{if valid_hid(v&255){v&0x700ff}else{0}}
// Forge's C ABI accepts HID usage IDs; using integers also covers keys omitted
// by the Rust SDK enum, without constructing invalid enum discriminants.
unsafe extern "C"{#[link_name="forge_input_isKeyDown"] fn hid_down(key:u32)->bool;}
fn valid_hid(n:u32)->bool{(4..=101).contains(&n)||n==103}
fn gui_key(n:u32)->Option<forge::sys::imgui::ImGuiKey>{
    use forge::sys::imgui::ImGuiKey as K;
    let v=match n{
        4..=29=>546+n-4,30..=38=>537+n-30,39=>536,58..=69=>572+n-58,
        40=>K::Enter as u32,41=>K::Escape as u32,42=>K::Backspace as u32,43=>512,
        44=>K::Space as u32,45=>K::Minus as u32,46=>K::Equal as u32,47=>K::LeftBracket as u32,
        48=>K::RightBracket as u32,49=>K::Backslash as u32,51=>K::Semicolon as u32,
        52=>K::Apostrophe as u32,53=>K::GraveAccent as u32,54=>K::Comma as u32,
        55=>K::Period as u32,56=>K::Slash as u32,57=>K::CapsLock as u32,
        50=>K::Oem102 as u32,70=>K::PrintScreen as u32,71=>K::ScrollLock as u32,72=>K::Pause as u32,
        83=>K::NumLock as u32,84=>K::KeypadDivide as u32,85=>K::KeypadMultiply as u32,
        86=>K::KeypadSubtract as u32,87=>K::KeypadAdd as u32,88=>K::KeypadEnter as u32,
        89..=97=>K::Keypad1 as u32+n-89,98=>K::Keypad0 as u32,99=>K::KeypadDecimal as u32,
        100=>K::Oem102 as u32,101=>K::Menu as u32,103=>K::KeypadEqual as u32,
        73=>K::Insert as u32,74=>K::Home as u32,75=>K::PageUp as u32,76=>K::Delete as u32,
        77=>K::End as u32,78=>K::PageDown as u32,79=>K::RightArrow as u32,80=>K::LeftArrow as u32,
        81=>K::DownArrow as u32,82=>K::UpArrow as u32,_=>return None};
    Some(unsafe{core::mem::transmute::<u32,K>(v)})
}
pub fn key_label(v:u32)->String{
    let mut s=String::new();
    for (bit,name) in [(CTRL,"Ctrl"),(SHIFT,"Shift"),(ALT,"Alt")]{if v&bit!=0{if !s.is_empty(){s.push_str(" + ");}s.push_str(name);}}
    if let Some(k)=gui_key(v&255){
        let ptr=unsafe{forge::sys::imgui::igGetKeyName(k)};
        if !ptr.is_null(){let name=match v&255{80=>"<-",79=>"->",81=>"Down",82=>"Up",_=>unsafe{core::ffi::CStr::from_ptr(ptr)}.to_str().unwrap_or("?")};if !s.is_empty(){s.push_str(" + ");}s.push_str(name);}
    }
    if s.is_empty(){s.push('—');}s
}
pub fn key_down(v:u32)->bool{
    use forge::input::Input;
    valid_hid(v&255)&&unsafe{hid_down(v&255)}
        &&(v&CTRL==0||Input::is_ctrl_down())&&(v&SHIFT==0||Input::is_shift_down())&&(v&ALT==0||Input::is_alt_down())
}
fn keyboard_input()->u32{
    use forge::input::Input;
    let mods=if Input::is_ctrl_down(){CTRL}else{0}|if Input::is_shift_down(){SHIFT}else{0}|if Input::is_alt_down(){ALT}else{0};
    let key=(4..=103).filter(|n|valid_hid(*n)).find(|n|*n!=73&&unsafe{hid_down(*n)}).unwrap_or(0);
    mods|key
}
static CAPTURING:core::sync::atomic::AtomicBool=core::sync::atomic::AtomicBool::new(false);
pub fn capturing()->bool{CAPTURING.load(Ordering::Acquire)}
#[derive(Clone,Copy)]
struct Capture{target:u8,ready:bool,candidate:u32,done:bool,last:u32}
impl Capture{
    const EMPTY:Self=Self{target:0,ready:false,candidate:0,done:false,last:0};
    fn has_binding(&self)->bool{self.candidate!=0&&(self.target<=2||self.candidate&255!=0)}
    fn sample(&mut self,raw:u32){
        if raw==0{self.ready=true;self.done=self.candidate!=0;self.last=0;return}
        if !self.ready{return}
        let changed=if self.target<=2{raw&!self.last!=0}else{
            (raw&255!=0&&raw&255!=self.last&255)||raw&!self.last&0x70000!=0
        };
        // Adding a key updates the chord. Releasing part of it keeps the full
        // candidate; starting another gesture replaces it instead of locking it.
        if self.done||self.candidate==0||changed{self.candidate=raw;self.done=false;}
        self.last=raw;
    }
}
#[derive(Clone,Copy,Default)]
struct PadKeyboardGuard{waiting_neutral:bool}
impl PadKeyboardGuard{
    fn filter(&mut self,raw:u32,keyboard:bool)->u32{
        if keyboard{self.waiting_neutral=true;}
        if self.waiting_neutral{
            // Keyboard and Npad caches can update on adjacent frames. Wait for
            // the translated pad release too, so it cannot become a new chord.
            if !keyboard&&raw==0{self.waiting_neutral=false;}
            0
        }else{raw}
    }
}
// Capture is manipulated only by the ImGui thread; the input hook publishes RAW atomically.
static mut CAPTURE:Capture=Capture::EMPTY;
static mut PAD_KEYBOARD_GUARD:PadKeyboardGuard=PadKeyboardGuard{waiting_neutral:false};
fn pad_input()->u32{
    use forge::input::{Input,Button};
    let buttons=[Button::A,Button::B,Button::X,Button::Y,Button::StickL,Button::StickR,Button::L,Button::R,Button::ZL,Button::ZR,Button::Plus,Button::Minus,Button::Left,Button::Up,Button::Right,Button::Down];
    let mut bits=0;for (i,b) in buttons.into_iter().enumerate(){if Input::is_button_down(b){bits|=1<<i;}}
    let (lx,ly)=Input::get_stick_l();let (rx,ry)=Input::get_stick_r();
    // The hook sees this frame before filtering; Forge's cache can be one frame old.
    let raw=RAW.load(Ordering::Acquire)|snapshot(bits,(lx*32767.) as i32,(ly*32767.) as i32,(rx*32767.) as i32,(ry*32767.) as i32);
    let keyboard=Input::is_ctrl_down()||Input::is_shift_down()||Input::is_alt_down()
        ||(4..=231).any(|key|unsafe{hid_down(key)});
    unsafe{
        let mut guard=core::ptr::read(core::ptr::addr_of!(PAD_KEYBOARD_GUARD));
        let filtered=guard.filter(raw,keyboard);
        core::ptr::write(core::ptr::addr_of_mut!(PAD_KEYBOARD_GUARD),guard);
        filtered
    }
}
fn sample_capture()->u32{
    let raw=unsafe{if CAPTURE.target<=2{pad_input()}else{keyboard_input()}};
    unsafe{let mut capture=CAPTURE;capture.sample(raw);CAPTURE=capture;}raw
}
fn capture_text(language:usize,live:Option<u32>){
    let binding=["등록할 조합", "New binding", "登録する組み合わせ", "待绑定组合", "待綁定組合", "Neue Belegung", "Nouvelle combinaison", "Nuova combinazione", "Nueva combinación"];
    let capture=unsafe{CAPTURE};
    let show=|value|if value==0{"-".into()}else if capture.target<=2{label(value)}else{key_label(value)};
    if let Some(raw)=live{
        let labels=["현재 입력","Live input","現在の入力","当前输入","目前輸入","Aktuelle Eingabe","Entrée actuelle","Input attuale","Entrada actual"];
        imgui::text(&alloc::format!("{}: {}",labels[language],show(raw)));
    }else if !capture.has_binding(){return}
    imgui::text(&alloc::format!("{}: {}",binding[language],show(capture.candidate)));
}
fn center_capture(){
    let viewport=unsafe{forge::sys::imgui::igGetMainViewport()};
    let display=imgui::get_io().DisplaySize;
    let (pos,size)=if viewport.is_null(){(ImVec2{x:0.,y:0.},display)}else{unsafe{((*viewport).Pos,(*viewport).Size)}};
    imgui::set_next_window_pos(ImVec2{x:pos.x+size.x*0.5,y:pos.y+size.y*0.5},forge::sys::imgui::ImGuiCond::Always,ImVec2{x:0.5,y:0.5});
}
fn capture_instructions(language:usize){
    let text=["Insert를 눌러 창을 닫고 조합을 입력한 뒤 다시 Insert를 눌러 키 바인딩을 완료하세요.","Press Insert to close the window, enter your combination, then press Insert again to complete the key binding.","Insert でウィンドウを閉じ、組み合わせを入力した後、もう一度 Insert を押してキー割り当てを完了してください。","按 Insert 关闭窗口，输入组合键后，再按 Insert 完成按键绑定。","按 Insert 關閉視窗，輸入組合鍵後，再按 Insert 完成按鍵綁定。","Mit Insert das Fenster schließen, die Kombination eingeben und erneut Insert drücken, um die Tastenbelegung abzuschließen.","Appuyez sur Insert pour fermer la fenêtre, saisissez la combinaison, puis appuyez à nouveau sur Insert pour terminer l’affectation.","Premi Insert per chiudere la finestra, inserisci la combinazione, poi premi di nuovo Insert per completare l’assegnazione.","Pulsa Insert para cerrar la ventana, introduce la combinación y vuelve a pulsar Insert para completar la asignación."];
    let pad=["버튼을 다시 설정하려면 Insert 키를 누르세요. 현재 설정을 적용하려면 확인 버튼을 누르세요.","Press Insert to rebind the buttons. Click Confirm to apply the current binding.","ボタンを再設定するには Insert を押してください。現在の設定を適用するには確定ボタンを押してください。","按 Insert 重新设置按钮。点击确认应用当前设置。","按 Insert 重新設定按鈕。點擊確認套用目前設定。","Insert drücken, um die Tasten neu zu belegen. Zum Übernehmen auf Bestätigen klicken.","Appuyez sur Insert pour réaffecter les boutons. Cliquez sur Confirmer pour appliquer la combinaison actuelle.","Premi Insert per riassegnare i pulsanti. Premi Conferma per applicare la combinazione attuale.","Pulsa Insert para reasignar los botones. Pulsa Confirmar para aplicar la combinación actual."];
    let keyboard=["키를 다시 설정하려면 Insert 키를 누르세요. 현재 설정을 적용하려면 확인 버튼을 누르세요.","Press Insert to rebind the keys. Click Confirm to apply the current binding.","キーを再設定するには Insert を押してください。現在の設定を適用するには確定ボタンを押してください。","按 Insert 重新设置按键。点击确认应用当前设置。","按 Insert 重新設定按鍵。點擊確認套用目前設定。","Insert drücken, um die Tastaturbelegung zu ändern. Zum Übernehmen auf Bestätigen klicken.","Appuyez sur Insert pour réaffecter les touches. Cliquez sur Confirmer pour appliquer la combinaison actuelle.","Premi Insert per riassegnare i tasti. Premi Conferma per applicare la combinazione attuale.","Pulsa Insert para reasignar las teclas. Pulsa Confirmar para aplicar la combinación actual."];
    let capture=unsafe{CAPTURE};
    let message=if capture.has_binding(){if capture.target<=2{pad[language]}else{keyboard[language]}}else{text[language]};
    let width=(imgui::get_io().DisplaySize.x-48.).clamp(180.,560.);
    imgui::push_text_wrap_pos(imgui::get_cursor_pos().x+width);
    imgui::text(message);imgui::pop_text_wrap_pos();
}
/// Always visible during capture, including while Forge's settings are closed.
pub fn capture_overlay(language:usize){
    if unsafe{CAPTURE.target==0}||forge::graphics::Graphics::is_menu_visible(){return}
    let raw=sample_capture();
    use forge::sys::imgui::ImGuiWindowFlags as F;
    let pushed=crate::push_font(18.);
    center_capture();
    if imgui::begin(c"Page binding##capture-overlay",None,F::AlwaysAutoResize|F::NoInputs|F::NoSavedSettings){
        let pad=["원하는 버튼을 누르세요. Insert를 눌러 설정을 완료해주세요.","Press the desired buttons. Press Insert to finish setup.","設定したいボタンを押してください。Insert で設定を完了してください。","请按下所需按钮。按 Insert 完成设置。","請按下所需按鈕。按 Insert 完成設定。","Gewünschte Tasten drücken. Mit Insert die Einrichtung abschließen.","Appuyez sur les boutons souhaités. Appuyez sur Insert pour terminer.","Premi i pulsanti desiderati. Premi Insert per completare la configurazione.","Pulsa los botones deseados. Pulsa Insert para finalizar."];
        let keyboard=["원하는 키를 누르세요. Insert를 눌러 설정을 완료해주세요.","Press the desired keys. Press Insert to finish setup.","設定したいキーを押してください。Insert で設定を完了してください。","请按下所需按键。按 Insert 完成设置。","請按下所需按鍵。按 Insert 完成設定。","Gewünschte Tastaturtasten drücken. Mit Insert die Einrichtung abschließen.","Appuyez sur les touches souhaitées. Appuyez sur Insert pour terminer.","Premi i tasti desiderati. Premi Insert per completare la configurazione.","Pulsa las teclas deseadas. Pulsa Insert para finalizar."];
        imgui::push_text_wrap_pos(imgui::get_cursor_pos().x+(imgui::get_io().DisplaySize.x-48.).clamp(180.,560.));
        imgui::text(if unsafe{CAPTURE.target<=2}{pad[language]}else{keyboard[language]});
        imgui::pop_text_wrap_pos();
        capture_text(language,Some(raw));
    }imgui::end();
    if pushed{imgui::pop_font();}
}
pub fn ui(c:&mut Config){
    let previous=[c"이전 페이지 키 지정",c"Bind previous page",c"前ページの割り当て",c"绑定上一页",c"綁定上一頁",c"Vorherige Seite belegen",c"Assigner page précédente",c"Assegna pagina precedente",c"Asignar página anterior"];
    let next=[c"다음 페이지 키 지정",c"Bind next page",c"次ページの割り当て",c"绑定下一页",c"綁定下一頁",c"Nächste Seite belegen",c"Assigner page suivante",c"Assegna pagina successiva",c"Asignar página siguiente"];
    let device=[["패드","Controller","コントローラー","手柄","控制器","Controller","Manette","Controller","Mando"],["키보드","Keyboard","キーボード","键盘","鍵盤","Tastatur","Clavier","Tastiera","Teclado"]];
    let title=c"Page binding##better-skillinfo";
    for d in 0..2{
        imgui::text(device[d][c.language]);
        for side in 0..2{
            let target=(d*2+side+1) as u8;
            unsafe{forge::sys::imgui::igPushID_Int(target as i32);}
            let button=if side==0{previous[c.language]}else{next[c.language]};
            let value=match target{1=>c.previous_binding,2=>c.next_binding,3=>c.keyboard_previous,_=>c.keyboard_next};
            if imgui::button(button,ImVec2{x:0.,y:0.}){
                unsafe{CAPTURE=Capture{target,..Capture::EMPTY};PAD_KEYBOARD_GUARD=PadKeyboardGuard::default();}
                CAPTURING.store(true,Ordering::Release);RAW.store(0,Ordering::Release);

            }
            // Open the modal at the parent ID scope; IDs above disambiguate the four buttons only.
            unsafe{forge::sys::imgui::igPopID();}
            imgui::same_line(0.,-1.);imgui::text(&if d==0{label(value)}else{key_label(value)});
            // Popup ID must match the ID used by begin_popup_modal below.
            if unsafe{CAPTURE.target==target}&&capturing()&&!unsafe{forge::sys::imgui::igIsPopupOpen_Str(title.as_ptr(),forge::sys::imgui::ImGuiPopupFlags::None)}{imgui::open_popup(title,forge::sys::imgui::ImGuiPopupFlags::None);}
        }
    }
    center_capture();
    if imgui::begin_popup_modal(title,None,forge::sys::imgui::ImGuiWindowFlags::AlwaysAutoResize|forge::sys::imgui::ImGuiWindowFlags::NoNavInputs){
        let target=unsafe{CAPTURE.target};
        // Both devices are sampled exclusively while Forge is closed.
        // Reopening shows the captured candidate without reading new input.
        capture_instructions(c.language);
        capture_text(c.language,None);
        let capture=unsafe{CAPTURE};
        let candidate=capture.candidate;
        let actions=&crate::locales::ACTIONS[c.language];
        if capture.has_binding(){
        if imgui::button(actions.confirm,ImVec2{x:0.,y:0.}){
            match target{1=>c.previous_binding=candidate,2=>c.next_binding=candidate,3=>c.keyboard_previous=candidate,_=>c.keyboard_next=candidate}
            unsafe{CAPTURE=Capture::EMPTY;}CAPTURING.store(false,Ordering::Release);imgui::close_current_popup();
        }
        imgui::same_line(0.,-1.);
        }
        if imgui::button(actions.cancel,ImVec2{x:0.,y:0.}){unsafe{CAPTURE=Capture::EMPTY;}CAPTURING.store(false,Ordering::Release);imgui::close_current_popup();}
        imgui::end_popup();
    }
}
