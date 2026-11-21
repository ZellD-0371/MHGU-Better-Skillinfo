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

/// One language's text for the page. The language row lists the mod's languages in
/// Config::language order (the Forge menu's), named in the page's language.
struct Text{names:Labels<21>,helps:Labels<21>,on_off:Labels<2>,reset:Labels<1>,languages:Labels<9>,pages:Labels<3>}
/// The page's text in the mod's language order (Config::language).
static TEXT:[Text;9]=[
    // ko
    Text{
        names:labels!("Better Skillinfo","장식주","방어구","장비 변경","모든 효과 표시","효과 설명","언어","왼쪽: 글자 크기","왼쪽: 너비","왼쪽: 최대 높이","왼쪽: X","왼쪽: Y","왼쪽: 불투명도","가운데: 글자 크기","가운데: 너비","가운데: 최대 높이","가운데: X","가운데: Y","가운데: 불투명도","Skill-Sim 글자 크기","기본값"),
        helps:labels!("스킬 정보 패널을 표시합니다.","장식주의 스킬 정보를 표시합니다.","생산 화면에서 방어구의 스킬 정보를 표시합니다.","장비를 바꿀 때 스킬 변화를 비교합니다.","포인트가 닿지 않은 효과도 함께 표시합니다.","각 스킬 효과의 설명을 표시합니다.","스킬 패널에 쓸 언어입니다.","왼쪽 패널(생산 화면)의 글자 크기입니다.","왼쪽 패널의 너비입니다.","왼쪽 패널이 페이지를 나누기 전 최대 높이입니다.","화면 왼쪽 끝에서 왼쪽 패널까지의 거리입니다.","화면 위쪽 끝에서 왼쪽 패널까지의 거리입니다.","왼쪽 패널 배경의 불투명도입니다.","가운데 패널(아이템 박스)의 글자 크기입니다.","가운데 패널의 너비입니다.","가운데 패널이 페이지를 나누기 전 최대 높이입니다.","화면 가운데에서 가운데 패널이 벗어난 거리입니다.","화면 위쪽 끝에서 가운데 패널까지의 거리입니다.","가운데 패널 배경의 불투명도입니다.","Skill-Simulator가 열려 있을 때의 글자 크기입니다.","모든 설정을 기본값으로 되돌립니다."),
        on_off:labels!("끄기","켜기"),
        reset:labels!("초기화"),
        languages:labels!("한국어","영어","일본어","중국어(간체)","중국어(번체)","독일어","프랑스어","이탈리아어","스페인어"),
        pages:labels!("장식주와 장비 옆에 각 스킬의 발동 포인트,\r\n켜지는 스킬과 그 효과를 표시합니다.","왼쪽은 생산 화면의 패널, 가운데는\r\n아이템 박스 위의 패널입니다. 크기와\r\n위치는 게임의 1280x720 단위입니다.","변경 내용은 바로 반영됩니다. 창을 닫으면\r\n세이브 옆의 better_skillinfo.ini에\r\n저장됩니다."),
    },
    // en
    Text{
        names:labels!("Better Skillinfo","Decorations","Armor","Equipment change","Show all effects","Descriptions","Language","Left: Text size","Left: Width","Left: Max height","Left: X","Left: Y","Left: Opacity","Center: Text size","Center: Width","Center: Max height","Center: X","Center: Y","Center: Opacity","Skill-Sim text size","Defaults"),
        helps:labels!("Show the skill information panel.","Skill info for decorations.","Skill info for armor at the Smithy.","Compare skills when changing equipment.","Also list effects the points don't reach.","Show what each skill effect does.","The language of the skill panel.","Text size of the left panel (Smithy).","Width of the left panel.","Height before the left panel adds pages.","Distance of the left panel from the left.","Distance of the left panel from the top.","Background opacity of the left panel.","Text size of the center panel (Item Box).","Width of the center panel.","Height before the center panel adds pages.","Offset of the center panel from the middle.","Distance of the center panel from the top.","Background opacity of the center panel.","Text size while Skill-Simulator is open.","Put every setting back to its default."),
        on_off:labels!("Off","On"),
        reset:labels!("Reset"),
        languages:labels!("Korean","English","Japanese","Chinese (S)","Chinese (T)","German","French","Italian","Spanish"),
        pages:labels!("Shows each skill's activation points,\r\nthe skills they turn on and what those\r\ndo, beside decorations and equipment.","Left is the panel at the Smithy, Center\r\nthe one over the Item Box. Sizes and\r\npositions use the game's 1280x720 units.","Changes show at once. They are saved\r\nto better_skillinfo.ini next to your\r\nsave when you close the window."),
    },
    // ja
    Text{
        names:labels!("Better Skillinfo","装飾品","防具","装備変更","すべての効果を表示","効果の説明","言語","左: 文字サイズ","左: 幅","左: 最大の高さ","左: X","左: Y","左: 不透明度","中央: 文字サイズ","中央: 幅","中央: 最大の高さ","中央: X","中央: Y","中央: 不透明度","Skill-Sim 文字サイズ","初期値"),
        helps:labels!("スキル情報パネルを表示します。","装飾品のスキル情報を表示します。","加工屋で防具のスキル情報を表示します。","装備を変更するときにスキルを比較します。","ポイントが届かない効果も表示します。","各スキル効果の説明を表示します。","スキルパネルの言語です。","左パネル（加工屋）の文字サイズです。","左パネルの幅です。","左パネルがページを分ける前の最大の高さです。","画面左端から左パネルまでの距離です。","画面上端から左パネルまでの距離です。","左パネルの背景の不透明度です。","中央パネル（アイテムボックス）の文字サイズです。","中央パネルの幅です。","中央パネルがページを分ける前の最大の高さです。","画面中央からの中央パネルのずれです。","画面上端から中央パネルまでの距離です。","中央パネルの背景の不透明度です。","Skill-Simulator表示中の文字サイズです。","すべての設定を初期値に戻します。"),
        on_off:labels!("オフ","オン"),
        reset:labels!("リセット"),
        languages:labels!("韓国語","英語","日本語","中国語（簡体）","中国語（繁体）","ドイツ語","フランス語","イタリア語","スペイン語"),
        pages:labels!("装飾品や装備の横に、各スキルの発動\r\nポイントと発動するスキル、その効果を\r\n表示します。","左は加工屋のパネル、中央はアイテム\r\nボックスのパネルです。サイズと位置は\r\nゲームの1280x720単位です。","変更はすぐに反映されます。ウィンドウを\r\n閉じるとセーブと同じ場所の\r\nbetter_skillinfo.iniに保存されます。"),
    },
    // zh-Hans
    Text{
        names:labels!("Better Skillinfo","装饰品","防具","更换装备","显示全部效果","效果说明","语言","左侧：字体大小","左侧：宽度","左侧：最大高度","左侧：X","左侧：Y","左侧：不透明度","中央：字体大小","中央：宽度","中央：最大高度","中央：X","中央：Y","中央：不透明度","Skill-Sim 字体大小","默认值"),
        helps:labels!("显示技能信息面板。","显示装饰品的技能信息。","在加工商处显示防具的技能信息。","更换装备时比较技能。","也显示点数未达到的效果。","显示各技能效果的说明。","技能面板的语言。","左侧面板（加工商）的字体大小。","左侧面板的宽度。","左侧面板分页前的最大高度。","左侧面板与画面左边的距离。","左侧面板与画面顶部的距离。","左侧面板背景的不透明度。","中央面板（道具箱）的字体大小。","中央面板的宽度。","中央面板分页前的最大高度。","中央面板相对画面中央的偏移。","中央面板与画面顶部的距离。","中央面板背景的不透明度。","Skill-Simulator 打开时的字体大小。","将所有设置恢复为默认值。"),
        on_off:labels!("关","开"),
        reset:labels!("重置"),
        languages:labels!("韩语","英语","日语","简体中文","繁体中文","德语","法语","意大利语","西班牙语"),
        pages:labels!("在装饰品和装备旁显示各技能的发动\r\n点数、发动的技能及其效果。","左侧是加工商的面板，中央是道具箱\r\n的面板。大小和位置使用游戏的\r\n1280x720 单位。","更改会立即生效。关闭窗口时保存到\r\n存档旁的 better_skillinfo.ini。"),
    },
    // zh-Hant
    Text{
        names:labels!("Better Skillinfo","裝飾品","防具","更換裝備","顯示全部效果","效果說明","語言","左側：字體大小","左側：寬度","左側：最大高度","左側：X","左側：Y","左側：不透明度","中央：字體大小","中央：寬度","中央：最大高度","中央：X","中央：Y","中央：不透明度","Skill-Sim 字體大小","預設值"),
        helps:labels!("顯示技能資訊面板。","顯示裝飾品的技能資訊。","在加工商處顯示防具的技能資訊。","更換裝備時比較技能。","也顯示點數未達到的效果。","顯示各技能效果的說明。","技能面板的語言。","左側面板（加工商）的字體大小。","左側面板的寬度。","左側面板分頁前的最大高度。","左側面板與畫面左邊的距離。","左側面板與畫面頂部的距離。","左側面板背景的不透明度。","中央面板（道具箱）的字體大小。","中央面板的寬度。","中央面板分頁前的最大高度。","中央面板相對畫面中央的偏移。","中央面板與畫面頂部的距離。","中央面板背景的不透明度。","Skill-Simulator 開啟時的字體大小。","將所有設定還原為預設值。"),
        on_off:labels!("關","開"),
        reset:labels!("重設"),
        languages:labels!("韓文","英文","日文","簡體中文","繁體中文","德文","法文","義大利文","西班牙文"),
        pages:labels!("在裝飾品和裝備旁顯示各技能的發動\r\n點數、發動的技能及其效果。","左側是加工商的面板，中央是道具箱\r\n的面板。大小和位置使用遊戲的\r\n1280x720 單位。","變更會立即生效。關閉視窗時儲存到\r\n存檔旁的 better_skillinfo.ini。"),
    },
    // de
    Text{
        names:labels!("Better Skillinfo","Dekorationen","Rüstung","Ausrüstungswechsel","Alle Effekte zeigen","Beschreibungen","Sprache","Links: Textgröße","Links: Breite","Links: Max. Höhe","Links: X","Links: Y","Links: Deckkraft","Mitte: Textgröße","Mitte: Breite","Mitte: Max. Höhe","Mitte: X","Mitte: Y","Mitte: Deckkraft","Skill-Sim-Textgröße","Standardwerte"),
        helps:labels!("Zeigt das Fertigkeitsinfo-Fenster.","Fertigkeitsinfos für Dekorationen.","Fertigkeitsinfos für Rüstung beim Schmied.","Vergleicht Fertigkeiten beim Ausrüstungswechsel.","Zeigt auch Effekte, die die Punkte nicht erreichen.","Zeigt, was jeder Effekt bewirkt.","Sprache des Fertigkeitsfensters.","Textgröße des linken Fensters (Schmied).","Breite des linken Fensters.","Höhe, ab der das linke Fenster Seiten bildet.","Abstand des linken Fensters vom linken Rand.","Abstand des linken Fensters vom oberen Rand.","Hintergrunddeckkraft des linken Fensters.","Textgröße des mittleren Fensters (Objekttruhe).","Breite des mittleren Fensters.","Höhe, ab der das mittlere Fenster Seiten bildet.","Versatz des mittleren Fensters von der Mitte.","Abstand des mittleren Fensters vom oberen Rand.","Hintergrunddeckkraft des mittleren Fensters.","Textgröße bei geöffnetem Skill-Simulator.","Setzt alle Einstellungen zurück."),
        on_off:labels!("Aus","An"),
        reset:labels!("Zurücksetzen"),
        languages:labels!("Koreanisch","Englisch","Japanisch","Chinesisch (V)","Chinesisch (T)","Deutsch","Französisch","Italienisch","Spanisch"),
        pages:labels!("Zeigt die Aktivierungspunkte jeder\r\nFertigkeit, die aktivierten Fertigkeiten\r\nund ihre Wirkung neben der Ausrüstung.","Links ist das Fenster beim Schmied, Mitte\r\ndas über der Objekttruhe. Größen und\r\nPositionen in 1280x720-Spieleinheiten.","Änderungen gelten sofort. Beim Schließen\r\nwerden sie in better_skillinfo.ini neben\r\ndem Spielstand gespeichert."),
    },
    // fr
    Text{
        names:labels!("Better Skillinfo","Décorations","Armure","Changement d'équipement","Tous les effets","Descriptions","Langue","Gauche : taille du texte","Gauche : largeur","Gauche : hauteur max.","Gauche : X","Gauche : Y","Gauche : opacité","Centre : taille du texte","Centre : largeur","Centre : hauteur max.","Centre : X","Centre : Y","Centre : opacité","Taille du texte Skill-Sim","Par défaut"),
        helps:labels!("Affiche le panneau d'infos des talents.","Infos des talents pour les décorations.","Infos des talents des armures à la forge.","Compare les talents lors d'un changement d'équipement.","Affiche aussi les effets non atteints.","Affiche l'effet de chaque talent.","Langue du panneau des talents.","Taille du texte du panneau gauche (forge).","Largeur du panneau gauche.","Hauteur avant que le panneau gauche ne pagine.","Distance du panneau gauche au bord gauche.","Distance du panneau gauche au bord supérieur.","Opacité du fond du panneau gauche.","Taille du texte du panneau central (boîte).","Largeur du panneau central.","Hauteur avant que le panneau central ne pagine.","Décalage du panneau central depuis le milieu.","Distance du panneau central au bord supérieur.","Opacité du fond du panneau central.","Taille du texte avec Skill-Simulator ouvert.","Rétablit tous les réglages par défaut."),
        on_off:labels!("Non","Oui"),
        reset:labels!("Réinitialiser"),
        languages:labels!("Coréen","Anglais","Japonais","Chinois (S)","Chinois (T)","Allemand","Français","Italien","Espagnol"),
        pages:labels!("Affiche les points d'activation de chaque\r\ntalent, les talents activés et leur effet,\r\nà côté des décorations et équipements.","À gauche, le panneau de la forge ; au\r\ncentre, celui de la boîte à objets. Tailles\r\net positions en unités 1280x720 du jeu.","Les changements s'appliquent aussitôt et\r\nsont enregistrés dans better_skillinfo.ini\r\nà la fermeture de la fenêtre."),
    },
    // it
    Text{
        names:labels!("Better Skillinfo","Decorazioni","Armatura","Cambio equipaggiamento","Mostra tutti gli effetti","Descrizioni","Lingua","Sinistra: dim. testo","Sinistra: larghezza","Sinistra: altezza max","Sinistra: X","Sinistra: Y","Sinistra: opacità","Centro: dim. testo","Centro: larghezza","Centro: altezza max","Centro: X","Centro: Y","Centro: opacità","Dim. testo Skill-Sim","Predefiniti"),
        helps:labels!("Mostra il pannello info abilità.","Info abilità per le decorazioni.","Info abilità delle armature dal fabbro.","Confronta le abilità quando cambi equipaggiamento.","Mostra anche gli effetti non raggiunti.","Mostra l'effetto di ogni abilità.","Lingua del pannello abilità.","Dimensione testo del pannello sinistro (fabbro).","Larghezza del pannello sinistro.","Altezza oltre la quale il pannello sinistro va a pagine.","Distanza del pannello sinistro dal bordo sinistro.","Distanza del pannello sinistro dal bordo superiore.","Opacità dello sfondo del pannello sinistro.","Dimensione testo del pannello centrale (cassa).","Larghezza del pannello centrale.","Altezza oltre la quale il pannello centrale va a pagine.","Spostamento del pannello centrale dal centro.","Distanza del pannello centrale dal bordo superiore.","Opacità dello sfondo del pannello centrale.","Dimensione testo con Skill-Simulator aperto.","Ripristina tutte le impostazioni predefinite."),
        on_off:labels!("No","Sì"),
        reset:labels!("Ripristina"),
        languages:labels!("Coreano","Inglese","Giapponese","Cinese (S)","Cinese (T)","Tedesco","Francese","Italiano","Spagnolo"),
        pages:labels!("Mostra i punti di attivazione di ogni\r\nabilità, le abilità attivate e il loro\r\neffetto accanto a decorazioni ed equip.","A sinistra il pannello del fabbro, al\r\ncentro quello della cassa oggetti. Misure\r\ne posizioni in unità 1280x720 del gioco.","Le modifiche valgono subito e vengono\r\nsalvate in better_skillinfo.ini alla\r\nchiusura della finestra."),
    },
    // es
    Text{
        names:labels!("Better Skillinfo","Adornos","Armadura","Cambio de equipo","Mostrar todos los efectos","Descripciones","Idioma","Izq.: tamaño de texto","Izq.: ancho","Izq.: altura máx.","Izq.: X","Izq.: Y","Izq.: opacidad","Centro: tamaño de texto","Centro: ancho","Centro: altura máx.","Centro: X","Centro: Y","Centro: opacidad","Tamaño de texto Skill-Sim","Predeterminados"),
        helps:labels!("Muestra el panel de información de habilidades.","Información de habilidades de los adornos.","Información de habilidades de armaduras en la herrería.","Compara habilidades al cambiar de equipo.","Muestra también los efectos no alcanzados.","Muestra qué hace cada efecto.","Idioma del panel de habilidades.","Tamaño de texto del panel izquierdo (herrería).","Ancho del panel izquierdo.","Altura antes de que el panel izquierdo pagine.","Distancia del panel izquierdo al borde izquierdo.","Distancia del panel izquierdo al borde superior.","Opacidad del fondo del panel izquierdo.","Tamaño de texto del panel central (baúl).","Ancho del panel central.","Altura antes de que el panel central pagine.","Desplazamiento del panel central desde el centro.","Distancia del panel central al borde superior.","Opacidad del fondo del panel central.","Tamaño de texto con Skill-Simulator abierto.","Restaura todos los ajustes predeterminados."),
        on_off:labels!("No","Sí"),
        reset:labels!("Restablecer"),
        languages:labels!("Coreano","Inglés","Japonés","Chino (S)","Chino (T)","Alemán","Francés","Italiano","Español"),
        pages:labels!("Muestra los puntos de activación de cada\r\nhabilidad, las habilidades activadas y su\r\nefecto junto a adornos y equipo.","A la izquierda, el panel de la herrería;\r\nen el centro, el del baúl de objetos.\r\nTamaños y posiciones en unidades 1280x720.","Los cambios se aplican al instante y se\r\nguardan en better_skillinfo.ini al\r\ncerrar la ventana."),
    },
];
/// Purple-lib passes the game's language (MT Framework order: 0 Japanese, 1 English,
/// 2 French, 3 Spanish, 4 German, 5 Italian, 6 Korean, 7 Chinese (T), 8 Chinese (S);
/// 6 under the Korean patch, confirmed in the log). The game's font has its glyphs.
fn text(language:u32)->&'static Text{
    const MOD_LANGUAGE:[usize;9]=[2,1,6,8,5,7,0,4,3];
    &TEXT[MOD_LANGUAGE.get(language as usize).copied().unwrap_or(1)]
}

