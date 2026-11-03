#!/usr/bin/env python3
"""stackd readiness_only probes for the GW2 server.

    gw2-probe.py health RUN_DIR   -> {"ready":bool,"running":bool,"detail":str}
    gw2-probe.py humans RUN_DIR   -> {"humans":int,"bots":0}  (GW2 has no bots)

Read-only. Readiness requires every process in RUN_DIR/pids.json to be alive,
to run the recorded executable, and to own (by /proc/<pid>/fd socket inode) a
TCP listener on its recorded bind address and port. humans counts distinct
remote peers with an ESTABLISHED connection to the login (6114), auth (6113) or
game (6112) listener, owned by that process: a player at the login screen keeps
cligate open (/Sts/Ping every 30 s), so the game port alone would read 0 while
someone is logging in. A server whose processes are all gone reports 0 (it is
stopped); a partly running one, or anything unreadable, exits non-zero instead
of reporting 0.
"""
import ipaddress, json, os, socket, sys

LISTEN, ESTABLISHED = '0A', '01'


def tcp_sockets():
    rows = []
    for name in ('/proc/net/tcp', '/proc/net/tcp6'):
        try:
            lines = open(name).read().splitlines()[1:]
        except FileNotFoundError:
            continue
        for line in lines:
            f = line.split()
            addr, port = f[1].split(':')
            raw = bytes.fromhex(addr)
            if len(raw) == 4:
                ip = socket.inet_ntop(socket.AF_INET, raw[::-1])
            else:
                ip = socket.inet_ntop(socket.AF_INET6, b''.join(raw[i:i + 4][::-1] for i in range(0, 16, 4)))
            rows.append((ip, int(port, 16), f[3], f[9], f[2]))
    return rows


def inodes(pid):
    out = set()
    for fd in os.listdir(f'/proc/{pid}/fd'):
        try:
            target = os.readlink(f'/proc/{pid}/fd/{fd}')
        except OSError:
            continue
        if target.startswith('socket:['):
            out.add(target[8:-1])
    return out


def load(run):
    return json.load(open(os.path.join(run, 'pids.json')))


def health(run):
    try:
        state = load(run)
    except (OSError, ValueError):
        return False, False, 'no pids.json; server not started'
    sockets = tcp_sockets()
    running = False
    problems = []
    for p in state['processes']:
        pid, role = p['pid'], p['role']
        try:
            exe = os.readlink(f'/proc/{pid}/exe')
        except OSError:
            problems.append(f'{role} not running')
            continue
        running = True
        if os.path.realpath(exe) != os.path.realpath(p['exe']):
            problems.append(f'{role} pid {pid} is not {p["exe"]}')
            continue
        owned = inodes(pid)
        bind = str(ipaddress.ip_address(state['bind']))
        if not any(ip == bind and port == p['port'] and st == LISTEN and ino in owned
                   for ip, port, st, ino, _ in sockets):
            problems.append(f'{role} not listening on {bind}:{p["port"]}')
    if problems:
        return False, running, '; '.join(problems)
    return True, True, 'all 4 listeners bound by their own processes'


def alive(p):
    # The recorded pid still runs the recorded executable (not a reused pid).
    try:
        return os.path.realpath(os.readlink(f'/proc/{p["pid"]}/exe')) == os.path.realpath(p['exe'])
    except OSError:
        return False


def humans(run):
    state = load(run)
    procs = state['processes']
    if not any(alive(p) for p in procs):
        return 0  # stopped (asleep): nobody can be connected
    bind = str(ipaddress.ip_address(state['bind']))
    sockets = tcp_sockets()
    peers = set()
    for p in procs:
        if p['role'] not in ('login', 'auth', 'game'):
            continue
        owned = inodes(p['pid'])  # raises if this process died: count unknown
        for ip, port, st, ino, remote in sockets:
            if ip == bind and port == p['port'] and st == ESTABLISHED and ino in owned:
                peers.add(remote.split(':')[0])
    return len(peers)


def main():
    mode, run = sys.argv[1], sys.argv[2]
    if mode == 'health':
        ready, running, detail = health(run)
        print(json.dumps({'ready': ready, 'running': running, 'detail': detail}))
        sys.exit(0 if ready else 1)
    if mode == 'humans':
        try:
            count = humans(run)
        except Exception as e:  # unknown count: never report 0
            print(json.dumps({'error': type(e).__name__}))
            sys.exit(1)
        print(json.dumps({'humans': count, 'bots': 0}))
        sys.exit(0)
    sys.exit('usage: gw2-probe.py health|humans RUN_DIR')


if __name__ == '__main__':
    main()
