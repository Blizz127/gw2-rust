# Guarded in-memory patches for the frozen build-209026 client.
# See tools/gw2-runner/gw2-runner. Prints status lines, the client's own strings and code
# offsets only; never memory values, keys or credentials.
set pagination off
set confirm off
set follow-fork-mode parent
# Forks are detached: the client must run in the traced process itself, as it does under a
# Wine 10-class loader (runs 61-64). GE-Proton 11's Wine instead runs a 32-bit start.exe
# that double-forks the client, so nothing is ever armed there; the script then says so.
set detach-on-fork on
handle all nostop noprint pass
set breakpoint pending on
python
import gdb, hashlib, os, struct, time

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

ARMED_INFERIOR = [None]
SLIDE = [0]
PREFERRED_BASE = 0x140000000
EXE_INODE = os.stat(os.environ['GW2_RUNNER_EXE']).st_ino
EXE_REALPATH = os.path.realpath(os.environ['GW2_RUNNER_EXE'])

def image_base(pid):
    # Newer Wine (GE-Proton 11) maps the dynamic-base client at a random address; only its
    # header page stays file-backed. Find that page (offset 0) by inode or path: on btrfs the
    # device in /proc/pid/maps differs from stat's, so the device is not compared.
    try:
        lines = open('/proc/%d/maps' % pid).read().splitlines()
    except OSError:
        return None
    for line in lines:
        fields = line.split(None, 5)
        if len(fields) < 6 or int(fields[2], 16) != 0:
            continue
        if int(fields[4]) == EXE_INODE or fields[5] == EXE_REALPATH:
            return int(fields[0].split('-')[0], 16)
    return None

def in_client():
    return gdb.selected_inferior().num == ARMED_INFERIOR[0]

def is_x86_64():
    # GE-Proton's files/bin/wine starts in its i386 preloader and only later execs the
    # 64-bit loader (same pid; gdb follows the exec). A stop in that 32-bit phase has no
    # 64-bit registers, so every handler checks the frame's architecture first.
    try:
        return gdb.selected_frame().architecture().name() == 'i386:x86-64'
    except gdb.error:
        return False

def abort(what):
    print('GW2_RUNNER: %s guard failed; stopping the client' % what)
    gdb.execute('kill')
    gdb.execute('quit 2')

class TrustPatch(gdb.Breakpoint):
    def stop(self):
        if not in_client():
            return False
        try:
            selector = int(gdb.parse_and_eval('$r8'))
        except gdb.error:
            abort('trust register')
        if selector & 255 != 11:
            return False
        inf = gdb.selected_inferior()
        trust = TRUST_VA + SLIDE[0]
        if inf.read_memory(trust, TRUST_LEN).tobytes() != ORIGINAL_TRUST:
            abort('trust')
        inf.write_memory(trust, OWN_TRUST)
        if inf.read_memory(trust, TRUST_LEN).tobytes() != OWN_TRUST:
            abort('trust write')
        print('GW2_RUNNER: trust patch applied')
        self.enabled = False
        return False

class DhPatch(gdb.Breakpoint):
    def stop(self):
        if not in_client():
            return False
        inf = gdb.selected_inferior()
        try:
            ptr = int(gdb.parse_and_eval('$rdx'))
            size = int(gdb.parse_and_eval('$ecx'))
        except gdb.error:
            abort('DH register')
        if size != DH_LEN or ptr != DH_VA + SLIDE[0] or inf.read_memory(ptr, size).tobytes() != ORIGINAL_DH:
            abort('DH')
        inf.write_memory(ptr + DH_PUBLIC_AT, OWN_DH[DH_PUBLIC_AT:])
        if inf.read_memory(ptr, size).tobytes() != OWN_DH:
            abort('DH write')
        print('GW2_RUNNER: DH patch applied')
        print('GW2_RUNNER: downloader at DH: %s' % downloader_state(inf))
        self.enabled = False
        return False

# World 1001 reads "Banshee Realm" (owner request; docs/re/209026-world-name.md). The
# world-id-to-name function 0x140312330 gets a jump to a stub in the zero tail of .text:
#   cmp ecx, 1001 / jne L / lea rax, [rip+name] / ret / L: the 12 original prologue bytes
#   / jmp 0x14031233c. Cosmetic only: any mismatch skips it and the run goes on unchanged.
NAME_FN, NAME_PROLOGUE, NAME_GUARD_LEN = 0x140312330, 12, 24
NAME_STUB, NAME_STUB_AREA = 0x141913040, 0x60
NAME_WORLD = 1001
NAME_TEXT = 'Banshee Realm'

def world_name_bytes(slide):
    fn, stub = NAME_FN + slide, NAME_STUB + slide
    original = file_bytes(NAME_FN, NAME_GUARD_LEN)
    text = (NAME_TEXT + '\0').encode('utf-16-le')
    code = b'\x81\xf9' + struct.pack('<I', NAME_WORLD) + b'\x75\x08'      # cmp ecx, id; jne +8
    text_at = stub + 40
    code += b'\x48\x8d\x05' + struct.pack('<i', text_at - (stub + 15)) + b'\xc3'  # lea rax,[rip+t]; ret
    code += original[:NAME_PROLOGUE]
    code += b'\xe9' + struct.pack('<i', fn + NAME_PROLOGUE - (stub + 33))  # jmp fn+12
    code = code.ljust(40, b'\xcc') + text
    entry = b'\xe9' + struct.pack('<i', stub - (fn + 5)) + b'\x90' * (NAME_PROLOGUE - 5)
    return original, code, entry

