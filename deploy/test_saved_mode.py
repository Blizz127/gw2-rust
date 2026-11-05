"""Focused supervisor argv tests; fake children never bind or read identities."""
import json
from pathlib import Path
import shlex
import subprocess
import tempfile
import time
import unittest

SUPERVISOR = Path(__file__).with_name("gw2-server.sh")
FAKE = '''#!/usr/bin/python3
import json, os, pathlib, sys, time
root = pathlib.Path(__file__).resolve().parents[2]
fd = os.open(root / "calls", os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
os.write(fd, (json.dumps([pathlib.Path(sys.argv[0]).name, sys.argv[1:]]) + "\\n").encode())
os.close(fd)
if "--check-config" in sys.argv:
    marker = root / "reject-preflight"
    port = sys.argv[sys.argv.index("--listen") + 1].rsplit(":", 1)[1]
    sys.exit(int(marker.exists() and marker.read_text() == port))
while True:
    time.sleep(1)
'''


class SavedModeTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="gw2-supervisor-")
        self.root = Path(self.tmp.name)
        self.kit = self.root / "kit"
        self.state = self.root / "state"
        for directory in [self.kit / "bin", self.kit / "data", self.state / "id", self.state / "run"]:
            directory.mkdir(parents=True, mode=0o700)
        self.state.chmod(0o700)
        for name in ["gw2-game-observe", "gw2-login-observe", "gw2-assetcdn"]:
            path = self.kit / "bin" / name
            path.write_text(FAKE)
            path.chmod(0o700)
        (self.kit / "data/latest.txt").write_text("209026\n")
        self.config = self.state / "gw2.conf"
        self.values = dict(GW2_BIN=self.kit / "bin", GW2_DATA=self.kit / "data",
                           GW2_ID=self.state / "id", GW2_RUN=self.state / "run",
                           GW2_BIND="127.0.0.1", GW2_SPAWN="0,0,7.4")

    def tearDown(self):
        self.tmp.cleanup()

    def saved(self):
        self.values.update(GW2_CHARACTER_MODE="saved-norn", GW2_CHARACTER_STORE=self.state / "characters",
                           GW2_MAP_KEY="379", GW2_MESMER_VALUES="1,0", GW2_REWARD_TRACKS="56,88,47,112",
                           GW2_EQUIPMENT="equipment.bin", GW2_SKILLS="skills.bin", GW2_HEALTH="388",
                           GW2_QUEST="154,0,1", GW2_HOSTILE_NPC="101,4,5,6", GW2_DAMAGE_ON_SKILL="10",
                           GW2_HEART="56,5")

    def calls(self):
        path = self.root / "calls"
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def launch(self, expected):
        self.config.write_text("".join(f"{key}={shlex.quote(str(value))}\n" for key, value in self.values.items()))
        self.config.chmod(0o600)
        process = subprocess.Popen(["bash", str(SUPERVISOR), str(self.config)],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            deadline = time.monotonic() + 5
            while process.poll() is None and time.monotonic() < deadline:
                if len(self.calls()) >= expected:
                    break
                time.sleep(0.01)
            if process.poll() is None:
                process.terminate()
            _, errors = process.communicate(timeout=5)
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate()
        return process.returncode, self.calls(), errors.decode()

    def test_default_prepared_argv_is_unchanged(self):
        code, calls, errors = self.launch(4)
        self.assertEqual(code, 143, errors)
        self.assertEqual(len(calls), 4)
        game = {argv[argv.index("--listen") + 1]: argv for name, argv in calls if name == "gw2-game-observe"}
        identity = ["--params", str(self.state / "id/dh-params.bin"), "--private", str(self.state / "id/dh-private.bin"),
                    "--log", str(self.state / "run/unused")]
        self.assertEqual(game["127.0.0.1:6113"], ["--no-capture", "--listen", "127.0.0.1:6113"] + identity + [
            "--reply-frame", str(self.kit / "data/auth-ready.bin"), "--auth-reply-body", str(self.kit / "data/auth-account-body.bin"),
            "--auth-agreement-state", str(self.state / "run/agreement-version"),
            "--auth-character-record", str(self.kit / "data/character-record.bin"), "--auth-play-route", "127.1.0.4:6112"])
        self.assertEqual(game["127.0.0.1:6112"], ["--no-capture", "--listen", "127.0.0.1:6112", "--mission-preamble"] + identity + [
            "--mission-map-key", "15", "--mission-population", str(self.kit / "data/character-record.bin"),
            "--mission-spawn", "0,0,7.4", "--mission-population-delay-ms", "0"])

    def test_saved_mode_preflights_both_roles_then_uses_same_four_routes(self):
        self.saved()
        code, calls, errors = self.launch(6)
        self.assertEqual(code, 143, errors)
        self.assertEqual(len(calls), 6)
        self.assertTrue(all(args[0] == "--check-config" for _, args in calls[:2]))
        active = [args for name, args in calls[2:] if name == "gw2-game-observe"]
        self.assertEqual(len(active), 2)
        for (_, checked), args in zip(calls[:2], sorted(active, key=lambda a: a[a.index("--listen") + 1], reverse=True)):
            self.assertEqual(checked[1:], args)
            self.assertIn("--no-capture", args)
            self.assertEqual(args[args.index("--character-store") + 1], str(self.state / "characters"))
            self.assertNotIn("--mission-population", args)
            self.assertNotIn("--auth-character-record", args)
        auth = next(a for a in active if "--auth-create-route" in a)
        game = next(a for a in active if "--creation-scene" in a)
        self.assertEqual(auth[auth.index("--auth-create-route") + 1], "127.1.0.4:6112")
        self.assertEqual(game[game.index("--creation-scene") + 1], "127.1.0.4:6112")
        self.assertIn("--mission-hostile-npc", game)
        self.assertEqual(game[game.index("--mission-heart") + 1], "56,5")
        self.assertNotIn("--mission-heart", auth)

    def test_failed_binary_preflight_starts_no_listeners_or_fallback(self):
        self.saved()
        for port, count in [("6113", 1), ("6112", 2)]:
            with self.subTest(port=port):
                (self.root / "calls").unlink(missing_ok=True)
                (self.root / "reject-preflight").write_text(port)
                code, calls, _ = self.launch(6)
                self.assertNotEqual(code, 0)
                self.assertEqual(len(calls), count)
                self.assertTrue(all(args[0] == "--check-config" for _, args in calls))

    def test_invalid_mode_profile_and_store_start_no_children(self):
        self.saved()
        original = self.values.copy()
        private_store = self.state / "private"
        private_store.mkdir(mode=0o700)
        public_store = self.state / "public"
        public_store.mkdir(mode=0o755)
        public_store.chmod(0o755)
        linked_store = self.state / "linked"
        linked_store.symlink_to(private_store, target_is_directory=True)
        cases = [("GW2_CHARACTER_MODE", "unknown"), ("GW2_CHARACTER_MODE", "prepared"),
                 ("GW2_MAP_KEY", "15"), ("GW2_EQUIPMENT", ""), ("GW2_CHARACTER_STORE", "relative"),
                 ("GW2_CHARACTER_STORE", self.kit / "data/store"),
                 ("GW2_CHARACTER_STORE", self.state), ("GW2_CHARACTER_STORE", linked_store),
                 ("GW2_CHARACTER_STORE", public_store)]
        for key, value in cases:
            with self.subTest(key=key, value=str(value)):
                self.values = original | {key: value}
                code, calls, _ = self.launch(4)
                self.assertNotEqual(code, 0)
                self.assertEqual(calls, [])

    def test_kit_profile_cannot_enable_saved_mode(self):
        (self.kit / "data/mission-profile.conf").write_text("GW2_CHARACTER_MODE=saved-norn\n")
        code, calls, _ = self.launch(4)
        self.assertNotEqual(code, 0)
        self.assertEqual(calls, [])

    def test_private_host_config_inside_kit_cannot_make_store_durable(self):
        self.saved()
        self.kit.chmod(0o700)
        self.config = self.kit / "gw2.conf"
        self.values["GW2_CHARACTER_STORE"] = self.kit / "characters"
        code, calls, errors = self.launch(4)
        self.assertNotEqual(code, 0)
        self.assertEqual(calls, [])
        self.assertIn("outside every pinned kit", errors)


if __name__ == "__main__":
    unittest.main()
