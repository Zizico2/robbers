#!/usr/bin/env python3
"""Exercise the real CLI: discover, visit every hex with every pawn, reject bad moves.

Build first: cargo build --no-default-features
Run: python3 scripts/playtest.py [path/to/robbers]
Uses only Python's standard library. Also import Driver in other agent scripts.
"""
from collections import deque
from pathlib import Path
import json
import os
import select
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
DELTAS = {"e": (1, 0), "ne": (1, -1), "nw": (0, -1), "w": (-1, 0), "sw": (-1, 1), "se": (0, 1)}


class Driver:
    def __init__(self, binary=ROOT / "target/debug/robbers"):
        binary = Path(binary).resolve()
        environment = os.environ.copy()
        # `cargo run` supplies these paths automatically. Support direct launches
        # of development builds too, without requiring global linker settings.
        if (binary.parent / "libbevy_dylib.so").exists():
            rust_lib = subprocess.check_output(["rustc", "--print", "target-libdir"],
                                               cwd=ROOT, text=True).strip()
            paths = [str(binary.parent), str(binary.parent / "deps"), rust_lib]
            if environment.get("LD_LIBRARY_PATH"):
                paths.append(environment["LD_LIBRARY_PATH"])
            environment["LD_LIBRARY_PATH"] = os.pathsep.join(paths)
        self.process = subprocess.Popen([str(binary), "--headless"], stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE, text=True, env=environment)
        self.sequence = 0

    def request(self, method, **params):
        self.sequence += 1
        request = {"id": self.sequence, "method": method, **params}
        self.process.stdin.write(json.dumps(request) + "\n")
        self.process.stdin.flush()
        if not select.select([self.process.stdout], [], [], 10)[0]:
            self.process.kill()
            raise TimeoutError(f"CLI did not answer request {self.sequence}")
        line = self.process.stdout.readline()
        if not line:
            raise RuntimeError("CLI exited before replying. Build first with cargo build --no-default-features.")
        response = json.loads(line)
        assert response["id"] == self.sequence, response
        return response

    def command(self, kind, **params):
        response = self.request("command", command={"type": kind, **params})
        assert "result" in response, response
        return response["result"]

    def next_turn(self):
        self.command("end_turn")
        return self.command("begin_turn")

    def turn_to(self, player):
        state = self.request("state")["result"]
        if state["turn"]["phase"] == "handoff":
            state = self.command("begin_turn")
        while state["selected_player"] != player:
            state = self.next_turn()
        return state

    def act(self, kind, **params):
        """Spend an action, cycling ordinary turns back to this actor if needed."""
        state = self.request("state")["result"]
        if state["turn"]["actions_remaining"] == 0:
            player = state["selected_player"]
            self.next_turn()
            self.turn_to(player)
        return self.command(kind, **params)

    def close(self):
        if self.process.poll() is None:
            self.request("quit")
        self.process.stdin.close()
        assert self.process.wait(timeout=5) == 0


def path(start, target, tiles):
    queue = deque([(start, [])])
    seen = {start}
    while queue:
        position, steps = queue.popleft()
        if position == target:
            return steps
        for direction, (dq, dr) in DELTAS.items():
            neighbor = (position[0] + dq, position[1] + dr)
            if neighbor in tiles and neighbor not in seen:
                seen.add(neighbor)
                queue.append((neighbor, steps + [direction]))
    raise AssertionError("disconnected board")


def main():
    driver = Driver(Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT / "target/debug/robbers")
    try:
        assert driver.request("schema")["result"]["schema_version"] == 2
        initial = driver.request("state")["result"]
        tiles = {(t["hex"]["q"], t["hex"]["r"]) for t in initial["tiles"]}
        assert len(tiles) == 37
        count = 0
        for player in range(4):
            state = driver.turn_to(player)
            for target in sorted(tiles):
                pos = state["players"][player]["hex"]
                for direction in path((pos["q"], pos["r"]), target, tiles):
                    before = state
                    state = driver.act("move", direction=direction)
                    assert state["revision"] > before["revision"]
                    assert state["moves"] == before["moves"] + 1
                    for other in range(4):
                        if other != player:
                            assert state["players"][other] == before["players"][other]
                    count += 1
                assert state["players"][player]["hex"] == {"q": target[0], "r": target[1]}
        for command in [{"type": "step", "q": 200, "r": -200}, {"type": "select", "player": 9},
                        {"type": "move", "direction": "north"}]:
            before = driver.request("state")["result"]
            assert "error" in driver.request("command", command=command)
            assert driver.request("state")["result"] == before
        state = driver.command("reset")
        assert state["players"] == initial["players"]
        assert state["moves"] == 0 and state["revision"] > initial["revision"]
        print(f"PASS: 4 investigators visited all 37 hexes; {count} moves; invalid commands atomic; reset verified.")
    finally:
        driver.close()


if __name__ == "__main__":
    main()