def apply_world_name(inf, slide):
    if os.environ.get('GW2_RUNNER_WORLD_NAME', '1') == '0':
        print('GW2_RUNNER: world-name patch off')
        return
    try:
        original, code, entry = world_name_bytes(slide)
        fn, stub = NAME_FN + slide, NAME_STUB + slide
        if inf.read_memory(fn, NAME_GUARD_LEN).tobytes() != original:
            print('GW2_RUNNER: world-name guard failed; skipped')
            return
        if inf.read_memory(stub, NAME_STUB_AREA).tobytes() != bytes(NAME_STUB_AREA):
            print('GW2_RUNNER: world-name stub area not empty; skipped')
            return
        inf.write_memory(stub, code)
        if inf.read_memory(stub, len(code)).tobytes() != code:
            inf.write_memory(stub, bytes(len(code)))
            print('GW2_RUNNER: world-name stub write failed; skipped')
            return
        # The jump goes in last, so the function is either untouched or fully redirected.
        inf.write_memory(fn, entry)
        if inf.read_memory(fn, NAME_GUARD_LEN).tobytes() != entry + original[NAME_PROLOGUE:]:
            inf.write_memory(fn, original[:NAME_PROLOGUE])
            print('GW2_RUNNER: world-name entry write failed; restored')
            return
        print('GW2_RUNNER: world-name patch applied')
    except (gdb.MemoryError, gdb.error, ValueError):
        print('GW2_RUNNER: world-name patch skipped (memory)')

# Read-only map-loader stage trace (docs/re/209026-map-ready.md). 0x1409bafd0 is a
# compiled-out marker (spills its four arguments, then ret) that the loader calls on every
# stage transition with ecx = state and rdx = an ASCII name such as
# "MapLoader: STATE_MAP_DOWNLOAD". The marker itself is also the profiler's scope marker and
# fires on every frame, so the breakpoints sit only on the loader's own 22 call sites in
# VdfLoad.cpp (the arguments are already set there). Nothing is written. Each site is guarded
# on its 5 call/jmp bytes. GW2_RUNNER_STAGE_TRACE=0 turns it off.
STAGE_SITES = (0x1409518ec, 0x140951907, 0x140951991, 0x1409519ac, 0x140951a1d, 0x140951a71,
               0x140951a8c, 0x140951c12, 0x140951c2d, 0x140952009, 0x140952024, 0x14095219d,
               0x1409525e0, 0x1409525fb, 0x140952646, 0x140952a0b, 0x140952a3e, 0x140952a59,
               0x140952e9a, 0x140952eb5, 0x140953011, 0x14095377c)

class StageTrace(gdb.Breakpoint):
    def stop(self):
        try:
            if not in_client() or not is_x86_64():
                return False
            inf = gdb.selected_inferior()
            state = int(gdb.parse_and_eval('$ecx'))
            raw = inf.read_memory(int(gdb.parse_and_eval('$rdx')), 64).tobytes().split(b'\0', 1)[0]
            name = raw.decode('ascii', 'replace') if all(32 <= b < 127 for b in raw) else '?'
            # The site tells entry, success exit and error exit apart (e.g. MAP_DOWNLOAD:
            # 0x140951a8c entry, 0x1409525e0 done, 0x14095219d / 0x140952646 error -> state 6).
            site = int(gdb.parse_and_eval('$pc')) - SLIDE[0]
            print('GW2_RUNNER: stage %s %s (%d) at %#x' % (time.strftime('%H:%M:%SZ', time.gmtime()), name, state, site))
        except (gdb.MemoryError, gdb.error):
            pass
        return False

def arm_stage_trace(inf, slide):
    if os.environ.get('GW2_RUNNER_STAGE_TRACE', '1') == '0':
        return
    try:
        for site in STAGE_SITES:
            if inf.read_memory(site + slide, 5).tobytes() != file_bytes(site, 5):
                print('GW2_RUNNER: stage trace guard failed; skipped')
                return
    except (gdb.MemoryError, gdb.error, ValueError):
        print('GW2_RUNNER: stage trace skipped (memory)')
        return
    for site in STAGE_SITES:
        StageTrace('*%#x' % (site + slide), internal=True)
    print('GW2_RUNNER: stage trace armed (%d loader sites)' % len(STAGE_SITES))

# Read-only AssetDownloader check (docs/re/209026-map-ready.md, MAP_DOWNLOAD). MAP_DOWNLOAD
# waits for the map's manifest preparers, which only make progress through the downloader
# instance at 0x1427fad70; registration (0x140d88140) silently does nothing when it is null.
# Local run 77: set at the first connect, registration twice during the manifest fetches.
# Prints the slot state (set/null, never the pointer) and the first 8 registrations.
# Nothing is written. Guarded on the registration prologue. Off with GW2_RUNNER_STAGE_TRACE=0.
DL_SLOT, DL_REGISTER = 0x1427fad70, 0x140d88140

def downloader_state(inf):
    try:
        return 'null' if int.from_bytes(inf.read_memory(DL_SLOT + SLIDE[0], 8).tobytes(), 'little') == 0 else 'set'
    except (gdb.MemoryError, gdb.error):
        return '?'

