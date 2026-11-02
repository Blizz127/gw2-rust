# Guarded in-memory patches for the frozen build-209026 client.
# See tools/gw2-runner/gw2-runner. Prints only status lines, never values.
set pagination off
set confirm off
set follow-fork-mode parent
set detach-on-fork on
handle all nostop noprint pass
set breakpoint pending on
python
import gdb, hashlib, os, struct

EXE_SHA256 = '99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621'
TRUST_VA, TRUST_LEN = 0x14230b4b7, 128
DH_VA, DH_LEN, DH_PUBLIC_AT = 0x142111220, 136, 72

exe = open(os.environ['GW2_RUNNER_EXE'], 'rb').read()
if hashlib.sha256(exe).hexdigest() != EXE_SHA256:
    print('GW2_RUNNER: exe is not the frozen build 209026; nothing started')
    gdb.execute('quit 3')

def file_bytes(va, size):
    pe = struct.unpack_from('<I', exe, 0x3c)[0]
    count = struct.unpack_from('<H', exe, pe + 6)[0]
    optsz = struct.unpack_from('<H', exe, pe + 20)[0]
    base = struct.unpack_from('<Q', exe, pe + 24 + 24)[0]
    rva = va - base
    for i in range(count):
        vsize, vaddr, rsize, raw = struct.unpack_from('<IIII', exe, pe + 24 + optsz + i * 40 + 8)
        if vaddr <= rva and rva + size <= vaddr + rsize:
            return exe[raw + rva - vaddr: raw + rva - vaddr + size]
    raise ValueError('address not in a file-backed section')

ORIGINAL_TRUST = file_bytes(TRUST_VA, TRUST_LEN)
ORIGINAL_DH = file_bytes(DH_VA, DH_LEN)
identity = os.environ['GW2_RUNNER_IDENTITY']
OWN_TRUST = open(os.path.join(identity, 'own-modulus.bin'), 'rb').read()
OWN_DH = open(os.path.join(identity, 'dh-params.bin'), 'rb').read()
if len(OWN_TRUST) != TRUST_LEN or len(OWN_DH) != DH_LEN or OWN_DH[:DH_PUBLIC_AT] != ORIGINAL_DH[:DH_PUBLIC_AT]:
    print('GW2_RUNNER: identity files do not fit this client; nothing started')
    gdb.execute('quit 3')

def abort(what):
    print('GW2_RUNNER: %s guard failed; stopping the client' % what)
    gdb.execute('kill')
    gdb.execute('quit 2')

class TrustPatch(gdb.Breakpoint):
    def stop(self):
        if int(gdb.parse_and_eval('$r8')) & 255 != 11:
            return False
        inf = gdb.selected_inferior()
        if inf.read_memory(TRUST_VA, TRUST_LEN).tobytes() != ORIGINAL_TRUST:
            abort('trust')
        inf.write_memory(TRUST_VA, OWN_TRUST)
        if inf.read_memory(TRUST_VA, TRUST_LEN).tobytes() != OWN_TRUST:
            abort('trust write')
        print('GW2_RUNNER: trust patch applied')
        self.enabled = False
        return False

class DhPatch(gdb.Breakpoint):
    def stop(self):
        inf = gdb.selected_inferior()
        ptr = int(gdb.parse_and_eval('$rdx'))
        size = int(gdb.parse_and_eval('$ecx'))
        if size != DH_LEN or ptr != DH_VA or inf.read_memory(ptr, size).tobytes() != ORIGINAL_DH:
            abort('DH')
        inf.write_memory(ptr + DH_PUBLIC_AT, OWN_DH[DH_PUBLIC_AT:])
        if inf.read_memory(ptr, size).tobytes() != OWN_DH:
            abort('DH write')
        print('GW2_RUNNER: DH patch applied')
        self.enabled = False
        return False

class Arm(gdb.Breakpoint):
    """First IPv4 connect after the image is mapped: install the two patch points."""
    def stop(self):
        try:
            inf = gdb.selected_inferior()
            if int.from_bytes(inf.read_memory(int(gdb.parse_and_eval('$rsi')), 2).tobytes(), 'little') != 2:
                return False
            if inf.read_memory(0x140000000, 2).tobytes() != b'MZ':
                return False
        except gdb.MemoryError:
            return False
        TrustPatch('*0x14158b740', internal=True)
        DhPatch('*0x140ff4480', internal=True)
        print('GW2_RUNNER: patch points armed')
        self.enabled = False
        return False

Arm('connect', internal=True)
end
run
