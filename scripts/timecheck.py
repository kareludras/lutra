#!/usr/bin/env python3
"""Measure how long Lutra takes to answer `go wtime/btime` against the
per-move budget it should be using, over many self-play positions.

Wins and losses on time only show that a move was late, not by how much.
This drives the engine directly, gives it a random clock on every move,
and reports latency per clock size plus the worst overshoot. A healthy
build stays within a few milliseconds of its budget at every clock size.

usage: python scripts/timecheck.py <engine> [--book FILE] [--games N] [--plies N]
"""

import argparse
import random
import subprocess
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLOCKS_MS = [15, 30, 60, 120, 250, 500, 1000, 3000, 10000]


def budget_ms(clock_ms):
    """Mirrors compute_move_time in lutra-uci: clock / 30, kept 100ms clear
    of the flag, never below 10ms."""
    return max(10, min(clock_ms // 30, max(clock_ms - 100, 0)))


class Engine:
    def __init__(self, path):
        self.proc = subprocess.Popen(
            [path], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1
        )
        self.send("uci")
        self.wait_for("uciok")

    def send(self, line):
        self.proc.stdin.write(line + "\n")
        self.proc.stdin.flush()

    def wait_for(self, prefix):
        while True:
            line = self.proc.stdout.readline()
            if not line:
                raise RuntimeError("engine exited")
            if line.startswith(prefix):
                return line.strip()

    def close(self):
        self.send("quit")
        self.proc.wait(timeout=5)


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("engine")
    p.add_argument("--book", default=str(ROOT / "scripts" / "books" / "lutra-8ply.epd"))
    p.add_argument("--games", type=int, default=15)
    p.add_argument("--plies", type=int, default=60)
    p.add_argument("--seed", type=int, default=1)
    args = p.parse_args()

    rng = random.Random(args.seed)
    fens = [line.strip() for line in open(args.book) if line.strip()]

    engine = Engine(str(Path(args.engine).resolve()))
    samples = []  # (clock_ms, elapsed_ms)
    for _ in range(args.games):
        fen = rng.choice(fens)
        moves = []
        engine.send("ucinewgame")
        for _ in range(args.plies):
            clock = rng.choice(CLOCKS_MS)
            position = f"position fen {fen}" + (" moves " + " ".join(moves) if moves else "")
            engine.send(position)
            engine.send("isready")
            engine.wait_for("readyok")
            start = time.perf_counter()
            engine.send(f"go wtime {clock} btime {clock} winc 0 binc 0")
            reply = engine.wait_for("bestmove")
            samples.append((clock, (time.perf_counter() - start) * 1000))
            move = reply.split()[1]
            if move == "0000":
                break
            moves.append(move)
    engine.close()

    print(f"{len(samples)} moves measured")
    print(f"{'clock':>7} {'budget':>7} {'n':>5} {'median':>8} {'max':>8} {'flagged':>8}")
    for clock in CLOCKS_MS:
        times = sorted(e for c, e in samples if c == clock)
        if not times:
            continue
        flagged = sum(1 for e in times if e >= clock)
        print(
            f"{clock:>7} {budget_ms(clock):>7} {len(times):>5} "
            f"{times[len(times) // 2]:>8.1f} {times[-1]:>8.1f} {flagged:>8}"
        )
    clock, elapsed = max(samples, key=lambda s: s[1] - budget_ms(s[0]))
    print(
        f"worst overshoot: {elapsed - budget_ms(clock):.1f}ms "
        f"(clock {clock}ms, budget {budget_ms(clock)}ms, took {elapsed:.1f}ms)"
    )
    flagged = sum(1 for c, e in samples if e >= c)
    print(f"moves slower than the whole clock: {flagged}")
    return 1 if flagged else 0


if __name__ == "__main__":
    raise SystemExit(main())
