use core::ffi::CStr;

pub struct Actions{
    pub defaults:&'static CStr,pub reset_title:&'static CStr,pub save_title:&'static CStr,
    pub reset_question:&'static str,pub save_question:&'static str,
    pub confirm:&'static CStr,pub cancel:&'static CStr,pub reset_done:&'static str,
}
pub const ACTIONS:[Actions;9]=[
Actions{defaults:c"기본값",reset_title:c"기본값 초기화##better-skillinfo-reset",save_title:c"설정 저장##better-skillinfo-save",reset_question:"모든 설정을 기본값으로 초기화할까요?",save_question:"현재 설정을 저장할까요?\n기존에 저장한 설정을 덮어씁니다.",confirm:c"확인",cancel:c"취소",reset_done:"기본값으로 초기화했습니다. 유지하려면 설정 저장을 눌러주세요."},
Actions{defaults:c"Defaults",reset_title:c"Restore defaults##better-skillinfo-reset",save_title:c"Save settings##better-skillinfo-save",reset_question:"Reset all settings to their defaults?",save_question:"Save the current settings?\nThis replaces the previously saved settings.",confirm:c"Confirm",cancel:c"Cancel",reset_done:"Defaults restored. Use Save settings to keep them."},
Actions{defaults:c"初期値",reset_title:c"初期値に戻す##better-skillinfo-reset",save_title:c"設定を保存##better-skillinfo-save",reset_question:"すべての設定を初期値に戻しますか？",save_question:"現在の設定を保存しますか？\n保存済みの設定は上書きされます。",confirm:c"確認",cancel:c"キャンセル",reset_done:"初期値に戻しました。保持するには設定を保存してください。"},
Actions{defaults:c"默认值",reset_title:c"恢复默认值##better-skillinfo-reset",save_title:c"保存设置##better-skillinfo-save",reset_question:"将所有设置恢复为默认值吗？",save_question:"保存当前设置吗？\n这将覆盖之前保存的设置。",confirm:c"确认",cancel:c"取消",reset_done:"已恢复默认值。请保存设置以保留更改。"},
Actions{defaults:c"預設值",reset_title:c"還原預設值##better-skillinfo-reset",save_title:c"儲存設定##better-skillinfo-save",reset_question:"將所有設定還原為預設值嗎？",save_question:"儲存目前的設定嗎？\n這將覆寫先前儲存的設定。",confirm:c"確認",cancel:c"取消",reset_done:"已還原預設值。請儲存設定以保留變更。"},
Actions{defaults:c"Standardwerte",reset_title:c"Standardwerte wiederherstellen##better-skillinfo-reset",save_title:c"Einstellungen speichern##better-skillinfo-save",reset_question:"Alle Einstellungen auf Standardwerte zurücksetzen?",save_question:"Aktuelle Einstellungen speichern?\nDie zuvor gespeicherten Einstellungen werden überschrieben.",confirm:c"Bestätigen",cancel:c"Abbrechen",reset_done:"Standardwerte wiederhergestellt. Zum Beibehalten speichern."},
Actions{defaults:c"Par défaut",reset_title:c"Rétablir les paramètres par défaut##better-skillinfo-reset",save_title:c"Enregistrer les paramètres##better-skillinfo-save",reset_question:"Rétablir tous les paramètres par défaut ?",save_question:"Enregistrer les paramètres actuels ?\nLes paramètres précédemment enregistrés seront remplacés.",confirm:c"Confirmer",cancel:c"Annuler",reset_done:"Paramètres par défaut rétablis. Enregistrez pour les conserver."},
Actions{defaults:c"Predefiniti",reset_title:c"Ripristina predefiniti##better-skillinfo-reset",save_title:c"Salva impostazioni##better-skillinfo-save",reset_question:"Ripristinare tutte le impostazioni predefinite?",save_question:"Salvare le impostazioni attuali?\nLe impostazioni salvate in precedenza verranno sovrascritte.",confirm:c"Conferma",cancel:c"Annulla",reset_done:"Impostazioni predefinite ripristinate. Salva per mantenerle."},
Actions{defaults:c"Valores predeterminados",reset_title:c"Restaurar valores predeterminados##better-skillinfo-reset",save_title:c"Guardar ajustes##better-skillinfo-save",reset_question:"¿Restaurar todos los ajustes predeterminados?",save_question:"¿Guardar los ajustes actuales?\nSe sobrescribirán los ajustes guardados anteriormente.",confirm:c"Confirmar",cancel:c"Cancelar",reset_done:"Valores predeterminados restaurados. Guarda para conservarlos."},
];