class DownloaderRegistration(gdb.Breakpoint):
    hits = 0
    def stop(self):
        if not in_client() or not is_x86_64():
            return False
        DownloaderRegistration.hits += 1
        if DownloaderRegistration.hits <= 8:
            print('GW2_RUNNER: downloader registration %d at %s; downloader %s' % (
                DownloaderRegistration.hits, time.strftime('%H:%M:%SZ', time.gmtime()), downloader_state(gdb.selected_inferior())))
        return False

def arm_downloader_check(inf, slide):
    if os.environ.get('GW2_RUNNER_STAGE_TRACE', '1') == '0':
        return
    try:
        if inf.read_memory(DL_REGISTER + slide, 8).tobytes() != file_bytes(DL_REGISTER, 8):
            print('GW2_RUNNER: downloader check guard failed; skipped')
            return
    except (gdb.MemoryError, gdb.error, ValueError):
        print('GW2_RUNNER: downloader check skipped (memory)')
        return
    DownloaderRegistration('*%#x' % (DL_REGISTER + slide), internal=True)
    print('GW2_RUNNER: downloader at arm: %s' % downloader_state(inf))

# Read-only crash report. 0x1409dc570 is ExeError.cpp's error sink: all 8 of its callers
# (the assertion handler 0x1409db7a0 with its 46211 call sites, e.g. "speed >= 0" in
# CoCharSimpleCli.cpp, plus the fatal/formatted error paths) pass ecx = kind, rdx = text,
# r8 = source path, r9d = line. 0x1409dc720 is the unhandled-exception filter installed at
# 0x1409dbdee (rcx = EXCEPTION_POINTERS). Both print the client's own strings and code
# addresses as image offsets (minus the slide), plus up to 12 return addresses from the
# stack. Nothing is written. Guarded on the first 8 bytes; each disables itself at its cap.
# Off with GW2_RUNNER_CRASH_REPORT=0.
ERROR_SINK, CRASH_FILTER = 0x1409dc570, 0x1409dc720
TEXT_START, TEXT_END = 0x140001000, 0x141913000

def text_offset(addr):
    addr -= SLIDE[0]
    return addr if TEXT_START <= addr < TEXT_END else None

def c_string(inf, addr, limit=200):
    try:
        raw = inf.read_memory(addr, limit).tobytes().split(b'\0', 1)[0]
    except (gdb.MemoryError, gdb.error):
        return '?'
    return raw.decode('ascii', 'replace') if all(32 <= b < 127 for b in raw) else '?'

def stack_calls(inf, sp):
    found = []
    # 64-byte chunks up to 2 KB: a crash near the top of a stack must keep what it could read
    # (the 03:23Z device report printed "from ?" with one 2 KB read).
    words = b''
    for at in range(sp, sp + 8 * 256, 64):
        try:
            words += inf.read_memory(at, 64).tobytes()
        except (gdb.MemoryError, gdb.error):
            break
    if not words:
        return '?'
    for i in range(0, len(words), 8):
        at = text_offset(int.from_bytes(words[i:i + 8], 'little'))
        if at is not None:
            found.append('%#x' % at)
            if len(found) == 12:
                break
    return ' '.join(found) or '-'

class ErrorReport(gdb.Breakpoint):
    hits = 0
    def stop(self):
        try:
            if not in_client() or not is_x86_64():
                return False
            ErrorReport.hits += 1
            if ErrorReport.hits >= 20:
                self.enabled = False
            inf = gdb.selected_inferior()
            source = c_string(inf, int(gdb.parse_and_eval('$r8'))).rsplit('\\', 1)[-1]
            print('GW2_RUNNER: assert %s kind %d %s:%d "%s" from %s' % (
                time.strftime('%H:%M:%SZ', time.gmtime()), int(gdb.parse_and_eval('$ecx')), source,
                int(gdb.parse_and_eval('$r9d')), c_string(inf, int(gdb.parse_and_eval('$rdx'))),
                stack_calls(inf, int(gdb.parse_and_eval('$rsp')))))
        except (gdb.MemoryError, gdb.error):
            pass
        return False

class CrashReport(gdb.Breakpoint):
    hits = 0
    def stop(self):
        try:
            if not in_client() or not is_x86_64():
                return False
            CrashReport.hits += 1
            if CrashReport.hits >= 5:
                self.enabled = False
            inf = gdb.selected_inferior()
            pointers = int(gdb.parse_and_eval('$rcx'))
            record, context = struct.unpack('<QQ', inf.read_memory(pointers, 16).tobytes())
            code = struct.unpack('<I', inf.read_memory(record, 4).tobytes())[0]
            where = int.from_bytes(inf.read_memory(record + 0x10, 8).tobytes(), 'little')
            rsp = int.from_bytes(inf.read_memory(context + 0x98, 8).tobytes(), 'little')
            at = text_offset(where)
            print('GW2_RUNNER: crash %s code %#010x at %s from %s' % (
                time.strftime('%H:%M:%SZ', time.gmtime()), code, '%#x' % at if at is not None else 'outside the client',
                stack_calls(inf, rsp)))
        except (gdb.MemoryError, gdb.error, struct.error):
            pass
        return False

