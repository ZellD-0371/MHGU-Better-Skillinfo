"""Package the MHGU Forge skill information overlay without game asset replacements."""
import hashlib,json,pathlib,shutil,zipfile
PROJECT=pathlib.Path(__file__).resolve().parents[1]
OUT=PROJECT/'dist/Better_Skillinfo'
files={
    'romfs/nativeNX/plugins/better_skillinfo.nro':PROJECT/'better_skillinfo.nro',
    'romfs/nativeNX/plugins/better-skillinfo/BetterSkillinfo-Subset.otf':PROJECT/'assets/BetterSkillinfo-Subset.otf',
    'NOTO_LICENSE.txt':PROJECT/'assets/NOTO_LICENSE.txt',
}
for resource in (PROJECT/'assets/languages').glob('*.bin'):
    files['romfs/nativeNX/plugins/better-skillinfo/languages/'+resource.name]=resource
manifest={}
for name,src in files.items():
    dest=OUT/name
    dest.parent.mkdir(parents=True,exist_ok=True)
    shutil.copy2(src,dest)
    manifest[name]=hashlib.sha256(dest.read_bytes()).hexdigest()
(PROJECT/'dist/manifest_sha256.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
with zipfile.ZipFile(PROJECT/'Better_Skillinfo.zip','w',zipfile.ZIP_DEFLATED) as archive:
    for name in files:archive.write(OUT/name,'Better_Skillinfo/'+name)
print(json.dumps({'files':len(files),'archive_bytes':(PROJECT/'Better_Skillinfo.zip').stat().st_size}))
