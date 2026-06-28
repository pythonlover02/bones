import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Final


ARCH: Final[str] = "x86_64"
BRANCH_PREFIX: Final[str] = f"runtime/org.freedesktop.Platform.VulkanLayer.bones/{ARCH}/"
SUBJECT_PREFIX: Final[str] = "bones extension "
OSTREE_BIN: Final[str] = "ostree"


@dataclass(frozen=True)
class CommitArgs:
    rt: str
    meta_file: Path
    repo: Path
    stage: Path


@dataclass(frozen=True)
class CommitPlan:
    branch: str
    subject: str
    metadata: str
    repo: Path
    stage: Path


def parse_argv(argv: list[str]) -> CommitArgs:
    _, rt, meta_file, repo, stage = argv
    return CommitArgs(
        rt=rt,
        meta_file=Path(meta_file),
        repo=Path(repo),
        stage=Path(stage),
    )


def build_plan(args: CommitArgs, metadata: str) -> CommitPlan:
    return CommitPlan(
        branch=BRANCH_PREFIX + args.rt,
        subject=SUBJECT_PREFIX + args.rt,
        metadata=metadata,
        repo=args.repo,
        stage=args.stage,
    )


def build_command(plan: CommitPlan) -> list[str]:
    return [
        OSTREE_BIN,
        "commit",
        f"--repo={plan.repo}",
        f"--branch={plan.branch}",
        f"--subject={plan.subject}",
        f"--add-metadata-string=xa.metadata={plan.metadata}",
        str(plan.stage),
    ]


def call_read_text(path: Path) -> str:
    return path.read_text()


def call_run(cmd: list[str]) -> int:
    return subprocess.run(cmd).returncode


def main() -> int:
    args = parse_argv(sys.argv)
    return call_run(build_command(build_plan(args, call_read_text(args.meta_file))))


sys.exit(main())
