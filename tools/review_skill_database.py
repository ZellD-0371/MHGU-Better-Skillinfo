"""Import and review the user's expanded descriptions without changing skill IDs."""
import copy,json,pathlib,re,sys
ROOT=pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/korean_port'))
from skill_typography import narrow

def review(source):
    supplied=json.loads(source.read_text(encoding='utf-8-sig'))
    result=copy.deepcopy(supplied);rows={s['id']:s for s in result};changes=[]
    def set_text(ident,lang,text,reason):
        old=rows[ident]['description'][lang]
        if old!=text:
            rows[ident]['description'][lang]=text
            changes.append(dict(id=ident,name=rows[ident]['name']['ko'],language=lang,before=old,after=text,reason=reason))
    def replace(ident,lang,a,b,reason):
        set_text(ident,lang,rows[ident]['description'][lang].replace(a,b),reason)
    # Fraction of the original speed, not an impossible negative percentage.
    for ident,fraction in [(216,'1/2'),(217,'1/4')]:
        texts={
            'ko':f'붉은색 회복 가능 체력의 자연 회복 속도가 평소의 {fraction}로 감소한다.',
            'en':f'Reduces the natural recovery speed of recoverable red Health to {fraction} of normal.',
            'ja':f'体力ゲージの赤い回復可能部分の自然回復速度が通常の{fraction}になる。',
            'zh-Hans':f'红色可恢复体力的自然回复速度降至通常的{fraction}。',
            'zh-Hant':f'紅色可恢復體力的自然回復速度降至平時的{fraction}。',
            'de':f'Verringert die natürliche Regeneration des roten, wiederherstellbaren Gesundheitsteils auf {fraction} des normalen Werts.',
            'fr':f'Réduit la vitesse de récupération naturelle de la vie rouge récupérable à {fraction} de sa valeur normale.',
            'it':f'Riduce la velocità di recupero naturale della salute rossa recuperabile a {fraction} del normale.',
            'es':f'Reduce la velocidad de recuperación natural de la salud roja recuperable a {fraction} de lo normal.'}
        for lang,text in texts.items():set_text(ident,lang,text,'Correct recovery-speed ratio')
    for ident in [113,114,115,116,139,149]:
        text=rows[ident]['description']['ko']
        text=re.sub(r'(\d+(?:\.\d+)?)배 증가한다',r'\1배가 된다',text)
        set_text(ident,'ko',text,'Distinguish multiplier from increase')
    replace(115,'ko','산탄의 피해량 1.2배, 확산활의 피해량이 1.3배가 된다.','산탄의 피해량은 1.2배, 확산활의 피해량은 1.3배가 된다.','Parallel phrasing')
    replace(152,'ko','역회심이 발생 시','역회심 발생 시','Grammar')
    for ident in [226,227]:replace(ident,'ko','최대 스태미나 최대치','스태미나 최대치','Remove duplicated maximum')
    for ident in [234,235]:replace(ident,'ko','체력이','체력을','Describe healing, not maximum Health');replace(ident,'ko','증가한다','회복한다','Describe healing, not maximum Health')
    for ident in [209,210]:
        replace(ident,'en','chance to reduces','chance to reduce','English grammar')
        replace(ident,'en','chance to increases','chance to increase','English grammar')
    replace(91,'en','damage multipliers increase by 1.1×, 1.15×, and 1.2×','physical damage is multiplied by 1.1, 1.15, and 1.2','Distinguish multiplier from increase')
    for ident in [218,219]:replace(ident,'en','coatings','Sword & Shield oils','Blade oils are not Bow coatings')
    # Original game action names, rather than literal translations of かけごえ.
    gestures={'en':("'Cheer'","'Fist Pump'"),'de':('„Anfeuern“','„Siegerfaust“'),'fr':('« Encourager »','« Victoire ! »'),'it':('« Incoraggia »','« Vittoria »'),'es':('« Animar »','« Puño en alto »'),'zh-Hans':('“欢呼”','“呼喊”'),'zh-Hant':('「歡呼」','「呼喊」')}
    for lang,(old,new) in gestures.items():replace(197,lang,old,new,'Use original localized gesture name')
    # 妖刀羅刹 is a Long Sword art, not the Switch Axe art Demon Riot.
    for lang,old,new in [('en','Demon Riot','Devouring Demon'),('de','Dämonenklinge','Hungriger Dämon'),('fr','Lame démoniaque','Démon ardent'),('it','Lama demoniaca','Lama Raksha'),('es','Hoja demoníaca','Devorademonio')]:
        replace(185,lang,old,new,'Use corresponding original MHGU Hunter Art name')
    replace(233,'en','Exciteshrooms = Dash Juice. Chaoshrooms','Mopeshrooms = Dash Juice. Exciteshrooms','Match the Japanese mushroom names to the correct English items')
    for lang,replacements in {
        'de':[('Müdigkeitspilz','Trauerpilz'),('Chaos-Pilze','Reizpilze')],
        'fr':[('champi épuisé', 'Champasthénie'),('champignons mystère','Champexciteurs')],
        'it':[('fungo stanco','fungo depressivo'),('funghi misteriosi','funghi stimolanti')],
        'es':[('seta cansada','seta depresiva'),('setas misteriosas','setas excitantes')],
    }.items():
        for a,b in replacements:replace(233,lang,a,b,'Use the corresponding original MHGU mushroom item name')
    # Avoid an added calculation-order assertion not present in the supplied intent.
    for ident in range(62,77):replace(ident,'en',', then multiplies',', and multiplies','Do not introduce an unverified order of operations')
    for s in result:
        # Keep the previous Korean fullwidth-ASCII correction in imported content.
        for field in ['name','description','skill_tree_name']:
            if field in s and 'ko' in s[field]:s[field]['ko']=narrow(s[field]['ko'])
    return supplied,result,changes

if __name__=='__main__':
    source=pathlib.Path(sys.argv[1]);original,result,changes=review(source)
    current=json.loads((ROOT/'Database/mhxx_skills_all_languages.json').read_text(encoding='utf-8'))
    assert len(result)==len(current)==327
    old={s['id']:s for s in current}
    for s in result:
        for key in s:
            if key not in ('description','name','skill_tree_name'):assert s[key]==old[s['id']][key],(s['id'],key)
        assert set(s['description'])=={'ko','en','ja','zh-Hans','zh-Hant','de','fr','it','es'}
        assert all(isinstance(t,str) and t.strip() for t in s['description'].values())
    dest=ROOT/'Database/mhxx_skills_all_languages.json'
    dest.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    report=ROOT/'MHGU-Better-Skillinfo/docs/skill_database_review.json'
    report.write_text(json.dumps(dict(source=str(source),skills=len(result),corrections=changes,sources=['https://game-cap.com/mhx/data/skill47.html','Original MHGU hunterArtsData and skill descriptions']),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(skills=len(result),reviewed_description_fields=sum(len(s['description']) for s in result if s['description']!=old[s['id']]['description']),corrected_fields=len(changes),database=str(dest)),ensure_ascii=True))
