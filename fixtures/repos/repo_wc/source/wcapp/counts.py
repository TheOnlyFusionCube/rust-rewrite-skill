"""Counting helpers for lines, words, and bytes."""

from __future__ import annotations


def count_lines(text: str) -> int:
    if not text:
        return 0
    # Match classic wc: number of newline characters; if file does not end
    # with newline, still count the last partial line.
    if text.endswith("\n"):
        return text.count("\n")
    return text.count("\n") + (1 if text else 0)


def count_words(text: str) -> int:
    return len(text.split())


def count_bytes(data: bytes) -> int:
    return len(data)


def count_text(text: str) -> tuple[int, int, int]:
    """Return (lines, words, bytes) for a UTF-8 text string."""
    data = text.encode("utf-8")
    return count_lines(text), count_words(text), count_bytes(data)


def count_file(path: str) -> tuple[int, int, int]:
    with open(path, "rb") as f:
        data = f.read()
    text = data.decode("utf-8", errors="replace")
    return count_lines(text), count_words(text), count_bytes(data)