def arm_crash_report(inf, slide):
    if os.environ.get('GW2_RUNNER_CRASH_REPORT', '1') == '0':
        return
    try:
        for fn in (ERROR_SINK, CRASH_FILTER):
            if inf.read_memory(fn + slide, 8).tobytes() != file_bytes(fn, 8):
                print('GW2_RUNNER: crash report guard failed; skipped')
                return
    except (gdb.MemoryError, gdb.error, ValueError):
        print('GW2_RUNNER: crash report skipped (memory)')
        return
    ErrorReport('*%#x' % (ERROR_SINK + slide), internal=True)
    CrashReport('*%#x' % (CRASH_FILTER + slide), internal=True)
    print('GW2_RUNNER: crash report armed')

# Full-resolution textures (owner device crash 2026-11-04 03:23Z/03:58Z/04:31Z). The texture
# load task 0x140a6f490 (GrTex2d.cpp) drops mip levels when the texture's flags +0x7c & 0x380
# ask for it; those bits come from s_textureQuality 0x14268b5b4 through 0x140a70670. Its
# "generate missing levels" branch then writes levels the image never allocated: the null write
# at 0x140b48c9d (ImgMip.cpp), right after the character's population. The patch makes the
# branch's je at 0x140a6f57a unconditional (90 e9 fa 00 00 00, same target 0x140a6f67a, the
# no-reduction path), so every texture keeps its full mip chain. Costs VRAM only. Guarded on the
# 6 original bytes; skipped on any mismatch. GW2_RUNNER_FULL_TEXTURES=0 turns it off.
TEX_JE = 0x140a6f57a
TEX_ORIGINAL, TEX_PATCHED = b'\x0f\x84\xfa\x00\x00\x00', b'\x90\xe9\xfa\x00\x00\x00'

def apply_full_textures(inf, slide):
    if os.environ.get('GW2_RUNNER_FULL_TEXTURES', '1') == '0':
        return
    try:
        if file_bytes(TEX_JE, 6) != TEX_ORIGINAL or inf.read_memory(TEX_JE + slide, 6).tobytes() != TEX_ORIGINAL:
            print('GW2_RUNNER: full-texture guard failed; skipped')
            return
        inf.write_memory(TEX_JE + slide, TEX_PATCHED)
        if inf.read_memory(TEX_JE + slide, 6).tobytes() != TEX_PATCHED:
            print('GW2_RUNNER: full-texture write did not stick; skipped')
            return
    except (gdb.MemoryError, gdb.error, ValueError):
        print('GW2_RUNNER: full-texture patch skipped (memory)')
        return
    print('GW2_RUNNER: full-texture patch applied')

# BEGIN EMPTY_COMPOSITE_COMPLETION
# Frozen ProcessComposite's empty-only branch skips submission after allocating a
# fresh job/group. Complete only that no-pixel-work job, then let the normal poll,
# destructor and later mip/ready writer run. No UI ownership, cached allocation,
# repair-count or time limit: world composites use the same concrete operation.
# Source/limits: docs/re/209026-empty-composite-runner.md. Set
# GW2_RUNNER_EMPTY_COMPOSITE=0 to leave this path unchanged.
EMPTY_COMPOSITE_SITE = 0x140a74ced
EMPTY_COMPOSITE_GUARDS = {
    0x140a747c0: bytes.fromhex('405355565741544155415641574881ec08030000'),
    0x140a750d5: bytes.fromhex('488d4424604889442428488b433848894424200f29442460e8cef6ffff'),
    0x140a74ce5: bytes.fromhex('85c00f84ba000000498b56584885d275'),
    EMPTY_COMPOSITE_SITE: bytes.fromhex('498b56584885d2'),
    0x140a6fdda: bytes.fromhex('85c00f85b70000004883c428c3488b51'),
    0x14026f540: bytes.fromhex('8b81f8000000c3'),
    0x140a6c720: bytes.fromhex('48895c2418574883ec204883b9000100'),
}
EMPTY_COMPOSITE = None


class EmptyCompositeReject(ValueError):
    pass


