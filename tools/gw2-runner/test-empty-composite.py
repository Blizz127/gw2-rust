#!/usr/bin/env python3
"""Host fixtures for the actual shipped GDB block; no client/archive access."""
import ast
import contextlib
import io
import os
from pathlib import Path
import struct
import types
import unittest
from unittest.mock import patch

RUNNER = Path(__file__).with_name('gw2-runner.gdb')
SOURCE = RUNNER.read_text().split('# BEGIN EMPTY_COMPOSITE_COMPLETION\n', 1)[1].split('# END EMPTY_COMPOSITE_COMPLETION', 1)[0]


class MemoryErrorFixture(Exception):
    pass


class AbortFixture(Exception):
    pass


class BreakpointFixture:
    def __init__(self, address, internal=False):
        self.address = address; self.internal = internal; self.number = 7
        self.enabled = True; self.condition = ''; self.deleted = False

    def is_valid(self): return not self.deleted
    def delete(self): self.deleted = True


class Inferior:
    num = 1

    def __init__(self):
        self.memory = {}; self.writes = []; self.fail_write = False; self.fail_readback = False

    def put(self, address, data): self.memory.update((address + i, byte) for i, byte in enumerate(data))
    def q(self, address, value): self.put(address, struct.pack('<Q', value))
    def d(self, address, value): self.put(address, struct.pack('<I', value))

    def read_memory(self, address, size):
        if self.fail_readback and self.writes and address == self.writes[-1][0]:
            raise MemoryErrorFixture()
        try: return memoryview(bytes(self.memory[address + i] for i in range(size)))
        except KeyError: raise MemoryErrorFixture() from None

    def write_memory(self, address, data):
        if self.fail_write: raise MemoryErrorFixture()
        self.writes.append((address, bytes(data))); self.put(address, data)


def fixture(slide=0):
    inf = Inferior(); regs = {}; commands = []; breakpoints = []
    def breakpoint(address, internal=False):
        bp = BreakpointFixture(address, internal); breakpoints.append(bp); return bp
    gdb = types.SimpleNamespace(parameter=lambda name: False, parse_and_eval=lambda name: regs[name[1:]],
        selected_inferior=lambda: inf, MemoryError=MemoryErrorFixture, error=MemoryErrorFixture,
        Breakpoint=breakpoint, execute=commands.append)
    def abort(reason): raise AbortFixture(reason)
    scope = dict(gdb=gdb, struct=struct, os=os, in_client=lambda: True, is_x86_64=lambda: True, abort=abort)
    exec(compile(SOURCE, str(RUNNER), 'exec'), scope)
    for address, expected in scope['EMPTY_COMPOSITE_GUARDS'].items(): inf.put(address + slide, expected)
    scope['file_bytes'] = lambda address, size: scope['EMPTY_COMPOSITE_GUARDS'][address][:size]
    state = scope['EmptyCompositeCompletion'](inf, slide)
    destination, source, op, group, job, rsp = 0x50000, 0x60000, 0x70004, 0x80000, 0x90000, 0xa0000
    for data, dims, flags in [(destination, (1024, 1024), 0x3104), (source, (512, 128), 0x10006)]:
        inf.q(data, 0x141bfc700 + slide); inf.q(data + 0x40, data + 0x1000)
        inf.d(data + 0x60, flags); inf.d(data + 0x64, 26); inf.d(data + 0x68, 1)
        inf.d(data + 0x48, dims[0]); inf.d(data + 0x4c, dims[1]); inf.q(data + 0xa8, 1)
    inf.d(destination + 0x94, 4)
    inf.q(op, 0x141bfc7c0 + slide); inf.q(op + 0x28, destination); inf.q(op + 0x30, source)
    inf.q(op + 0x100, group); inf.q(op + 0x108, job); inf.d(op + 0x50, 1); inf.q(op + 0x58, 0)
    inf.put(op + 0x38, struct.pack('<2I', 896, 896)); inf.put(op + 0x40, struct.pack('<4I', 512, 128, 640, 256))
    inf.q(rsp + 0x340, op); inf.q(rsp + 0x348, 0x140a750f2 + slide)
    inf.q(rsp + 0xa8, job); inf.q(rsp + 0xb0, group); inf.q(rsp + 0x378, rsp + 0x3b0)
    inf.put(rsp + 0x3b0, struct.pack('<4I', 512, 128, 512, 128)); inf.put(rsp + 0x370, struct.pack('<2I', 896, 896))
    for offset, value, size in [(0, 1, 4), (8, 0, 8), (0x10, 1, 4), (0x20, 0, 8), (0x28, group, 8), (0x30, 1, 4)]:
        (inf.d if size == 4 else inf.q)(group + offset, value)
    inf.d(job + 0x18, 5); inf.d(job + 0x1c, 1); inf.d(job + 0xf8, 0)
    # Callback and old job links intentionally remain unreadable/recycled.
    regs.update(eax=1, rsp=rsp, rdi=destination, r14=source, r13=rsp + 0x3b0,
                rip=scope['EMPTY_COMPOSITE_SITE'] + slide)
    return state, inf, regs, scope, dict(destination=destination, source=source, op=op, group=group, job=job, rsp=rsp,
                                      commands=commands, breakpoints=breakpoints)


