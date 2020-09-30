from pathlib import Path


def root_dir() -> Path:
    root = Path(__file__).parents[2]
    return root
