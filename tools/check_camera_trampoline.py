"""Headless ARM regression: relocated camera prologue vs the original NSO."""
import random, struct, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path[:0] = [str(ROOT / 'tools/wishlist_analysis_deps'), str(ROOT / 'tools/MHGURecomp/tools'), str(ROOT / 'MHGU-Better-Skillinfo/target/testdeps')]
from nso import Nso
from unicorn import Uc, UC_ARCH_ARM, UC_MODE_ARM, UcError
from unicorn.arm_const import *
from elftools.elf.elffile import ELFFile
image = Nso(Path('D:/Games/Bok/MHGU/data/exefs/main')).image
base, site, thunk, stop = 0x200000, 0x5de4b0, 0x4000000, 0x4001000
with (ROOT / 'MHGU-Better-Skillinfo/target/armv7a-none-eabi/release/libbetter_skillinfo.so').open('rb') as f:
    elf = ELFFile(f)
    sym = next(s for s in elf.get_section_by_name('.symtab').iter_symbols() if 'camera_original' in s.name)
    sec = elf.get_section(sym['st_shndx'])
    off = sym['st_value'] - sec['sh_addr']
    code = sec.data()[off:off+sym['st_size']]
assert code == struct.pack('<3I', 0xe92d4800, 0xe1a00002, 0xe12fff13), code.hex()
literal = struct.unpack_from('<I', image, 0x3de4f0)[0]
other = struct.unpack_from('<I', image, 0x3de4f4)[0]
# Literal-relative table pointer slots in the original address space.
slots = [((site+0x14+literal)&0xffffffff), ((site+0x1c+other)&0xffffffff)]
u = Uc(UC_ARCH_ARM, UC_MODE_ARM)
u.mem_map(base, (len(image)+0xfff)&~0xfff); u.mem_write(base,bytes(image))
u.mem_map(thunk,0x3000); u.mem_write(thunk,code)
u.mem_map(0x5000000,0x10000)
for slot,ptr in zip(slots,[0x5000000,0x5000000]): u.mem_write(slot,struct.pack('<I',ptr))
rng=random.Random(106)
inputs=[0,0xffffffff]+[1<<i for i in range(32)]+[rng.getrandbits(32) for _ in range(300)]
for word in inputs:
    expected=0
    for i in range(25):
        mask=1<<i; mapped=(1<<((i*7)%32))
        u.mem_write(0x5000000+i*8,struct.pack('<2I',mapped,mask))
        u.mem_write(0x5001000+i*8,struct.pack('<2I',mapped,0))
        if word&mask: expected|=mapped
    def run(addr,patched):
        u.mem_write(site,struct.pack('<2I',0xe51ff004,thunk) if patched else bytes(image[0x3de4b0:0x3de4b8]))
        u.reg_write(UC_ARM_REG_SP,0x500f000); u.reg_write(UC_ARM_REG_LR,stop)
        u.reg_write(UC_ARM_REG_R0,0x12345678); u.reg_write(UC_ARM_REG_R1,word)
        u.reg_write(UC_ARM_REG_R2,literal); u.reg_write(UC_ARM_REG_R3,site+8)
        u.emu_start(addr,stop,count=1000)
        assert u.reg_read(UC_ARM_REG_SP)==0x500f000
        return u.reg_read(UC_ARM_REG_R0)
    actual=run(site,False)
    assert actual==expected,(hex(word),hex(actual),hex(expected),list(map(hex,slots)))
    assert run(thunk,True)==expected
# Reproduce the old copied-PC-relative-load fault using the same original.
bad=thunk+0x2000
u.mem_write(bad,bytes(image[0x3de4b0:0x3de4b8])+struct.pack('<2I',0xe51ff004,site+8))
u.mem_write(bad+0x40,struct.pack('<I',0xe51ff004))
u.reg_write(UC_ARM_REG_SP,0x500f000);u.reg_write(UC_ARM_REG_LR,stop)
try:
    u.emu_start(bad,stop,count=1000)
except UcError:
    pass
else:
    raise AssertionError('old trampoline unexpectedly survived')
print(f'PASS: old literal-load failure reproduced; built safe thunk matches original for {len(inputs)} words, stack balanced')