pub struct Ui {
    pub enabled:&'static CStr, pub language:&'static CStr, pub all:&'static CStr,
    pub descriptions:&'static CStr, pub height:&'static CStr, pub page:&'static CStr, pub width:&'static CStr, pub size:&'static CStr,
    pub opacity:&'static CStr, pub preview:&'static CStr, pub item:&'static CStr,
    pub save:&'static CStr, pub saved:&'static str, pub failed:&'static str, pub unsupported:&'static str,
}
pub const UI:[Ui;9]=[
    Ui{enabled:c"활성화",language:c"Language",all:c"전체 표시",descriptions:c"효과 설명",height:c"최대 높이",page:c"미리보기 페이지",width:c"패널 너비",size:c"글자 크기",opacity:c"배경 불투명도",preview:c"미리보기",item:c"미리보기 스킬 수",save:c"설정 저장",saved:"저장 완료",failed:"저장 실패: 로그를 확인하세요.",unsupported:"지원하지 않는 게임 버전: 자동 추적이 비활성화되었습니다."},
    Ui{enabled:c"Enabled",language:c"Language",all:c"Show all",descriptions:c"Effect descriptions",height:c"Maximum height",page:c"Preview page",width:c"Panel width",size:c"Font size",opacity:c"Background opacity",preview:c"Preview",item:c"Preview skill count",save:c"Save settings",saved:"Settings saved",failed:"Save failed: check the log.",unsupported:"Unsupported game build: automatic tracking is disabled."},
    Ui{enabled:c"有効化",language:c"Language",all:c"すべて表示",descriptions:c"効果の説明",height:c"最大高さ",page:c"プレビューページ",width:c"パネルの幅",size:c"文字サイズ",opacity:c"背景の不透明度",preview:c"プレビュー",item:c"プレビューのスキル数",save:c"設定を保存",saved:"保存しました",failed:"保存に失敗しました。ログを確認してください。",unsupported:"非対応のゲームバージョン：自動追跡は無効です。"},
    Ui{enabled:c"启用",language:c"Language",all:c"显示全部",descriptions:c"效果说明",height:c"最大高度",page:c"预览页面",width:c"面板宽度",size:c"字体大小",opacity:c"背景不透明度",preview:c"预览",item:c"预览技能数量",save:c"保存设置",saved:"设置已保存",failed:"保存失败，请查看日志。",unsupported:"不支持此游戏版本：已禁用自动追踪。"},
    Ui{enabled:c"啟用",language:c"Language",all:c"顯示全部",descriptions:c"效果說明",height:c"最大高度",page:c"預覽頁面",width:c"面板寬度",size:c"字體大小",opacity:c"背景不透明度",preview:c"預覽",item:c"預覽技能數量",save:c"儲存設定",saved:"設定已儲存",failed:"儲存失敗，請查看日誌。",unsupported:"不支援此遊戲版本：已停用自動追蹤。"},
    Ui{enabled:c"Aktiviert",language:c"Language",all:c"Alle anzeigen",descriptions:c"Effektbeschreibungen",height:c"Maximale Höhe",page:c"Vorschauseite",width:c"Fensterbreite",size:c"Schriftgröße",opacity:c"Hintergrunddeckkraft",preview:c"Vorschau",item:c"Anzahl der Vorschaufähigkeiten",save:c"Einstellungen speichern",saved:"Einstellungen gespeichert",failed:"Speichern fehlgeschlagen. Protokoll prüfen.",unsupported:"Nicht unterstützte Spielversion: automatische Erkennung deaktiviert."},
    Ui{enabled:c"Activé",language:c"Language",all:c"Tout afficher",descriptions:c"Description des effets",height:c"Hauteur maximale",page:c"Page de l’aperçu",width:c"Largeur du panneau",size:c"Taille du texte",opacity:c"Opacité du fond",preview:c"Aperçu",item:c"Nombre de talents de l’aperçu",save:c"Enregistrer les paramètres",saved:"Paramètres enregistrés",failed:"Échec de l’enregistrement. Consultez le journal.",unsupported:"Version du jeu non prise en charge : suivi automatique désactivé."},
    Ui{enabled:c"Attivo",language:c"Language",all:c"Mostra tutto",descriptions:c"Descrizione degli effetti",height:c"Altezza massima",page:c"Pagina anteprima",width:c"Larghezza pannello",size:c"Dimensione testo",opacity:c"Opacità dello sfondo",preview:c"Anteprima",item:c"Numero abilità in anteprima",save:c"Salva impostazioni",saved:"Impostazioni salvate",failed:"Salvataggio non riuscito. Controlla il registro.",unsupported:"Versione del gioco non supportata: rilevamento automatico disattivato."},
    Ui{enabled:c"Activado",language:c"Language",all:c"Mostrar todo",descriptions:c"Descripción de efectos",height:c"Altura máxima",page:c"Página de vista previa",width:c"Ancho del panel",size:c"Tamaño del texto",opacity:c"Opacidad del fondo",preview:c"Vista previa",item:c"Cantidad de habilidades de vista previa",save:c"Guardar ajustes",saved:"Ajustes guardados",failed:"Error al guardar. Consulta el registro.",unsupported:"Versión del juego no compatible: seguimiento automático desactivado."},
];

