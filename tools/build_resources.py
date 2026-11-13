"""Externalize localized skill text and subset the bundled font. Requires fonttools."""
import json,re,struct,pathlib,hashlib
ROOT=pathlib.Path(__file__).resolve().parents[1]
LANGS=['ko','en','ja','zh-Hans','zh-Hant','de','fr','it','es']
def main():
    generated=ROOT/'src/generated.rs';source=generated.read_text(encoding='utf-8')
    catalog=ROOT/'resources/texts.json';catalog.parent.mkdir(exist_ok=True)
    if "pub names:[&'static str;9]" in source:
        # Decoration names are not displayed. Aliases below remain shared for game text detection.
        source=re.sub(r'(Deco\{gu_item:\d+,)names:(\[.*?\]),slots:',r'\1slots:',source)
        texts=[]
        def external(m):
            values=json.loads(m[2]);assert len(values)==9
            index=len(texts);texts.append(values)
            return ('name' if m[1]=='names' else 'description')+':'+str(index)
        source=re.sub(r'\b(names|descriptions):(\[(?:"(?:[^"\\]|\\.)*"(?:,)?){9}\])',external,source)
        source=source.replace("pub names:[&'static str;9]",'pub name:u32').replace("pub descriptions:[&'static str;9]",'pub description:u32')
        source=source.replace('pub struct Deco { pub gu_item:u16, pub name:u32,','pub struct Deco { pub gu_item:u16,')
        catalog.write_text(json.dumps(texts,ensure_ascii=False,indent=1)+'\n',encoding='utf-8')
        generated.write_text(source,encoding='utf-8')
    texts=json.loads(catalog.read_text(encoding='utf-8'))
    source=re.sub(r'pub const TEXT_COUNT:usize=\d+;\n','',source)
    source='pub const TEXT_COUNT:usize='+str(len(texts))+';\n'+source
    generated.write_text(source,encoding='utf-8')
    out=ROOT/'assets/languages';out.mkdir(exist_ok=True)
    sizes={}
    for i,lang in enumerate(LANGS):
        strings=[row[i].encode('utf-8') for row in texts];offsets=[0]
        for value in strings:offsets.append(offsets[-1]+len(value))
        data=b'BSI1'+struct.pack('<I',len(strings))+struct.pack('<'+'I'*len(offsets),*offsets)+b''.join(strings)
        (out/(lang+'.bin')).write_bytes(data);sizes[lang]=len(data)
    # Union covers all nine languages, settings, bindings, symbols and dynamic numbers.
    chars=set(''.join(value for row in texts for value in row))
    for path in (ROOT/'src').glob('*.rs'):chars.update(path.read_text(encoding='utf-8'))
    chars.update(chr(n) for n in range(32,127))
    from fontTools import subset
    from fontTools.ttLib import TTFont
    original=ROOT/'assets/NotoSansCJKkr-Regular.otf';target=ROOT/'assets/BetterSkillinfo-Subset.otf'
    font=TTFont(original);cmap=font.getBestCmap();requested={ord(c) for c in chars if ord(c) in cmap}
    opts=subset.Options();opts.layout_features=['*'];opts.name_IDs=['*'];opts.name_languages=['*']
    sub=subset.Subsetter(options=opts);sub.populate(unicodes=requested);sub.subset(font)
    for record in font['name'].names:
        if record.nameID in (1,4,6,16):
            name='BetterSkillinfoSubset' if record.nameID==6 else 'Better Skillinfo Subset'
            record.string=name.encode(record.getEncoding())
    font.save(target)
    actual=TTFont(target).getBestCmap();assert requested<=actual.keys()
    report={'strings_per_language':len(texts),'language_bytes':sizes,'all_language_bytes':sum(sizes.values()),'font_original_bytes':original.stat().st_size,'font_subset_bytes':target.stat().st_size,'glyph_codepoints':len(actual),'font_missing_required':0,'language_sha256':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.glob('*.bin')}}
    (ROOT/'resources/resource_report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':main()
