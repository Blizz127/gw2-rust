#!/usr/bin/env python3
"""Check a host-selected character store without reading character contents."""
import os
from pathlib import Path
import stat
import sys


def private_directory(path):
    info = path.lstat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
        raise ValueError("saved character directories must be owned, private directories")


def validate(store, config, binaries, data):
    store = Path(store)
    if not store.is_absolute():
        raise ValueError("character store must be absolute")
    state = Path(config).resolve(strict=True).parent
    private_directory(state)
    parent = store.parent.resolve(strict=True)
    private_directory(parent)
    resolved = (parent / store.name).resolve(strict=False)
    if resolved == state or not resolved.is_relative_to(state):
        raise ValueError("character store must be beneath the host configuration directory")
    kit_roots = [Path(binaries).resolve(strict=True).parent,
                 Path(data).resolve(strict=True).parent,
                 Path(__file__).resolve().parent.parent]
    if any(resolved == root or resolved.is_relative_to(root) for root in kit_roots):
        raise ValueError("character store must be outside every pinned kit")
    if store.exists() or store.is_symlink():
        private_directory(store)


if __name__ == "__main__":
    try:
        if len(sys.argv) != 5:
            raise ValueError("expected STORE CONFIG BIN DATA")
        validate(*sys.argv[1:])
    except (OSError, ValueError) as error:
        # Do not emit paths or record contents from an OS exception.
        detail = str(error) if isinstance(error, ValueError) else "saved-store path check failed"
        print(detail, file=sys.stderr)
        sys.exit(2)