pub struct Extra{pub deco:&'static CStr,pub change:&'static CStr,pub left:&'static CStr,pub center:&'static CStr,pub preview_center:&'static CStr}
pub const EXTRA:[Extra;9]=[
Extra{deco:c"장식주 정보",change:c"장비 변경 정보",left:c"왼쪽 배치 — 생산·NPC 탈착",center:c"중앙 배치 — 아이템 박스",preview_center:c"중앙 배치 미리보기"},
Extra{deco:c"Decoration skills",change:c"Change equipment info",left:c"Left layout — crafting / smithy",center:c"Center layout — item box",preview_center:c"Preview center layout"},
Extra{deco:c"装飾品情報",change:c"装備変更情報",left:c"左配置 — 生産・加工屋",center:c"中央配置 — アイテムボックス",preview_center:c"中央配置プレビュー"},
Extra{deco:c"装饰品信息",change:c"更换装备信息",left:c"左侧布局 — 生产／加工商",center:c"中央布局 — 道具箱",preview_center:c"预览中央布局"},
Extra{deco:c"裝飾品資訊",change:c"更換裝備資訊",left:c"左側配置 — 生產／加工商",center:c"中央配置 — 更換裝備",preview_center:c"預覽中央配置"},
Extra{deco:c"Dekorationsinfos",change:c"Ausrüstungswechsel",left:c"Linkes Layout — Herstellung / Schmied",center:c"Mittiges Layout — Objekttruhe",preview_center:c"Mittiges Layout vorschauen"},
Extra{deco:c"Infos des décorations",change:c"Infos de changement d’équipement",left:c"Disposition gauche — fabrication / forge",center:c"Disposition centrale — boîte à objets",preview_center:c"Aperçu central"},
Extra{deco:c"Info decorazioni",change:c"Info cambio equipaggiamento",left:c"Layout sinistro — creazione / fabbro",center:c"Layout centrale — cassa oggetti",preview_center:c"Anteprima layout centrale"},
Extra{deco:c"Info de adornos",change:c"Info de cambio de equipo",left:c"Diseño izquierdo — fabricación / herrero",center:c"Diseño central — baúl de objetos",preview_center:c"Vista previa central"},
];

/// Equipment does not support the hunter's current weapon type.
pub const INCOMPATIBLE:[&str;9]=[
    "호환되지 않음","Incompatible","互換性なし","不兼容","不相容",
    "Nicht kompatibel","Incompatible","Non compatibile","Incompatible",
];
