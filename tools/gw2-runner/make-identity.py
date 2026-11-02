#!/usr/bin/env python3
"""Create a fresh GW2 server identity for one deployment.

    make-identity.py FROZEN_GW2_64_EXE SERVER_DIR RUNNER_DIR

SERVER_DIR (mode 0700, stays on the server, never shipped) receives
cligate-101.key.pem, cligate-101.cert.der (via tools/gen-test-cert.sh),
dh-params.bin and dh-private.bin, the inputs to gw2-login-observe
--tls-key/--tls-cert and gw2-game-observe --params/--private.

RUNNER_DIR receives the PUBLIC values the runner patches in: own-modulus.bin
(the certificate's 128-byte RSA modulus, big-endian, as the client stores its
trust modulus) and dh-params.bin (the client's original 136-byte DH block with
our public value at +72). No private material is ever written there.
"""
import hashlib, os, secrets, struct, subprocess, sys
from pathlib import Path

EXE_SHA256 = '99342078ef57ca5653046392c2cf8aa197f1badbbe2c68297ca97e0d925d8621'
DH_VA, DH_LEN = 0x142111220, 136
ROOT = Path(__file__).resolve().parents[2]


def file_bytes(exe, va, size):
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


def write(path, data):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'wb') as out:
        out.write(data)


def main():
    exe_path, server, runner = map(Path, sys.argv[1:4])
    exe = exe_path.read_bytes()
    if hashlib.sha256(exe).hexdigest() != EXE_SHA256:
        sys.exit('not the frozen build 209026 executable')
    for d in (server, runner):
        if d.exists():
            sys.exit(f'{d} already exists; refusing to overwrite an identity')
    os.umask(0o077)
    server.mkdir(mode=0o700, parents=True)
    runner.mkdir(mode=0o700, parents=True)
    subprocess.run(['bash', str(ROOT / 'tools/gen-test-cert.sh'), str(server)],
                   check=True, stdout=subprocess.DEVNULL)
    modulus = subprocess.run(['openssl', 'x509', '-inform', 'DER', '-in', str(server / 'cligate-101.cert.der'),
                              '-noout', '-modulus'], check=True, capture_output=True, text=True).stdout
    modulus = bytes.fromhex(modulus.strip().split('=', 1)[1])
    if len(modulus) != 128:
        sys.exit('unexpected modulus size')
    original = file_bytes(exe, DH_VA, DH_LEN)
    p = int.from_bytes(original[8:72], 'little')
    g = int.from_bytes(original[4:8], 'little')
    if int.from_bytes(original[0:4], 'little') != 1 or g != 4:
        sys.exit('unexpected DH parameter block')
    private = secrets.randbelow(p - 3) + 2
    params = original[:72] + pow(g, private, p).to_bytes(64, 'little')
    write(server / 'dh-params.bin', params)
    write(server / 'dh-private.bin', private.to_bytes(64, 'little'))
    write(runner / 'own-modulus.bin', modulus)
    write(runner / 'dh-params.bin', params)
    print('identity created: server', server, 'runner', runner)


if __name__ == '__main__':
    main()
