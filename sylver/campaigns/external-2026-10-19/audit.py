#!/usr/bin/env python3
"""Replay the small claims in RESULT.md with this repository's Python solver.

The million-state ledger rows are not replayed here. Run from the repository
root:

    python sylver/campaigns/external-2026-10-19/audit.py
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT))

from sylver.analyze_opening_16 import exceptional_odd_wins  # noqa: E402
from sylver.short_certificates import (  # noqa: E402
    OPENING_16_EVEN_RESPONSES,
    is_generated,
    minimal_generators,
)
from sylver.solver import frobenius_number, solve_position  # noqa: E402


def check_proof(path: Path) -> None:
    proof = json.loads(path.read_text())
    if proof.get("schema") != 1:
        raise AssertionError(f"{path.name} is not schema 1")
    root = tuple(int(part) for part in proof["root"].split(","))
    if minimal_generators(root) != root:
        raise AssertionError(f"{path.name} root is not minimal")
    nodes = proof["nodes"]
    if proof["root"] not in nodes:
        raise AssertionError(f"{path.name} is missing its root")
    for key, node in nodes.items():
        generators = tuple(int(part) for part in key.split(","))
        if minimal_generators(generators) != generators:
            raise AssertionError(f"{key} is not a minimal key")
        rule = node["rule"]
        if rule == "finite":
            if node["outcome"] != "P":
                raise AssertionError(f"{key} finite node is not P")
            solution = solve_position(generators)
            if solution.is_winning or solution.frobenius != frobenius_number(generators):
                raise AssertionError(f"{key} did not replay as P")
            continue
        if rule != "edge" or node["outcome"] != "N":
            raise AssertionError(f"{key} has an unexpected rule")
        move = node["move"]
        child = tuple(int(part) for part in node["child"].split(","))
        if is_generated(generators, move):
            raise AssertionError(f"{move} is illegal in {key}")
        if minimal_generators((*generators, move)) != child:
            raise AssertionError(f"{key}+{move} is not {child}")
        if node["child"] not in nodes:
            raise AssertionError(f"{key} points at a missing child")
    print(f"proof {path.name}: root {proof['root']} replayed")


def main() -> int:
    table = {move: (reply, destination) for move, reply, destination in OPENING_16_EVEN_RESPONSES}
    expected = {14: (8, "E"), 20: (34, "T"), 22: (12, "P0"), 24: (10, "K")}
    for move, row in expected.items():
        if table.get(move) != row:
            raise AssertionError(f"opening table changed at {move}: {table.get(move)}")
    if any(reply == 46 for _, reply, _ in OPENING_16_EVEN_RESPONSES):
        raise AssertionError("46 is already an opening-table answer; drop that note")

    wins = exceptional_odd_wins(40)
    if wins:
        raise AssertionError(f"analyze_opening_16 found odd wins after 40: {wins}")
    print("analyze_opening_16 40: no exceptional odd winning move")

    for name in ("3-16.json", "blok14-25.json", "blok14-251.json"):
        check_proof(HERE / "proofs" / name)

    leaf = solve_position((14, 22, 24, 25, 32, 34, 40))
    if leaf.is_winning or leaf.frobenius != 55:
        raise AssertionError("Blok leaf {...,25} is not P with Frobenius number 55")
    three = solve_position((3, 16))
    if three.winning_move != 2 or three.frobenius != 29:
        raise AssertionError("{3,16} is not N by 2")
    print("finite leaves: {3,16} N by 2; {14,22,24,25,32,34,40} P, Frobenius 55")
    print("AUDIT_OK")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except AssertionError as error:
        print(f"AUDIT_FAIL {error}", file=sys.stderr)
        sys.exit(1)