def quiet(call):
    output = io.StringIO()
    with contextlib.redirect_stdout(output): call()
    return output.getvalue()


class CompletionTests(unittest.TestCase):
    def test_qualified_current_frame_changes_only_completion_dword(self):
        for slide in (0, 0x24000000):
            with self.subTest(slide=slide):
                state, inf, regs, scope, p = fixture(slide)
                before = dict(inf.memory); quiet(state.complete)
                self.assertEqual(inf.writes, [(p['job'] + 0xf8, b'\x01\0\0\0')])
                changed = {address for address in inf.memory if inf.memory[address] != before[address]}
                self.assertEqual(changed, {p['job'] + 0xf8})
                self.assertEqual(state.d(p['destination'] + 0x60), 0x3104)

    def test_nonempty_wrong_site_and_wrong_inferior_never_write(self):
        for change in ['eax', 'rectangle', 'rip', 'inferior', 'architecture', 'nonstop']:
            state, inf, regs, scope, p = fixture()
            if change == 'eax': regs['eax'] = 0
            elif change == 'rectangle': inf.put(p['rsp'] + 0x3b0, struct.pack('<4I', 0, 0, 320, 128))
            elif change == 'rip': regs['rip'] += 1
            elif change == 'inferior': scope['in_client'] = lambda: False
            elif change == 'architecture': scope['is_x86_64'] = lambda: False
            else: scope['gdb'].parameter = lambda name: True
            quiet(state.complete); self.assertEqual(inf.writes, [], change)

    def test_each_fresh_group_and_job_predicate_is_required(self):
        for offset, value, size in [(0, 2, 4), (8, 0x1230, 8), (0x10, 0, 4), (0x20, 0x1230, 8), (0x28, 0, 8), (0x30, 2, 4)]:
            state, inf, regs, scope, p = fixture()
            (inf.d if size == 4 else inf.q)(p['group'] + offset, value)
            quiet(state.complete); self.assertEqual(inf.writes, [], ('group', offset))
        for offset, value in [(0x18, 1), (0x18, 6), (0x1c, 2), (0xf8, 1)]:
            state, inf, regs, scope, p = fixture(); inf.d(p['job'] + offset, value)
            quiet(state.complete); self.assertEqual(inf.writes, [], ('job', offset))

    def test_call_frame_operation_identity_null_and_alias_exclusions(self):
        for base, offset, value in [('rsp', 0x348, 0), ('rsp', 0x340, 0), ('op', 0, 0),
                                   ('op', 0x28, 0), ('op', 0x30, 0), ('op', 0x100, 0), ('op', 0x108, 0),
                                   ('rsp', 0xa8, 0), ('rsp', 0xb0, 0), ('rsp', 0x378, 0), ('destination', 0, 0)]:
            state, inf, regs, scope, p = fixture(); inf.q(p[base] + offset, value)
            quiet(state.complete); self.assertEqual(inf.writes, [], (base, offset))
        state, inf, regs, scope, p = fixture(); regs['r13'] += 4
        quiet(state.complete); self.assertEqual(inf.writes, [])
        state, inf, regs, scope, p = fixture()
        inf.q(p['rsp'] + 0xb0, p['job']); inf.q(p['op'] + 0x100, p['job'])
        quiet(state.complete); self.assertEqual(inf.writes, [])

    def test_backing_readiness_format_counts_and_code_mismatch_exclusions(self):
        for base, offset, value, size in [('source', 0x40, 0, 8), ('source', 0x60, 6, 4), ('source', 0xa8, 0xabc0, 8),
                                        ('source', 0x64, 31, 4), ('destination', 0x40, 0, 8),
                                        ('destination', 0x60, 0x1203104, 4), ('destination', 0x64, 27, 4),
                                        ('destination', 0x94, 0, 4), ('destination', 0x94, 0xffffffff, 4),
                                        ('op', 0x50, 0, 4), ('op', 0x50, 6, 4)]:
            state, inf, regs, scope, p = fixture()
            (inf.d if size == 4 else inf.q)(p[base] + offset, value)
            quiet(state.complete); self.assertEqual(inf.writes, [], (base, offset))
        state, inf, regs, scope, p = fixture(); inf.put(0x140a747c0, b'\x90')
        quiet(state.complete); self.assertEqual(inf.writes, [])

    def test_actual_three_pass_shape_and_missing_or_undersized_later_mask(self):
        for variant in ['valid', 'missing', 'small', 'no_levels']:
            state, inf, regs, scope, p = fixture(); inf.d(p['op'] + 0x50, 3)
            for index, mask in [(1, 0xb0000), (2, 0xc0000)]:
                inf.q(p['op'] + 0x58 + index * 8, mask)
                inf.q(mask, 0x141bfc700); inf.q(mask + 0x40, mask + 0x1000)
                inf.d(mask + 0x60, 0x10006); inf.d(mask + 0x64, 27); inf.d(mask + 0x68, 1)
                inf.d(mask + 0x48, 512); inf.d(mask + 0x4c, 128); inf.q(mask + 0xa8, 1)
            if variant == 'missing': inf.q(p['op'] + 0x60, 0)
            elif variant == 'small': inf.d(0xc0000 + 0x4c, 127)
            elif variant == 'no_levels': inf.d(0xc0000 + 0x68, 0)
            quiet(state.complete); self.assertEqual(len(inf.writes), int(variant == 'valid'))

    def test_second_snapshot_revalidates_every_input_and_current_frame(self):
        for change in ['caller', 'outstanding', 'backing']:
            state, inf, regs, scope, p = fixture(); capture = state.capture; calls = [0]
            def changed():
                calls[0] += 1
                if calls[0] == 2:
                    if change == 'caller': inf.q(p['rsp'] + 0x348, 0)
                    elif change == 'outstanding': inf.d(p['destination'] + 0x94, 3)
                    else: inf.q(p['source'] + 0x40, 0xd0000)  # Still valid, but changed.
                return capture()
            state.capture = changed
            quiet(state.complete); self.assertEqual(inf.writes, [], change)

    def test_read_failure_before_write_is_unchanged_and_write_failure_is_fatal(self):
        state, inf, regs, scope, p = fixture(); del inf.memory[p['group']]
        quiet(state.complete); self.assertEqual(inf.writes, [])
        for fail in ['fail_write', 'fail_readback']:
            state, inf, regs, scope, p = fixture(); setattr(inf, fail, True)
            with self.assertRaises(AbortFixture): quiet(state.complete)
            self.assertEqual(len(inf.writes), int(fail == 'fail_readback'))

    def test_logging_budget_never_limits_repairs_and_keeps_no_lifetime_cache(self):
        state, inf, regs, scope, p = fixture(); initial_keys = set(vars(state))
        def many():
            for index in range(260):
                # New current allocation each time; no UI or cached history exists.
                job = 0x100000 + 0x200 * index
                inf.q(p['rsp'] + 0xa8, job); inf.q(p['op'] + 0x108, job)
                inf.d(job + 0x18, 5); inf.d(job + 0x1c, 1); inf.d(job + 0xf8, 0)
                state.complete()
        output = quiet(many)
        self.assertEqual(len(inf.writes), 260)
        self.assertEqual(len(output.splitlines()), 9)
        self.assertIn('handler remains armed', output)
        self.assertEqual(state.logs, 9); self.assertEqual(set(vars(state)), initial_keys)
        names = {node.id for node in ast.walk(ast.parse(SOURCE)) if isinstance(node, ast.Name)}
        self.assertNotIn('time', names); self.assertNotIn('owner', names)

    def test_installer_plain_native_condition_optout_and_partial_cleanup(self):
        state, inf, regs, scope, p = fixture()
        with patch.dict(os.environ, {'GW2_RUNNER_EMPTY_COMPOSITE': '0'}):
            quiet(lambda: scope['arm_empty_composite'](inf, 0))
        self.assertEqual(p['breakpoints'], [])
        with patch.dict(os.environ, {'GW2_RUNNER_EMPTY_COMPOSITE': '1'}):
            quiet(lambda: scope['arm_empty_composite'](inf, 0))
            quiet(lambda: scope['arm_empty_composite'](inf, 0))
        self.assertEqual(len(p['breakpoints']), 1)
        bp = p['breakpoints'][0]
        self.assertFalse(bp.internal); self.assertTrue(bp.enabled)
        self.assertEqual(bp.address, '*0x140a74ced')
        self.assertEqual(bp.condition, '$eax == 1 && *(unsigned long long*)($rsp+0x348) == 0x140a750f2')
        self.assertEqual(p['commands'], ['commands 7\nsilent\npython empty_composite_dispatch()\ncontinue\nend'])
        state, inf, regs, scope, p = fixture()
        scope['gdb'].execute = lambda command: (_ for _ in ()).throw(MemoryErrorFixture())
        quiet(lambda: scope['arm_empty_composite'](inf, 0))
        self.assertIsNone(scope['EMPTY_COMPOSITE']); self.assertTrue(p['breakpoints'][0].deleted)

    def test_file_or_memory_guard_prevents_installation(self):
        for which in ['file', 'memory']:
            state, inf, regs, scope, p = fixture()
            if which == 'file': scope['file_bytes'] = lambda address, size: bytes(size)
            else: inf.put(0x140a74ced, b'\x90')
            quiet(lambda: scope['arm_empty_composite'](inf, 0))
            self.assertIsNone(scope['EMPTY_COMPOSITE']); self.assertEqual(p['breakpoints'], [])

    def test_only_one_target_write_and_existing_dimension_fix_retained(self):
        tree = ast.parse(SOURCE)
        writes = [n for n in ast.walk(tree) if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and n.func.attr == 'write_memory']
        self.assertEqual(len(writes), 1)
        self.assertFalse(any(n.bases for n in ast.walk(tree) if isinstance(n, ast.ClassDef) and n.name == 'EmptyCompositeCompletion'))
        runner = RUNNER.read_text()
        self.assertIn("struct.unpack('<II', inf.read_memory(task + 0x68, 8)", runner)
        self.assertIn('arm_empty_composite(inf, slide)', runner.split('class Arm(', 1)[1])
        self.assertNotIn('set $', SOURCE)


if __name__ == '__main__': unittest.main(verbosity=2)
