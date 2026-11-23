"""Verify guarded words against the original NSO and its function boundaries."""
import argparse,bisect,json,re,struct,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'tools/wishlist_analysis_deps'),str(ROOT/'tools/MHGURecomp/tools')]
from nso import Nso,R_ARM_RELATIVE,STT_FUNC
from capstone import Cs,CS_ARCH_ARM,CS_MODE_ARM
parser=argparse.ArgumentParser();parser.add_argument('--nso',default='D:/Games/Bok/MHGU/data/exefs/main');args=parser.parse_args()
n=Nso(Path(args.nso));b=n.image;starts=set()
def signed31(v):
 v&=0x7fffffff
 return v-(1<<31) if v&(1<<30) else v
for at in range(n.eh_start,n.eh_end,8):starts.add(at+signed31(n.u32(at)))
for name,info,shndx,value,size in n.syms:
 if shndx and info&15==STT_FUNC:starts.add(value&~1)
for off,typ,sym,add in n.relocs:
 if typ==R_ARM_RELATIVE and n.text[0]<=add<n.text[0]+n.text[1]:starts.add(add)
# Include direct calls as well as compiler unwind/symbol/vtable starts.
for at in range(n.text[0],n.text[0]+n.text[1],4):
 w=n.u32(at)
 if w&0xff000000==0xeb000000:
  d=w&0xffffff;d=d-(1<<24) if d&(1<<23) else d
  starts.add(at+8+d*4)
starts.add(0x56e810) # wrapper at 56E808 tail-branches after exactly two instructions
starts=sorted(starts);source=(ROOT/'MHGU-Better-Skillinfo/src/hook_guard.rs').read_text()
specs=[(name,int(off,16),[int(x,16) for x in words.split(',')]) for name,off,words in re.findall(r'Spec\{name:"([^"]+)",offset:(0x[0-9a-f]+),words:&\[([^\]]+)\]',source)]
specs += [('menu input',0x3de4f8,[0xe3a02010,0xe3a00040,0xe0022821,0xe0000821]),('camera input',0x3de4b0,[0xe92d4800,0xe59f0034,0xe3a02000,0xe79fe000]),('sound',0x4eeb78,[0xe1a02000,0xe59200c0,0xe3500000,0x012fff1e]),('KatSmithy observer',0x6f1ae8,[0xe92d41f0,0xe24dd018])]
c=Cs(CS_ARCH_ARM,CS_MODE_ARM);report=[]
for name,at,expected in specs:
 actual=list(struct.unpack_from('<'+'I'*len(expected),b,at));assert actual==expected,(name,actual,expected)
 end=starts[bisect.bisect_right(starts,at)];assert at+4*len(expected)<=end,(name,hex(end))
 report.append(dict(feature=name,start=hex(at),guard_end=hex(at+4*len(expected)),next_function=hex(end),instructions=[i.mnemonic+' '+i.op_str for i in c.disasm(b[at:at+4*len(expected)],at)]))
for off,target in [(0x176e344,0x57e3ac),(0x177613c,0x62abf8),(0x1747bb4,0x26e50c)]:
 assert n.u32(off)==target
 assert any(r[0]==off and r[1]==R_ARM_RELATIVE for r in n.relocs),hex(off)
assert list(struct.unpack_from('<2I',b,0x3de4f0))==[0x145e73c,0x145e734]
out=ROOT/'MHGU-Better-Skillinfo/target/hook_boundary_audit.json';out.write_text(json.dumps(dict(build_id=n.build_id,checks=report),indent=2)+'\n')
print(f'PASS: {len(report)} bounded entry guards, 3 relocated layout anchors, camera literals; report {out}')