class EmptyCompositeCompletion:
    def __init__(self, inf, slide):
        self.inf, self.slide, self.bp = inf, slide, None
        self.logs = 0

    def log(self, status):
        # Only constant status classes. Exhausting logs never disables repair.
        if self.logs < 8:
            print('GW2_RUNNER: empty-composite ' + status)
        elif self.logs == 8:
            print('GW2_RUNNER: empty-composite further status lines suppressed; handler remains armed')
        self.logs = min(self.logs + 1, 9)

    def read(self, address, size):
        if not address or not 0 < size <= 256:
            raise EmptyCompositeReject('invalid bounded read')
        return self.inf.read_memory(address, size).tobytes()

    def q(self, address):
        return struct.unpack('<Q', self.read(address, 8))[0]

    def d(self, address):
        return struct.unpack('<I', self.read(address, 4))[0]

    def reg(self, name):
        return int(gdb.parse_and_eval('$' + name))

    def code_guard(self):
        if gdb.parameter('non-stop'):
            raise EmptyCompositeReject('requires all-stop')
        for address, expected in EMPTY_COMPOSITE_GUARDS.items():
            if self.read(address + self.slide, len(expected)) != expected:
                raise EmptyCompositeReject('instruction mismatch')

    def capture(self):
        self.code_guard()
        if self.reg('rip') != EMPTY_COMPOSITE_SITE + self.slide or self.reg('eax') & 0xffffffff != 1:
            raise EmptyCompositeReject('wrong branch')
        rsp, destination, source = self.reg('rsp'), self.reg('rdi'), self.reg('r14')
        if self.q(rsp + 0x348) != 0x140a750f2 + self.slide:
            raise EmptyCompositeReject('wrong caller')
        op = self.q(rsp + 0x340)
        if not op or op & 3 or self.q(op) != 0x141bfc7c0 + self.slide:
            raise EmptyCompositeReject('wrong operation')
        if (self.q(op + 0x28), self.q(op + 0x30)) != (destination, source):
            raise EmptyCompositeReject('operation input mismatch')
        job, group = self.q(rsp + 0xa8), self.q(rsp + 0xb0)
        if not job or not group or job & 15 or group & 15 or len({job, group, op, destination, source}) != 5:
            raise EmptyCompositeReject('allocation mismatch')
        if (self.q(op + 0x108), self.q(op + 0x100)) != (job, group):
            raise EmptyCompositeReject('operation job mismatch')
        if self.reg('r13') != rsp + 0x3b0 or self.q(rsp + 0x378) != rsp + 0x3b0:
            raise EmptyCompositeReject('rectangle pointer mismatch')
        rectangle = struct.unpack('<4I', self.read(rsp + 0x3b0, 16))
        if rectangle[2] > rectangle[0] and rectangle[3] > rectangle[1]:
            raise EmptyCompositeReject('nonempty rectangle')
        group_state = (self.d(group), self.q(group + 8), self.d(group + 0x10),
                       self.q(group + 0x20), self.q(group + 0x28), self.d(group + 0x30))
        if group_state != (1, 0, 1, 0, group, 1):
            raise EmptyCompositeReject('group not fresh')
        job_state = (self.d(job + 0x18), self.d(job + 0x1c), self.d(job + 0xf8))
        if job_state != (5, 1, 0):
            raise EmptyCompositeReject('job not fresh zero completion')
        outstanding, count = self.d(destination + 0x94), self.d(op + 0x50)
        if not outstanding or outstanding == 0xffffffff or not 1 <= count <= 5:
            raise EmptyCompositeReject('operation count mismatch')
        destination_state = (self.q(destination), self.q(destination + 0x40), self.d(destination + 0x60),
                             self.d(destination + 0x64), self.d(destination + 0x48), self.d(destination + 0x4c))
        table, backing, flags, fmt, _, _ = destination_state
        if table != 0x141bfc700 + self.slide or not backing or not flags & 4 or flags & 0x1200000 or fmt != 0x1a:
            raise EmptyCompositeReject('destination backing mismatch')
        pointers = (source,) + tuple(self.q(op + 0x58 + i * 8) for i in range(count))
        inputs, source_dims = [], None
        for index, data in enumerate(pointers):
            if not data and index == 1:
                inputs.append(None); continue  # Only mask zero may be absent.
            if not data:
                raise EmptyCompositeReject('missing input')
            state = (self.q(data), self.q(data + 0x40), self.d(data + 0x60), self.q(data + 0xa8),
                     self.d(data + 0x64), self.d(data + 0x48), self.d(data + 0x4c), self.d(data + 0x68))
            table, backing, flags, head, fmt, width, height, levels = state
            if table != 0x141bfc700 + self.slide or not backing:
                raise EmptyCompositeReject('input backing mismatch')
            if not flags & 0x10000 or (head and not head & 1):
                raise EmptyCompositeReject('input not ready')
            if fmt != (0x1a if index == 0 else 0x1b):
                raise EmptyCompositeReject('input format mismatch')
            if index == 0:
                if not flags & 6:
                    raise EmptyCompositeReject('primary backing flags')
                source_dims = (width, height)
            elif not levels or width < source_dims[0] or height < source_dims[1]:
                raise EmptyCompositeReject('mask dimensions mismatch')
            inputs.append(state)
        # Every checked dynamic field is part of the unchanged second snapshot.
        return (job, group, op, destination, source, rsp, rectangle, group_state, job_state,
                outstanding, count, destination_state, pointers, tuple(inputs),
                self.read(rsp + 0x370, 8), self.read(op + 0x38, 24))

    def complete(self):
        if not in_client() or not is_x86_64() or gdb.selected_inferior().num != self.inf.num:
            return
        try:
            first = self.capture()
            if self.capture() != first:
                raise EmptyCompositeReject('state changed before write')
        except (EmptyCompositeReject, gdb.MemoryError, gdb.error, struct.error):
            self.log('unqualified invocation left unchanged')
            return
        # No lifetime pointers survive this callback. No target function calls,
        # registers, flags, texture-ready bits or counters are changed.
        try:
            self.inf.write_memory(first[0] + 0xf8, struct.pack('<I', 1))
            if self.d(first[0] + 0xf8) != 1:
                abort('empty-composite completion readback')
                return
        except (gdb.MemoryError, gdb.error, struct.error):
            abort('empty-composite completion write/readback')
            return
        self.log('empty unsubmitted job completed; readback verified')


def empty_composite_dispatch():
    if EMPTY_COMPOSITE is not None:
        EMPTY_COMPOSITE.complete()