#[derive(Clone,Copy)]
enum Kind{
    Switch(fn(&mut Config)->&mut bool),
    Number{first:i32,step:i32,count:u32,suffix:Option<&'static Str>,field:fn(&mut Config)->&mut i32},
    Opacity(fn(&mut Config)->&mut f32),
    Language,
    Defaults,
}
const fn size(field:fn(&mut Config)->&mut i32)->Kind{Kind::Number{first:5,step:1,count:21,suffix:None,field}}
const fn width(field:fn(&mut Config)->&mut i32)->Kind{Kind::Number{first:180,step:5,count:65,suffix:None,field}}
const fn height(field:fn(&mut Config)->&mut i32)->Kind{Kind::Number{first:200,step:5,count:97,suffix:None,field}}
const fn top(field:fn(&mut Config)->&mut i32)->Kind{Kind::Number{first:0,step:1,count:201,suffix:None,field}}

// Sizes and positions are in the game's 1280x720 screen units, as in the Forge menu.
static ROWS:[Kind;21]=[
    Kind::Switch(|c|&mut c.enabled),
    Kind::Switch(|c|&mut c.deco_enabled),
    Kind::Switch(|c|&mut c.armor_enabled),
    Kind::Switch(|c|&mut c.change_enabled),
    Kind::Switch(|c|&mut c.show_all),
    Kind::Switch(|c|&mut c.descriptions),
    Kind::Language,
    size(|c|&mut c.font_size),
    width(|c|&mut c.width),
    height(|c|&mut c.max_height),
    Kind::Number{first:0,step:1,count:86,suffix:None,field:|c|&mut c.x},
    top(|c|&mut c.y),
    Kind::Opacity(|c|&mut c.opacity),
    size(|c|&mut c.center_font_size),
    width(|c|&mut c.center_width),
    height(|c|&mut c.center_height),
    Kind::Number{first:-200,step:5,count:81,suffix:None,field:|c|&mut c.center_x},
    top(|c|&mut c.center_y),
    Kind::Opacity(|c|&mut c.center_opacity),
    size(|c|&mut c.sim_font_size),
    Kind::Defaults,
];
static PERCENT:Str=s!("%");

unsafe extern "C" fn title(_:*mut c_void,_:u32)->*const c_char{c"Better Skillinfo".as_ptr()}

unsafe extern "C" fn row(_:*mut c_void,language:u32,index:u32,out:*mut Row)->bool{
    let (Some(&kind),Some(out))=(ROWS.get(index as usize),unsafe{out.as_mut()}) else{return false};
    let t=text(language);
    out.name=t.names.0[index as usize];out.help=t.helps.0[index as usize];
    match kind{
        Kind::Switch(_)=>{out.kind=ROW_LABELS;out.labels=t.on_off.0.as_ptr();out.label_count=2;}
        Kind::Language=>{out.kind=ROW_LABELS;out.labels=t.languages.0.as_ptr();out.label_count=9;}
        Kind::Number{first,step,count,suffix,..}=>{
            out.kind=ROW_NUMBERS;out.value_count=count;out.first=first;out.step=step;
            out.suffix=suffix.map_or(core::ptr::null(),|s|s.0);out.flags=NO_WRAP;
        }
        Kind::Opacity(_)=>{out.kind=ROW_NUMBERS;out.value_count=10;out.first=10;out.step=10;out.suffix=PERCENT.0;out.flags=NO_WRAP;}
        Kind::Defaults=>{out.kind=ROW_ACTION;out.labels=t.reset.0.as_ptr();out.label_count=1;}
    }
    true
}

unsafe extern "C" fn get(_:*mut c_void,index:u32)->u32{
    let Some(&kind)=ROWS.get(index as usize) else{return 0};
    let mut c=STATE.with(|s|s.config);
    match kind{
        Kind::Switch(field)=>*field(&mut c) as u32,
        Kind::Language=>c.language.min(8) as u32,
        Kind::Number{first,step,count,field,..}=>{
            // Nearest step, so a value set in the Forge menu between two steps still shows.
            let v=*field(&mut c);(((v-first)*2+step)/(2*step)).clamp(0,count as i32-1) as u32
        }
        Kind::Opacity(field)=>((*field(&mut c)*10.+0.5) as i32-1).clamp(0,9) as u32,
        Kind::Defaults=>0,
    }
}

unsafe extern "C" fn set(_:*mut c_void,index:u32,value:u32){
    let Some(&kind)=ROWS.get(index as usize) else{return};
    STATE.with(|s|{
        let c=&mut s.config;
        match kind{
            Kind::Switch(field)=>*field(c)=value!=0,
            Kind::Language=>if value<9{c.language=value as usize;},
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
    if !matches!(ROWS.get(index as usize),Some(Kind::Defaults)){return}
    STATE.with(|s|{s.config=Config::DEFAULT;s.page=0;s.page_count=1;s.page_key=0;});
    save();
}

struct Record<T>(T);
unsafe impl<T> Sync for Record<T> {}
static OPTIONS:Record<Options>=Record(Options{
    size:core::mem::size_of::<Options>() as u32,context:core::ptr::null_mut(),title:Some(title),
    row_count:ROWS.len() as u32,row:Some(row),get:Some(get),set:Some(set),closed:Some(closed),value_text:None,action:Some(action),
});

unsafe extern "C" fn help_page(_:*mut c_void,language:u32,index:u32)->*const c_char{
    text(language).pages.0.get(index as usize).copied().unwrap_or(core::ptr::null())
}
static SET_HELP:Record<Help>=Record(Help{
    size:core::mem::size_of::<Help>() as u32,context:core::ptr::null_mut(),category:0,
    page_count:3,title:Some(title),page:Some(help_page),open:None,
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
