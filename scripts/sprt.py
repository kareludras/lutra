#!/usr/bin/env python3
"""Play two Lutra builds against each other with fastchess.

By default this runs an SPRT: games continue until the test can say, with
the given error rates, whether `dev` is at least `elo1` stronger than
`base` (H1) or no better than `elo0` (H0). With --games it instead plays a
fixed number of games and reports the Elo estimate.

Builds are made from git refs (or the current working tree) and cached by
commit under target/sprt/builds, so re-running against the same base is
cheap. Everything fastchess writes (PGN, log, resume config) goes to a
per-run directory under target/sprt/runs.

Examples:
    python scripts/sprt.py                              # working tree vs main
    python scripts/sprt.py --base HEAD~1 --dev HEAD
    python scripts/sprt.py --games 200 --tc 4+0.04      # fixed-length Elo match

The GitHub workflow .github/workflows/elo.yml runs this on pull requests
with --summary (Markdown results for the job page) and
--fail-on-regression (non-zero exit only if the whole 95% confidence interval is
below a 50% score).

fastchess: https://github.com/Disservin/fastchess. Pass --fastchess or set
FASTCHESS to the executable if it's not on PATH.
"""

import argparse
import io
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPRT_DIR = ROOT / "target" / "sprt"
EXE = ".exe" if platform.system() == "Windows" else ""
DEFAULT_BOOK = ROOT / "scripts" / "books" / "lutra-8ply.epd"


def run(cmd, **kwargs):
    return subprocess.run(cmd, check=True, **kwargs)


def git(*args):
    out = run(["git", *args], cwd=ROOT, capture_output=True, text=True)
    return out.stdout.strip()


def cargo_build(manifest_dir, target_dir):
    env = dict(os.environ, CARGO_TARGET_DIR=str(target_dir))
    run(
        ["cargo", "build", "--release", "--bin", "lutra", "--quiet"],
        cwd=manifest_dir,
        env=env,
    )
    return Path(target_dir) / "release" / f"lutra{EXE}"


def build(ref, label):
    """Returns the path to a built engine for `ref` ("worktree" = the
    current checkout, uncommitted changes included)."""
    if ref == "worktree":
        print(f"[{label}] building working tree")
        built = cargo_build(ROOT, ROOT / "target")
        dest = SPRT_DIR / "builds" / f"worktree-{label}{EXE}"
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(built, dest)
        return dest

    sha = git("rev-parse", "--verify", f"{ref}^{{commit}}")
    dest = SPRT_DIR / "builds" / f"{sha[:12]}{EXE}"
    if dest.exists():
        print(f"[{label}] {ref} = {sha[:12]} (cached build)")
        return dest

    print(f"[{label}] building {ref} = {sha[:12]}")
    src = SPRT_DIR / "src" / sha[:12]
    if src.exists():
        shutil.rmtree(src)
    src.mkdir(parents=True)
    archive = run(["git", "archive", sha], cwd=ROOT, capture_output=True).stdout
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        tar.extractall(src)
    built = cargo_build(src, SPRT_DIR / "cargo-target")
    shutil.copy2(built, dest)
    shutil.rmtree(src, ignore_errors=True)
    return dest


def bench_signature(engine):
    """The engine's bench node count, or None for builds that predate the
    bench command."""
    try:
        out = subprocess.run(
            [str(engine), "bench"], capture_output=True, text=True, timeout=120
        ).stdout
    except (OSError, subprocess.TimeoutExpired):
        return None
    m = re.search(r"(\d+) nodes", out)
    return m.group(1) if m else None


def parse_results(output):
    """Final Elo estimate, game counts and SPRT verdict from fastchess's
    output (it prints a running report; the last one wins)."""
    results = {}
    for line in output.splitlines():
        if m := re.match(r"Elo: (-?[\d.]+|-?inf|nan) \+/- ([\d.]+|inf|nan)", line):
            results["elo"], results["error"] = float(m.group(1)), float(m.group(2))
        elif m := re.match(r"Games: (\d+), Wins: (\d+), Losses: (\d+), Draws: (\d+)", line):
            results["games"], results["wins"], results["losses"], results["draws"] = map(
                int, m.groups()
            )
        elif line.startswith("LLR:"):
            results["llr"] = line.strip()
        elif "SPRT" in line and "accepted" in line:
            results["verdict"] = "H1 accepted" if "H1" in line else "H0 accepted"
    return results