def arm_empty_composite(inf, slide):
    global EMPTY_COMPOSITE
    if os.environ.get('GW2_RUNNER_EMPTY_COMPOSITE', '1') == '0':
        print('GW2_RUNNER: empty-composite completion off')
        return
    if EMPTY_COMPOSITE is not None:
        return
    state = EmptyCompositeCompletion(inf, slide)
    try:
        # Arm runs only after the script's SHA-256 and mapped-image checks.
        for address, expected in EMPTY_COMPOSITE_GUARDS.items():
            if file_bytes(address, len(expected)) != expected:
                raise EmptyCompositeReject('file instruction mismatch')
        state.code_guard()
        state.bp = gdb.Breakpoint('*%#x' % (EMPTY_COMPOSITE_SITE + slide), internal=False)
        state.bp.enabled = False
        if state.bp.number <= 0:
            raise EmptyCompositeReject('public breakpoint required')
        state.bp.condition = '$eax == 1 && *(unsigned long long*)($rsp+0x348) == %#x' % (0x140a750f2 + slide)
        gdb.execute('commands %d\nsilent\npython empty_composite_dispatch()\ncontinue\nend' % state.bp.number)
    except (EmptyCompositeReject, gdb.MemoryError, gdb.error, struct.error, ValueError):
        if state.bp is not None and state.bp.is_valid():
            state.bp.delete()
        print('GW2_RUNNER: empty-composite completion guard failed; skipped')
        return
    EMPTY_COMPOSITE = state
    state.bp.enabled = True
    print('GW2_RUNNER: empty-composite completion armed')
# END EMPTY_COMPOSITE_COMPLETION

# Opt-in diagnostic (GW2_RUNNER_TEXTURE_DIAG=1, off by default): just before the load task's
# "generate missing levels" call at 0x140a6f625 (only reached without the full-texture patch),
# print the texture's file id (encoded reference at task [+0x8]: id = w1*0xff00 + w0 + 0xff00ff01),
# the levels present (esi) and wanted (ecx), the format (+0x58) and dims (+0x68). Read-only.
TEX_GEN_SITE = 0x140a6f625

class TextureDiag(gdb.Breakpoint):
    hits = 0
    def stop(self):
        try:
            if not in_client() or not is_x86_64():
                return False
            TextureDiag.hits += 1
            if TextureDiag.hits > 40:
                self.enabled = False
            inf = gdb.selected_inferior()
            task = int(gdb.parse_and_eval('$rdi'))
            ref = int.from_bytes(inf.read_memory(task + 8, 8).tobytes(), 'little')
            w0, w1 = struct.unpack('<HH', inf.read_memory(ref, 4).tobytes())
            fid = (w1 * 0xff00 + w0 + 0xff00ff01) & 0xffffffff
            fmt, dims = struct.unpack('<I', inf.read_memory(task + 0x58, 4).tobytes())[0], struct.unpack('<II', inf.read_memory(task + 0x68, 8).tobytes())
            print('GW2_RUNNER: texture-diag file %d levels %d -> %d format %#x dims %dx%d' % (
                fid, int(gdb.parse_and_eval('$esi')), int(gdb.parse_and_eval('$ecx')), fmt, dims[0], dims[1]))
        except (gdb.MemoryError, gdb.error, struct.error):
            pass
        return False

def arm_texture_diag(inf, slide):
    if os.environ.get('GW2_RUNNER_TEXTURE_DIAG', '0') != '1':
        return
    TextureDiag('*%#x' % (TEX_GEN_SITE + slide), internal=True)
    print('GW2_RUNNER: texture diagnostic armed')

# Opt-in diagnostic (GW2_RUNNER_SEND_TRACE=1, off by default): read-only breakpoint on the
# client's generic message send 0x140ff0110 (rcx = connection, edx = size, r8 = buffer whose
# first Word is the message id). Prints the first 3 sends of each (connection, id) pair.
SEND_FN = 0x140ff0110

class SendTrace(gdb.Breakpoint):
    seen = {}
    def stop(self):
        try:
            if not in_client() or not is_x86_64():
                return False
            inf = gdb.selected_inferior()
            conn = int(gdb.parse_and_eval('$rcx'))
            msg = struct.unpack('<H', inf.read_memory(int(gdb.parse_and_eval('$r8')), 2).tobytes())[0]
            key = (conn, msg)
            n = SendTrace.seen.get(key, 0) + 1
            SendTrace.seen[key] = n
            if n <= 3:
                print('GW2_RUNNER: send %s conn %#x id %d size %d (#%d)' % (
                    time.strftime('%H:%M:%SZ', time.gmtime()), conn & 0xffffff, msg, int(gdb.parse_and_eval('$edx')), n))
        except (gdb.MemoryError, gdb.error, struct.error):
            pass
        return False

def arm_send_trace(inf, slide):
    if os.environ.get('GW2_RUNNER_SEND_TRACE', '0') != '1':
        return
    SendTrace('*%#x' % (SEND_FN + slide), internal=True)
    print('GW2_RUNNER: send trace armed')

# Opt-in diagnostic (GW2_RUNNER_COMMAND_TRACE=1, off by default): read-only breakpoints on
# agent command submission 0x14102fbd0 (rcx = world, rdx = command; command +0x20 = agent
# +0x18 interface, +0x50 flags, +0x54 type byte) and its two exits, drop 0x14102fc66 and
# accept 0x14102fc83. Prints the first 30 submissions with the gating fields.
CMD_SUBMIT, CMD_DROP, CMD_ACCEPT = 0x14102fbd0, 0x14102fc66, 0x14102fc83

