#!/usr/bin/env python3
"""Focused observer smoke: pipelined STS/TLS upgrade, then split/coalesced STS.

Build gw2-login-observe first. Uses a fresh local test identity and loopback only.
"""
import pathlib
import re
import select
import socket
import ssl
import subprocess
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]


def main():
    tmp = ROOT / "target" / "tmp"
    tmp.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="starttls-smoke-", dir=tmp) as name:
        work = pathlib.Path(name)
        subprocess.run(["bash", str(ROOT / "tools/gen-test-cert.sh"), name],
                       check=True, stdout=subprocess.DEVNULL)
        cert = work / "cligate-101.cert.der"
        trusted = work / "trusted.pem"
        subprocess.run(["openssl", "x509", "-inform", "DER", "-in", str(cert),
                        "-out", str(trusted)], check=True)
        observer = subprocess.Popen([
            str(ROOT / "target/debug/gw2-login-observe"), "--listen", "127.0.0.1:0",
            "--log", str(work / "capture"), "--tls-key", str(work / "cligate-101.key.pem"),
            "--tls-cert", str(cert),
            "--answer-gethostname", "cligate.101.ncplatform.net.",
        ], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            assert select.select([observer.stdout], [], [], 10)[0], "observer startup timeout"
            line = observer.stdout.readline()
            match = re.search(r"observing 127\.0\.0\.1:(\d+)", line)
            assert match, "observer did not report its bound loopback address"
            with socket.create_connection(("127.0.0.1", int(match[1])), timeout=5) as sock:
                ctx = ssl.create_default_context()
                ctx.minimum_version = ctx.maximum_version = ssl.TLSVersion.TLSv1_2
                ctx.set_ciphers("AES128-SHA:@SECLEVEL=0")
                ctx.options |= getattr(ssl, "OP_LEGACY_SERVER_CONNECT", 0x4)
                ctx.load_verify_locations(cafile=str(trusted))
                incoming, outgoing = ssl.MemoryBIO(), ssl.MemoryBIO()
                peer = ctx.wrap_bio(incoming, outgoing, server_hostname="cligate.101.ncplatform.net.")
                try:
                    peer.do_handshake()
                except ssl.SSLWantReadError:
                    pass
                hello = outgoing.read()
                assert hello, "no ClientHello generated"
                starttls = b"P /Auth/StartTls STS/1.0\r\ns:42;timeout=29000\r\nl:10\r\n\r\n<Request/>"
                sock.sendall(starttls + hello)
                reply = b""
                while b"\r\n\r\n" not in reply:
                    reply += sock.recv(65536)
                head, rest = reply.split(b"\r\n\r\n", 1)
                assert head.startswith(b"STS/1.0 200 OK\r\n") and b"s:42R" in head
                length = int(re.search(rb"(?:^|\r\n)l:(\d+)", head)[1])
                while len(rest) < length:
                    rest += sock.recv(65536)
                assert rest[:length] == b"<Reply/>"
                incoming.write(rest[length:])
                while True:
                    try:
                        peer.do_handshake()
                        sock.sendall(outgoing.read())
                        break
                    except ssl.SSLWantReadError:
                        sock.sendall(outgoing.read())
                        packet = sock.recv(65536)
                        assert packet, "server closed during handshake"
                        incoming.write(packet)
                assert peer.version() == "TLSv1.2" and peer.cipher()[0] == "AES128-SHA"
                first = b"P /Smoke/Split STS/1.0\r\ns:43\r\nl:8\r\n\r\n<Probe/>"
                second = b"P /Smoke/Next STS/1.0\r\ns:44\r\n\r\n"
                for part in (first[:17], first[17:] + second):
                    peer.write(part)
                    sock.sendall(outgoing.read())
                peer.write(b"P /Auth/GetHostname STS/1.0\r\ns:45\r\nl:0\r\n\r\n")
                sock.sendall(outgoing.read())
                response = b""
                expected_body = b"<Reply><Hostname>cligate.101.ncplatform.net.</Hostname><Local/></Reply>\n"
                while not response.endswith(expected_body):
                    try:
                        response += peer.read(65536)
                    except ssl.SSLWantReadError:
                        packet = sock.recv(65536)
                        assert packet, "server closed before GetHostname reply"
                        incoming.write(packet)
                assert response.startswith(b"STS/1.0 200 OK\r\n") and b"s:45R\r\n" in response
                try:
                    peer.unwrap()
                except ssl.SSLWantReadError:
                    sock.sendall(outgoing.read())
                    incoming.write(sock.recv(65536))
                    peer.unwrap()
            deadline = time.monotonic() + 5
            while True:
                logs = list((work / "capture").glob("*.log"))
                text = "".join(p.read_text() for p in logs)
                if "# TLS capture closed" in text:
                    break
                assert time.monotonic() < deadline, "capture did not finish"
                time.sleep(0.05)
            assert "# TLS established: TLS1.2 AES128-SHA" in text
            assert text.count("decrypted STS") == 3
            assert "/Smoke/Split" in text and "/Smoke/Next" in text and "<Probe/>" in text
            assert all(p.stat().st_mode & 0o777 == 0o600 for p in logs)
            print("StartTls smoke: trusted TLS1.2, pipelined ClientHello, split/coalesced STS, GetHostname reply, private logs: PASS")
        finally:
            observer.terminate()
            try:
                observer.wait(timeout=5)
            except subprocess.TimeoutExpired:
                observer.kill()
                observer.wait()


if __name__ == "__main__":
    main()
