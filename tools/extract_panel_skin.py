"""Extract the original XX window_sub border as lossless colored rectangle runs.

No renderer-specific texture handles or runtime game resource reads are needed.
"""
import pathlib, sys
ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'tools/korean_port'))
from inspect_inputs import XX, arc_entries, unpack
from inspect_textures import rgba

d = (XX/'romfs/nativeNX/arc/resident.arc').read_bytes()
im = rgba(unpack(d, next(e for e in arc_entries(d) if e['name'] == r'GUI\99_texture\HD_window_BM_MQ_NOMIP')))
out = ['// Generated from MHXX resident.arc / window_sub. Do not edit by hand.']
for name, rect in [('CORNER', (0,23,16,39)), ('EDGE', (14,12,16,20))]:
    tile = im.crop(rect)
    spans = []
    for y in range(tile.height):
        x = 0
        while x < tile.width:
            c = tile.getpixel((x,y)); end = x+1
            while end < tile.width and tile.getpixel((end,y)) == c: end += 1
            if c[3]: spans.append((x,y,end-x,c))
            x = end
    out.append(f'pub const {name}: &[(u8,u8,u8,u32)] = &[')
    for x,y,w,(r,g,b,a) in spans:
        out.append(f'    ({x},{y},{w},0x{a:02x}{b:02x}{g:02x}{r:02x}),')
    out.append('];')
    print(name, len(spans), 'runs')
(ROOT/'MHGU-Better-Skillinfo/src/skin_generated.rs').write_text('\n'.join(out)+'\n',encoding='utf-8')