class CommandTrace(gdb.Breakpoint):
    hits = 0
    def __init__(self, spec, what):
        super().__init__(spec, internal=True)
        self.what = what
    def stop(self):
        try:
            if not in_client() or not is_x86_64():
                return False
            CommandTrace.hits += 1
            if CommandTrace.hits > 90:
                return False
            inf = gdb.selected_inferior()
            rd = lambda a, n: int.from_bytes(inf.read_memory(a, n).tobytes(), 'little')
            if self.what == 'submit':
                world, cmd = int(gdb.parse_and_eval('$rcx')), int(gdb.parse_and_eval('$rdx'))
                iface = rd(cmd + 0x20, 8)
                agent_e8 = rd(iface - 0x18 + 0xe8, 1) if iface else -1
                iface30 = rd(iface + 0x30, 1) if iface else -1
                print('GW2_RUNNER: cmd submit type %#x flags %#x agent+e8 %#x iface+30 %#x world+1cc %#x mode %d' % (
                    rd(cmd + 0x54, 1), rd(cmd + 0x50, 4), agent_e8, iface30, rd(world + 0x1cc, 4), rd(rd(world + 0x10, 8) + 8, 4)))
            else:
                print('GW2_RUNNER: cmd %s' % self.what)
        except (gdb.MemoryError, gdb.error):
            pass
        return False

def arm_command_trace(inf, slide):
    if os.environ.get('GW2_RUNNER_COMMAND_TRACE', '0') != '1':
        return
    CommandTrace('*%#x' % (CMD_SUBMIT + slide), 'submit')
    CommandTrace('*%#x' % (CMD_DROP + slide), 'DROPPED')
    CommandTrace('*%#x' % (CMD_ACCEPT + slide), 'accepted')
    print('GW2_RUNNER: command trace armed')

# Opt-in diagnostic (GW2_RUNNER_MOVE_TRACE=1, off by default): read-only ladder through the
# movement-input path (MvCliContext, Game\Movement\Cli). Each point prints its first hit and
# then only when its observed value changes, so per-frame points stay quiet. Every
# address is an instruction boundary in the frozen exe.
MOVE_POINTS = (
    (0x14141d380, 'MvCli Start', None),
    (0x14141d476, 'MvCli Start sets +0x178|=0x20', None),
    (0x14141a4d0, 'MvCli tick (+0x178)', ('mem32', 'rcx', 0x178)),
    (0x14141b473, 'update: controlled char', ('reg', 'rax')),
    (0x14141b6fa, 'update: input blocked (0x14141da80)', ('reg', 'eax')),
    (0x14141dabb, 'blocked: char+0x360 (state==1)', ('reg', 'eax')),
    (0x14141dad2, 'blocked: char+0x370 (state==2)', ('reg', 'eax')),
    (0x14141db1d, 'blocked: view lookup', ('reg', 'rax')),
    (0x14141db30, 'blocked: view+0x738', ('reg', 'eax')),
    (0x14141db25, 'can-move: +0x738 object vtable (rax)', ('reg', 'rax')),
    (0x14141db27, 'can-move: +0x738 object byte +0xe8', ('mem32', 'rdi', 0xe8)),
    (0x14141db40, 'can-move: char +0x298 (must be null)', ('reg', 'rax')),
    (0x14141db45, 'can-move: arg esi (0 = also check speed)', ('reg', 'esi')),
    (0x14141db55, 'can-move: agent +0x630 speed (xmm0, must be nonzero)', ('xmm', 'xmm0')),
    (0x14141db6b, 'can-move: agent +0xd8 (must be null)', ('reg', 'rax')),
    (0x14141b88a, 'update: M+0xc8 path', ('reg', 'eax')),
    (0x141284c20, 'V+0x228 entry (V+0x40)', ('mem8', 'rcx', 0x40)),
    (0x141284c83, 'V+0x228 state 0x78 found', None),
    (0x141284c7e, 'V+0x228 exit (no state 0x78)', None),
    (0x14102d3b0, 'agent vtable +0xa8 command entry (type)', ('reg', 'edx')),
    (0x14141b77f, 'update: move vector (+0x9c..+0xa8)', ('vec', 'rdi', 0x9c)),
    (0x14102dd30, 'motion-state recompute entry', None),
    (0x14109ede0, 'collider +0x2c0: vector +0x170', ('vec', 'rcx', 0x170)),
    (0x14109ee20, 'collider +0x2c8: flags +0x58 (low 32)', ('mem32', 'rcx', 0x58)),
    (0x14109edc0, 'collider +0x2b8 entry (rcx+0xf8 state block)', ('reg', 'rcx')),
    (0x14102dda9, 'motion-state: collision +0x238 value', ('reg', 'eax')),
    (0x14102dd86, 'motion-state: from parent +0x660', ('reg', 'eax')),
    (0x14102dd8d, 'motion-state: parent without +0x660 -> 1', None),
    (0x14102e095, 'motion-state: new state (edi)', ('reg', 'edi')),
    (0x14141b7bf, 'update: 0x1410fbfe0()->+0x98', ('reg', 'eax')),
    (0x14141b7c3, 'update: calls 0x14141be20', None),
    (0x14141b7cf, 'update: calls 0x141420960(ctx, 1)', None),
    (0x141420960, 'apply entry (edx)', ('reg', 'edx')),
    (0x1414209bb, 'apply: [r8+0x48]() result', ('reg', 'rax')),
    (0x1414209d3, 'apply: +0x80() result', ('reg', 'rax')),
    (0x141420a25, 'apply: +0x20() result', ('reg', 'eax')),
    (0x141420a40, 'apply: state object vtable (rcx) and +0x660 target (rdx)', ('reg', 'rdx')),
    (0x141420a45, 'apply: state passed to 0x141025cc0 (1/6/10/13 block)', ('reg', 'eax')),
    (0x141420a4c, 'apply: 0x141025cc0 result', ('reg', 'eax')),
    (0x141420a90, 'apply: +0x1c0() result', ('reg', 'eax')),
    (0x141420aa1, 'apply: calls 0x1409c4450', None),
    (0x141420af2, 'apply: early exit', None),
)

