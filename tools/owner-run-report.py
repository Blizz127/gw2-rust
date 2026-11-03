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
    ('world-name patch', r'GW2_RUNNER: world-name [^\n]*'),
    ('gdb script error', r'Python Exception'),
    ('gdb stopped (run ends)', r'^Breakpoint -?\d+, '),
    ('wine stack overflow', r'stack overflow'),
    ('bubble/bwrap problem', r'gw2 bubble:|bwrap:|gw2 bridge:'),
    ('runner refused', r'^gw2-runner: '),
    ('provisioning', r'gw2 provision: '),
]


# Map-loader stages (VdfLoad.cpp, build 209026) as the b114 runner's read-only stage trace
# prints them: "GW2_RUNNER: stage <UTC> MapLoader: STATE_<NAME> (<n>)". The marker fires on
# leaving a stage and on entering the next, so the last name is the stage the load is in.
# Each stop point maps to what that stage waits on and the next step (docs/re/209026-map-ready.md,
# docs/re/209026-request-reply.md).
STAGE_TRIAGE = {
    'LOAD_CONTENT': 'no wait in this stage; a stop here is an assert on the map definition or content (1063 map id vs the route map key). Check ArenaNet.log for an assertion.',
    'LOAD_MANIFEST': 'waits with NO timeout for the map-portal loader (MpLoader.cpp) async Gw2.dat read: 0x1417e9740 starts it, 0x1417e9cd0 polls +0xd8 and the per-item +0x18. Not a server message. Next: find the file id it reads (loader +0x30 vtable +0x20/+0x08), check it in the frozen Gw2.dat, check the DNS lines for CATCH-ALL names.',
    'SERVER_WAIT': 'waits for 1060 (flag +0x2c4); after 300 s it goes to error state 4 and sits. 1060 sets the flag unconditionally, so check that the server log shows "Mission phase sent server 1063 MapData and 1060 LoadMap" for this connection.',
    'MAP_DOWNLOAD': 'waits with NO timeout for the map resource object (loader +0x218, done slot +0x48, error slot +0x50), kicking 0x140d8f620 every 10 s. An encrypted map without a key errors out instead (Queensdale is unencrypted). Next: the map file (Queensdale 192711) in Gw2.dat and the DNS lines for an unrouted download host.',
    'MAP_STREAM': 'waits with NO timeout for every map streamer (loader +0x230, count +0x23c; status = vtable +0x30): 1-3 in progress (pumped by +0x20 each tick), 4 done (result via +0x40, then destroyed), 5 failed. All done -> client 420; any failed -> 418, which the server handles. Purely local (map file data), no server input: a stop here is a streamer stuck in 1-3. Next: a read-only runner probe of each streamer status at 0x1409527d9.',
    'MODELS_STREAM': 'after 420; model streaming. The server already sent the population on 420; check the population frames were sent.',
    'MAP_ASSET_STREAM': 'asset streaming before AGENT_STREAM.',
    'AGENT_STREAM': 'waits with NO timeout for a finalized controlled character (population 428 ... 170, 618, 444), then up to about 120 s for world readiness. A stop here is a population problem.',
    'READY_WAIT': 'countdown, then READY.',
    'READY': 'loader done: the client should send 412 and the server answers 1036 + 431. If 412 never came, check the view and connection conditions in map-ready.md.',
}


def stage_report(text):
    stages = re.findall(r'GW2_RUNNER: stage (\S+) (.+?) \((-?\d+)\)\s*$', text, re.M)
    if 'GW2_RUNNER: stage trace armed' in text:
        print('  stage trace: armed')
    elif 'stage trace' in text:
        print('  stage trace: ' + re.search(r'GW2_RUNNER: stage trace[^\n]*', text).group(0))
    if not stages:
        return
    print('  map-loader stages (in order):')
    for ts, name, n in stages[-30:]:
        print(f'    {ts} {name} ({n})')
    loader = [(ts, name) for ts, name, n in stages if name.startswith('MapLoader: STATE_')]
    if loader:
        ts, name = loader[-1]
        key = name.split('STATE_', 1)[1]
        print(f'  LAST LOADER STAGE: {key} (since {ts})')
        print(f'  triage: {STAGE_TRIAGE.get(key, "not in the triage table; read VdfLoad.cpp dispatcher 0x140951770")}')


