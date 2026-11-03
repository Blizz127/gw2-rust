#!/usr/bin/env python3
"""Line up an owner device run: Banshee's stamped server.log (deploy/tail-stamped.sh)
with the launcher's gw2-runner-<UTC>.log and any journal lines, and name the first
milestone that did not happen.

    tools/owner-run-report.py [LOG_DIR]      (default /home/blizz/.cache/gw2-owner-logs)

Read-only. Prints milestones in protocol order with the first time each was seen,
then the first missing one, then the runner's own markers. Never prints typed values
(the server runs --no-capture, so its log has none either).
"""
import glob, os, re, sys

LOG_DIR = sys.argv[1] if len(sys.argv) > 1 else '/home/blizz/.cache/gw2-owner-logs'

# Server-side milestones, in the order a successful run reaches them.
SERVER = [
    ('cligate connect (login server)', r'^conn \d+: \S+ -> \S+ \(no capture\)'),
    ('login TLS + GetHostname', r'TLS STS: /Auth/GetHostname'),
    ('login StartSsoLogin', r'TLS STS: /Auth/StartSsoLogin'),
    ('login RequestGameToken', r'TLS STS: /Auth/RequestGameToken'),
    ('auth key exchange (6113)', r'Game key reply sent'),
    ('auth login 10 -> fixture reply', r'Explicit correlated auth fixture sent'),
    ('auth play 22 (name select)', r'Play phase parsed client message 22'),
    ('auth play 29 (route request)', r'Play phase parsed client message 29'),
    ('game server mission opening (6112)', r'Mission phase sent server 1063'),
    ('client 420 MapLoaded', r'client 420 MapLoaded'),
    ('population sent', r'sending candidate population sequence'),
    ('client 412 AgentsLoaded', r'client 412 AgentsLoaded'),
    ('1036 BeginPlaying + 431 sent', r'sending server 1036 BeginPlaying'),
    ('in-world agent traffic', r'Mission phase: client message (?:[3-9]|1[0-9]|2[0-6]) '),
]
FAILURES = [r'TLS handshake failed', r'disabled', r'FailedToLoad', r'error', r'panic']
RUNNER = [
    ('runner armed', r'GW2_RUNNER: patch points armed'),
    ('trust patch', r'GW2_RUNNER: trust patch applied'),
    ('DH patch', r'GW2_RUNNER: DH patch applied'),
    ('client never traced', r'client image was never seen'),
    ('guard failure', r'GW2_RUNNER: .*guard failed|nothing started'),
    ('gdb script error', r'Python Exception'),
    ('gdb stopped (run ends)', r'^Breakpoint -?\d+, '),
    ('wine stack overflow', r'stack overflow'),
    ('bubble/bwrap problem', r'gw2 bubble:|bwrap:|gw2 bridge:'),
    ('runner refused', r'^gw2-runner: '),
    ('provisioning', r'gw2 provision: '),
]


def stamped(path):
    for line in open(path, errors='replace'):
        if line.startswith('#'):
            continue
        ts, _, rest = line.rstrip('\n').partition(' ')
        yield ts, rest