class MoveTrace(gdb.Breakpoint):
    def __init__(self, addr, label, probe, slide):
        super().__init__('*%#x' % (addr + slide), internal=True)
        self.label, self.probe, self.hits, self.last = label, probe, 0, object()
    def stop(self):
        try:
            if not in_client() or not is_x86_64():
                return False
            self.hits += 1
            value = None
            if self.probe:
                kind, reg = self.probe[0], self.probe[1]
                if kind == 'xmm':
                    value = (float(gdb.parse_and_eval('$%s.v4_float[0]' % reg)),)
                    r = None
                else:
                    r = int(gdb.parse_and_eval('$' + reg))
                if kind == 'xmm':
                    pass
                elif kind == 'reg':
                    value = r
                elif kind == 'vec':
                    value = struct.unpack('<4f', gdb.selected_inferior().read_memory(r + self.probe[2], 16).tobytes())
                else:
                    size = 4 if kind == 'mem32' else 1
                    value = int.from_bytes(gdb.selected_inferior().read_memory(r + self.probe[2], size).tobytes(), 'little')
            decade = self.probe is None and self.hits in (10, 100, 1000, 10000, 100000)
            if self.hits == 1 or value != self.last or decade:
                self.last = value
                shown = '-' if value is None else (' '.join('%.3g' % v for v in value) if isinstance(value, tuple) else '%#x' % value)
                extra = ' rcx %#x' % int(gdb.parse_and_eval('$rcx')) if 'vtable' in self.label else ''
                print('GW2_RUNNER: move %s %s hit %d value %s%s' % (
                    time.strftime('%H:%M:%SZ', time.gmtime()), self.label, self.hits, shown, extra))
        except (gdb.MemoryError, gdb.error):
            pass
        return False

def arm_move_trace(inf, slide):
    if os.environ.get('GW2_RUNNER_MOVE_TRACE', '0') != '1':
        return
    only = [w for w in os.environ.get('GW2_RUNNER_MOVE_ONLY', '').split(',') if w]
    points = [p for p in MOVE_POINTS if not only or any(w in p[1] for w in only)]
    for addr, label, probe in points:
        MoveTrace(addr, label, probe, slide)
    print('GW2_RUNNER: move trace armed (%d points)' % len(points))

class Arm(gdb.Breakpoint):
    """First IPv4 connect after the image is mapped: install the two patch points."""
    def stop(self):
        # A Python exception here would make gdb stop (and -batch then ends the run),
        # so anything unexpected means "not this connect", never a stop.
        if not is_x86_64():
            return False
        try:
            inf = gdb.selected_inferior()
            if int.from_bytes(inf.read_memory(int(gdb.parse_and_eval('$rsi')), 2).tobytes(), 'little') != 2:
                return False
            # The client image, not another 64-bit Wine process: its header page is the exe's
            # own and its trust bytes are mapped and still original.
            # No file-backed header page (some Wine builds copy the image into anonymous
            # memory): try the preferred base. The header and trust checks below decide.
            base = image_base(inf.pid)
            if base is None:
                base = PREFERRED_BASE
            header = inf.read_memory(base, 0x400).tobytes()
            # The loader may rewrite ImageBase in a relocated header, so compare the DOS
            # header and the PE signature + COFF header (machine, timestamp) only.
            pe = struct.unpack_from('<I', exe, 0x3c)[0]
            if header[:0x40] != exe[:0x40] or header[pe:pe + 24] != exe[pe:pe + 24]:
                return False
            slide = base - PREFERRED_BASE
            if inf.read_memory(TRUST_VA + slide, TRUST_LEN).tobytes() != ORIGINAL_TRUST:
                return False
        except (gdb.MemoryError, gdb.error):
            return False
        ARMED_INFERIOR[0] = inf.num
        SLIDE[0] = slide
        TrustPatch('*%#x' % (0x14158b740 + slide), internal=True)
        DhPatch('*%#x' % (0x140ff4480 + slide), internal=True)
        print('GW2_RUNNER: patch points armed (image %s)' % ('at its preferred base' if slide == 0 else 'relocated'))
        apply_world_name(inf, slide)
        arm_stage_trace(inf, slide)
        arm_downloader_check(inf, slide)
        arm_crash_report(inf, slide)
        apply_full_textures(inf, slide)
        arm_empty_composite(inf, slide)
        arm_texture_diag(inf, slide)
        arm_send_trace(inf, slide)
        arm_command_trace(inf, slide)
        arm_move_trace(inf, slide)
        self.enabled = False
        return False

Arm('connect', internal=True)
end
run
python
if ARMED_INFERIOR[0] is None:
    print('GW2_RUNNER: the client image was never seen in the traced process; no patch was applied')
end