def write_summary(path, args, results, benches):
    elo = results.get("elo")
    lines = [
        f"### Elo: `{args.dev}` vs `{args.base}`",
        "",
        "| | |",
        "|---|---|",
    ]
    if elo is not None:
        lines.append(f"| Elo | **{elo:+.1f} ± {results['error']:.1f}** (95%) |")
    if "games" in results:
        lines.append(
            f"| Games | {results['games']} (W {results['wins']} / "
            f"L {results['losses']} / D {results['draws']}) |"
        )
    if "verdict" in results:
        lines.append(f"| SPRT [{args.elo0}, {args.elo1}] | {results['verdict']} |")
    elif "llr" in results:
        lines.append(f"| SPRT [{args.elo0}, {args.elo1}] | inconclusive ({results['llr']}) |")
    lines.append(f"| Time control | {args.tc}, {args.concurrency} concurrent games |")
    for label, bench in benches:
        lines.append(f"| Bench ({label}) | {bench or 'n/a'} |")
    with open(path, "a", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n\n")


def is_clear_regression(results):
    """True if even the optimistic end of a 95% confidence interval on the
    dev score is below 50%. Computed from W/L/D rather than fastchess's Elo
    error bars, which come out as `nan` for one-sided results (exactly the
    clearest regressions)."""
    n = results.get("games", 0)
    if n == 0:
        return False
    wins, losses, draws = results["wins"], results["losses"], results["draws"]
    mean = (wins + 0.5 * draws) / n
    variance = (wins * (1 - mean) ** 2 + draws * (0.5 - mean) ** 2 + losses * mean**2) / n
    return mean + 1.96 * (variance / n) ** 0.5 < 0.5


def find_fastchess(arg):
    candidate = arg or os.environ.get("FASTCHESS") or shutil.which("fastchess")
    if not candidate or not Path(candidate).exists():
        sys.exit(
            "fastchess not found: pass --fastchess PATH or set FASTCHESS. "
            "Releases: https://github.com/Disservin/fastchess/releases"
        )
    return Path(candidate).resolve()


def main():
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("--dev", default="worktree", help="git ref, or 'worktree' (default)")
    p.add_argument("--base", default="main", help="git ref (default: main)")
    p.add_argument("--fastchess", help="path to the fastchess executable")
    p.add_argument("--tc", default="8+0.08", help="time control (default: 8+0.08)")
    p.add_argument("--concurrency", type=int, default=max(1, (os.cpu_count() or 2) // 2))
    p.add_argument("--hash", type=int, default=16, help="Hash (MB) for both engines")
    p.add_argument("--book", default=str(DEFAULT_BOOK), help="EPD opening book")
    p.add_argument("--elo0", type=float, default=0.0)
    p.add_argument("--elo1", type=float, default=5.0)
    p.add_argument("--alpha", type=float, default=0.05)
    p.add_argument("--beta", type=float, default=0.05)
    p.add_argument(
        "--max-games", type=int, default=20000, help="SPRT safety cap (default: 20000)"
    )
    p.add_argument(
        "--games", type=int, help="play exactly this many games instead of an SPRT"
    )
    p.add_argument("--summary", help="append a Markdown results table to this file")
    p.add_argument(
        "--fail-on-regression",
        action="store_true",
        help="exit 1 if the whole 95%% confidence interval is below a 50%% score",
    )
    args = p.parse_args()

    fastchess = find_fastchess(args.fastchess)
    dev = build(args.dev, "dev")
    base = build(args.base, "base")
    benches = [(label, bench_signature(engine)) for label, engine in (("dev", dev), ("base", base))]
    for label, bench in benches:
        print(f"[{label}] bench: {bench or 'n/a'}")

    run_dir = SPRT_DIR / "runs" / datetime.now().strftime("%Y%m%d-%H%M%S")
    run_dir.mkdir(parents=True)

    games = args.games if args.games else args.max_games
    cmd = [
        str(fastchess),
        "-engine", f"cmd={dev}", "name=dev",
        "-engine", f"cmd={base}", "name=base",
        "-each", f"tc={args.tc}", f"option.Hash={args.hash}",
        "-openings", f"file={Path(args.book).resolve()}", "format=epd", "order=random",
        "-games", "2", "-repeat",
        "-rounds", str((games + 1) // 2),
        "-concurrency", str(args.concurrency),
        "-recover",
        "-pgnout", f"file={run_dir / 'games.pgn'}",
        "-log", f"file={run_dir / 'fastchess.log'}", "level=warn",
    ]
    if not args.games:
        cmd += [
            "-sprt",
            f"elo0={args.elo0}", f"elo1={args.elo1}",
            f"alpha={args.alpha}", f"beta={args.beta}",
        ]

    print(f"results -> {run_dir}")
    # Run inside the run directory: fastchess writes its resume config
    # (config.json) to the working directory.
    with open(run_dir / "output.txt", "w", encoding="utf-8") as log:
        proc = subprocess.Popen(
            cmd, cwd=run_dir, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
        )
        seen_warnings = set()
        output = []
        for line in proc.stdout:
            log.write(line)
            output.append(line)
            if line.startswith(("Started game", "Finished game")):
                continue
            # e.g. an old base build without the Hash option warns per game.
            if line.startswith("Warning"):
                if line in seen_warnings:
                    continue
                seen_warnings.add(line)
            sys.stdout.write(line)
        code = proc.wait()

    results = parse_results("".join(output))
    if args.summary:
        write_summary(args.summary, args, results, benches)
    if code != 0:
        return code
    if args.fail_on_regression and is_clear_regression(results):
        print("regression: the 95% confidence interval is entirely below 50%")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
