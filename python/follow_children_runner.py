#!/usr/bin/env python3

import argparse
import json
import shutil
import sys
import threading
import time
from pathlib import Path

try:
    import frida
except ModuleNotFoundError as exc:
    print(
        "frida python package is required for --follow-children (pip install frida)",
        file=sys.stderr,
        flush=True,
    )
    raise SystemExit(2) from exc


class ArgusPythonRunner:
    def __init__(self, script_source: str) -> None:
        self._script_source = script_source
        self._device = frida.get_local_device()
        self._lock = threading.RLock()
        self._tracked_pids: set[int] = set()
        self._sessions: dict[int, frida.core.Session] = {}
        self._scripts: dict[int, frida.core.Script] = {}

    def run_exec(self, argv: list[str]) -> None:
        self._device.on("child-added", self._on_child_added)
        self._device.on("child-removed", self._on_child_removed)

        root_pid = self._device.spawn(argv)
        self._attach_and_load(root_pid, enable_child_gating=True)
        self._attach_pending_children()
        self._device.resume(root_pid)
        self._wait_until_tracked_processes_exit()
        self._teardown_all()

    def run_pid(self, pid: int) -> None:
        self._device.on("child-added", self._on_child_added)
        self._device.on("child-removed", self._on_child_removed)

        self._attach_and_load(pid, enable_child_gating=True)
        self._attach_pending_children()
        self._wait_until_tracked_processes_exit()
        self._teardown_all()

    def _attach_pending_children(self) -> None:
        for child in self._device.enumerate_pending_children():
            self._on_child_added(child)

    def _attach_and_load(self, pid: int, *, enable_child_gating: bool) -> None:
        with self._lock:
            if pid in self._sessions:
                return

        session = self._device.attach(pid)
        if enable_child_gating:
            session.enable_child_gating()

        script = session.create_script(self._script_source)
        script.on("message", self._on_message)
        script.load()

        with self._lock:
            self._tracked_pids.add(pid)
            self._sessions[pid] = session
            self._scripts[pid] = script

    def _on_message(self, message: dict, data) -> None:
        if isinstance(message, dict) and message.get("type") == "send":
            self._emit_stdout(message)
            return
        self._emit_stderr(f"script message: {message}")

    def _on_child_added(self, child) -> None:
        pid = int(child.pid)
        try:
            self._attach_and_load(pid, enable_child_gating=True)
            self._safe_resume(pid)
        except Exception as exc:  # pragma: no cover - defensive logging path
            self._emit_stderr(
                f"failed to attach child pid={pid}: {exc}; child={self._describe_child(child)}"
            )

    def _on_child_removed(self, child) -> None:
        self._emit_stderr(f"child removed: {self._describe_child(child)}")

    def _wait_until_tracked_processes_exit(self) -> None:
        while True:
            live_pids = {int(process.pid) for process in self._device.enumerate_processes()}
            with self._lock:
                remaining = self._tracked_pids & live_pids
            if not remaining:
                return
            time.sleep(0.2)

    def _teardown_all(self) -> None:
        with self._lock:
            items = list(self._sessions.items())
            scripts = dict(self._scripts)
            self._sessions.clear()
            self._scripts.clear()

        for pid, script in scripts.items():
            try:
                script.unload()
            except Exception:
                pass

        for pid, session in items:
            try:
                session.disable_child_gating()
            except Exception:
                pass
            try:
                session.detach()
            except Exception:
                pass

    def _safe_resume(self, pid: int) -> None:
        try:
            self._device.resume(pid)
        except Exception as exc:  # pragma: no cover - defensive logging path
            self._emit_stderr(f"resume failed for pid={pid}: {exc}")

    @staticmethod
    def _describe_child(child) -> dict:
        return {
            "pid": getattr(child, "pid", None),
            "parent_pid": getattr(child, "parent_pid", None),
            "origin": getattr(child, "origin", None),
            "identifier": getattr(child, "identifier", None),
            "path": getattr(child, "path", None),
            "argv": getattr(child, "argv", None),
        }

    @staticmethod
    def _emit_stdout(message: dict) -> None:
        sys.stdout.write(json.dumps(message, ensure_ascii=False) + "\n")
        sys.stdout.flush()

    @staticmethod
    def _emit_stderr(message: str) -> None:
        print(message, file=sys.stderr, flush=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--script-source-file", required=True)

    target = parser.add_mutually_exclusive_group(required=True)
    target.add_argument("--pid", type=int)
    target.add_argument("--program")

    parser.add_argument("--arg", action="append", default=[])
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    script_source = Path(args.script_source_file).read_text(encoding="utf-8")
    runner = ArgusPythonRunner(script_source)

    if args.pid is not None:
        runner.run_pid(args.pid)
    else:
        program = resolve_program(args.program)
        argv = [program, *args.arg]
        runner.run_exec(argv)

    return 0


def resolve_program(program: str) -> str:
    if Path(program).exists():
        return str(Path(program))
    resolved = shutil.which(program)
    return resolved or program


if __name__ == "__main__":
    raise SystemExit(main())