def main():
    servers = sorted(glob.glob(os.path.join(LOG_DIR, 'banshee-server-*.log')))
    runners = sorted(glob.glob(os.path.join(LOG_DIR, 'gw2-runner-*.log')))
    print('log dir:', LOG_DIR)
    if servers:
        lines = list(stamped(servers[-1]))
        print(f'server log: {os.path.basename(servers[-1])} ({len(lines)} new lines)')
        furthest = -1
        for i, (name, pattern) in enumerate(SERVER):
            hit = next((ts for ts, text in lines if re.search(pattern, text)), None)
            print(f'  {"OK " if hit else "-- "} {name:40} {hit or ""}')
            if hit:
                furthest = i
        if furthest < 0:
            print('  nothing reached Banshee: the stall is on the device (runner, bubble, relay)')
        elif furthest + 1 < len(SERVER):
            print(f'  furthest: {SERVER[furthest][0]}; next expected: {SERVER[furthest + 1][0]}')
            if furthest == 5:
                print('  (stopping here is normal until the owner clicks I ACCEPT then PLAY)')
        else:
            print('  reached in-world agent traffic')
        # In-world client messages seen, annotated with the client's own descriptor layout
        # (data/protocol/209026/agent-client.xml, from its AgMsg.cpp client table).
        seen = []
        for ts, text in lines:
            m = re.search(r'Mission phase: client message (\d+) \((\d+) bytes\)', text)
            if m and m.group(1) not in [x[0] for x in seen]:
                seen.append((m.group(1), m.group(2), ts))
        if seen:
            # Layouts from agent-client.xml first, then the game's main client table, then
            # any other client table (data/protocol/209026/client-tables.xml).
            layouts = {}
            proto = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'data', 'protocol', '209026')
            sources = []
            if os.path.exists(os.path.join(proto, 'agent-client.xml')):
                sources.append(('agent', open(os.path.join(proto, 'agent-client.xml')).read()))
            if os.path.exists(os.path.join(proto, 'client-tables.xml')):
                tables = re.findall(r'<Messages Name="(ctable-0x[0-9a-f]+)">(.*?)</Messages>', open(os.path.join(proto, 'client-tables.xml')).read(), re.S)
                tables.sort(key=lambda t: t[0] != 'ctable-0x14216e260')
                sources += [(n, b) for n, b in tables if n != 'ctable-0x141923f00']
            for table, body in sources:
                for mm in re.finditer(r'<Message Id="(\d+)"[^>]*?(?:/>|>(.*?)</Message>)', body, re.S):
                    layouts.setdefault(mm.group(1), f"{table}: {' '.join(re.findall(r'<(\w+) Name', mm.group(2) or '')) or '(no fields)'}")
            print('  in-world client messages (first seen):')
            for mid, size, ts in seen:
                print(f'    {ts} client {mid} ({size} bytes): {layouts.get(mid, "not in any client table")}')
        # Sizes-only byte timeline from the game server (net +Nms: up/down/EOF lines).
        net = [(ts, text) for ts, text in lines if text.startswith('net +')]
        if net:
            print('  game-server byte timeline (last 30):')
            for ts, text in net[-30:]:
                print(f'    {ts} {text}')
        bad = [(ts, text) for ts, text in lines if any(re.search(p, text, re.I) for p in FAILURES)]
        for ts, text in bad[:15]:
            print(f'  ! {ts} {text[:140]}')
    else:
        print('server log: none (start deploy/tail-stamped.sh before the run)')
    for path in runners:
        text = open(path, errors='replace').read()
        start = re.search(r'gw2-runner-(\d{8}T\d{6}Z)', path)
        print(f'runner log: {os.path.basename(path)} (started {start.group(1) if start else "?"}, {len(text.splitlines())} lines)')
        for name, pattern in RUNNER:
            hits = [l for l in text.splitlines() if re.search(pattern, l)]
            if hits:
                print(f'  {name:22} x{len(hits)}: {hits[0][:120]}')
    # The client's own crash/disconnect log from the launcher-managed GW2 prefix (the
    # bubble's profile, never the owner's real AppData).
    for path in sorted(glob.glob(os.path.join(LOG_DIR, '*ArenaNet*.log'))):
        text = open(path, errors='replace').read()
        print(f'client log: {os.path.basename(path)}')
        for pattern in (r'MsgConnDispatch failed on message (\d+)', r'Forcing disconnect on \{[^}]*\}, reason: (\d+)',
                        r'^Assertion: .*', r'^File: .*', r'^When: .*', r'DISCONNECT_[A-Z_]+'):
            for m in re.finditer(pattern, text, re.M):
                print('  ', m.group(0)[:160])
        failed = re.findall(r'MsgConnDispatch failed on message (\d+)', text)
        if failed:
            print('  FIRST FAILING SERVER MESSAGE ID:', failed[0])
    for path in sorted(glob.glob(os.path.join(LOG_DIR, '*journal*'))):
        print(f'journal: {os.path.basename(path)}')
        for line in open(path, errors='replace'):
            if re.search(r'gw2|bwrap|wine|BansheeRealm', line, re.I):
                print('  ', line.rstrip()[:160])


if __name__ == '__main__':
    main()
