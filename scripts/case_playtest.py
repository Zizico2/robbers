#!/usr/bin/env python3
"""Real-process turn-based case test. Only this authoring harness reads the answer file."""
import json
from pathlib import Path
import sys
from playtest import Driver, path, ROOT


def main():
    driver = Driver(Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT / "target/debug/robbers")
    try:
        state = driver.request("state")["result"]
        assert not state["notebook"]["clues"]
        tiles = {(t["hex"]["q"], t["hex"]["r"]) for t in state["tiles"]}
        for site in state["case"]["clue_sites"]:
            current = state["players"][0]["hex"]
            for direction in path((current["q"], current["r"]), (site["q"], site["r"]), tiles):
                state = driver.act("move", direction=direction)
            state = driver.act("investigate")
        assert len(state["notebook"]["clues"]) == 7
        assert "error" in driver.request("command", command={"type":"select","player":1})
        assert not driver.turn_to(1)["notebook"]["clues"]
        driver.turn_to(0)
        driver.command("plan_start")
        for q,r in [(-2,0),(-1,0),(0,0),(0,1),(0,2),(1,1),(1,2)]:
            driver.command("plan_step", q=q, r=r)
        assert driver.act("submit_route")["notebook"]["last_submission"] == "incorrect"
        # The production protocol never serves this author-only solution.
        authored = json.loads((ROOT / "crates/robbers-case/cases/morning_escape.json").read_text())
        driver.command("plan_start")
        for hex in authored["solution"][1:]:
            driver.command("plan_step", **hex)
        state = driver.act("submit_route")
        assert state["notebook"]["solved"] and state["notebook"]["attempts"] == 2
        assert state["turn"]["phase"] == "finished" and state["turn"]["winner"] == 0
        assert "solution" not in state["case"]
        state = driver.command("reset")
        assert not state["notebook"]["clues"] and not state["notebook"]["plan"]
        print("PASS: collected seven clues; notebooks isolated; wrong guess retried; exact route solved; reset verified.")
    finally:
        driver.close()


if __name__ == "__main__":
    main()