def dns_report(text):
    lines = re.findall(r'gw2 dns: [^\n]*', text)
    if not lines:
        return
    odd = [l for l in lines if 'CATCH-ALL' in l or 'unlisted' in l]
    names = sorted({re.sub(r'^gw2 dns: \S+ \S+ (\S+).*$', r'\1', l) for l in lines})
    print(f'  dns: {len(lines)} logged queries, {len(names)} names; unrouted: {len(odd)}')
    for l in odd[:20]:
        print('    UNROUTED', l[:150])


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
            modules = {'ctable-0x141923b70': 'GcApi.cpp', 'ctable-0x141923f00': 'GcAuthCmd.cpp', 'ctable-0x141927cc0': 'PlWatch.cpp', 'ctable-0x141b7b9b0': 'VendCliContext.cpp', 'ctable-0x141b812f0': 'VoteCliVote.cpp', 'ctable-0x141b8e2e0': 'WvwCommon.cpp', 'ctable-0x14211bdb0': 'AgMsg.cpp', 'ctable-0x1421590f0': 'GaeCliContext.cpp', 'ctable-0x14215dce0': 'BigbCliContext.cpp', 'ctable-0x14216e260': 'CharForce.cpp', 'ctable-0x1421800a0': 'CtCliMsg.cpp', 'ctable-0x142183680': 'ClmCliClaimableGuild.cpp', 'ctable-0x142239a50': 'ConFunContext.cpp', 'ctable-0x14223a5b0': 'ConvCliContext.cpp', 'ctable-0x14223bf20': 'DiCliContext.cpp', 'ctable-0x1422518c0': 'GldCommon.cpp', 'ctable-0x1422557f0': 'HsCliHomestead.cpp', 'ctable-0x14225b6a0': 'InsConst.cpp', 'ctable-0x1422d6a08': 'LbCliContext.cpp', 'ctable-0x1422d7368': 'LfgCliContext.cpp', 'ctable-0x1422d9290': 'LootCliContext.cpp', 'ctable-0x1422d9df0': 'MailConst.cpp', 'ctable-0x1422db6f0': 'MlCache.cpp', 'ctable-0x1422ddaa0': 'MsCliContext.cpp', 'ctable-0x1422e7490': 'PexCliContext.cpp', 'ctable-0x1422e96c0': 'PvpCliContext.cpp', 'ctable-0x1422eab98': 'QaCliContext.cpp', 'ctable-0x1422ee6a0': 'SquadCliContext.cpp', 'ctable-0x1422ef300': 'TournamentConst.cpp'}
            for table, body in sources:
                for mm in re.finditer(r'<Message Id="(\d+)"[^>]*?(?:/>|>(.*?)</Message>)', body, re.S):
                    layouts.setdefault(mm.group(1), f"{(table + ' ' + modules.get(table, '')).strip()}: {' '.join(re.findall(r'<(\w+) Name', mm.group(2) or '')) or '(no fields)'}")
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
        stage_report(text)
        dns_report(text)
    # The client's own crash/disconnect log from the launcher-managed GW2 prefix (the
    # bubble's profile, never the owner's real AppData).
    for path in sorted(glob.glob(os.path.join(LOG_DIR, '*ArenaNet*.log'))):
        text = open(path, errors='replace').read()
        print(f'client log: {os.path.basename(path)}')
        for pattern in (r'MsgConnDispatch failed on message (\d+)', r'Forcing disconnect on \{[^}]*\}, reason: (\d+)',
                        r'^Assertion: .*', r'^File: .*', r'^When: .*', r'DISCONNECT_[A-Z_]+',
                        # Loading screen (docs/re/209026-map-ready.md): the world-readiness
                        # timeout (about 120 s, then it continues) and the loader's stage lines.
                        r'Map load hang on STATE_[A-Z_]+ detected[^\n]*', r'MapLoader: [^\n]*'):
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
