#!/usr/bin/env python3
"""Readability pass that runs after rustfmt: adds one blank line after a multi-line brace block
statement (if / for / while / loop / match / bare block) so humans get vertical breathing room that
stable rustfmt will not insert on its own.

Only blank lines are ever added, never tokens, so it cannot change what the code does. Braces inside
strings, char literals and comments are ignored, and blank lines are only added between statements,
never between items (rustfmt already spaces those) or between match arms.

  tools/space_blocks.py <files...>            rewrite in place
  tools/space_blocks.py --check <files...>    exit 1 when any file would change
"""

import re
import sys

# The next line after a block is left untouched when it opens with one of these, so continuations,
# closers, match arms and items keep hugging the block.
SKIP_PREFIX = ("}", ")", "]", ".", ",", ";", "=>", "?", "#[", "#!", "else", "where")
ITEM_KW = {"fn", "pub", "impl", "struct", "enum", "trait", "const", "static", "type", "mod", "use", "async", "unsafe", "extern", "macro_rules!", "union"}


def code_mask(src: str) -> list[bool]:
    """True where a character is real code, False inside strings, chars and comments."""
    n = len(src)
    mask = [True] * n
    i = 0
    state = "code"
    block_depth = 0
    raw_hashes = 0
    while i < n:
        c = src[i]
        if state == "code":
            if c == "/" and i + 1 < n and src[i + 1] == "/":
                mask[i] = mask[i + 1] = False
                i += 2
                state = "line"
            elif c == "/" and i + 1 < n and src[i + 1] == "*":
                mask[i] = mask[i + 1] = False
                i += 2
                state = "block"
                block_depth = 1
            elif c == '"':
                mask[i] = False
                i += 1
                state = "str"
            elif c in "rb" and (m := re.match(r'b?r(#*)"', src[i:])):
                raw_hashes = len(m.group(1))
                for p in range(i, i + m.end()):
                    mask[p] = False
                i += m.end()
                state = "raw"
            elif c == "b" and i + 1 < n and src[i + 1] == '"':
                mask[i] = mask[i + 1] = False
                i += 2
                state = "str"
            elif c == "'" and (m := re.match(r"b?'(\\.|[^\\'])'", src[i:])):
                for p in range(i, i + m.end()):
                    mask[p] = False
                i += m.end()
            else:
                i += 1
        elif state == "line":
            if c == "\n":
                state = "code"
            else:
                mask[i] = False
            i += 1
        elif state == "block":
            if c == "/" and i + 1 < n and src[i + 1] == "*":
                mask[i] = mask[i + 1] = False
                block_depth += 1
                i += 2
            elif c == "*" and i + 1 < n and src[i + 1] == "/":
                mask[i] = mask[i + 1] = False
                block_depth -= 1
                i += 2
                if block_depth == 0:
                    state = "code"
            else:
                if c != "\n":
                    mask[i] = False
                i += 1
        elif state == "str":
            if c == "\\" and i + 1 < n:
                mask[i] = mask[i + 1] = False
                i += 2
            elif c == '"':
                mask[i] = False
                i += 1
                state = "code"
            else:
                if c != "\n":
                    mask[i] = False
                i += 1
        elif state == "raw":
            if c == '"' and src[i + 1 : i + 1 + raw_hashes] == "#" * raw_hashes:
                for p in range(i, i + 1 + raw_hashes):
                    mask[p] = False
                i += 1 + raw_hashes
                state = "code"
            else:
                if c != "\n":
                    mask[i] = False
                i += 1
    return mask


def next_starts_new_block(stripped: str) -> bool:
    if not stripped or stripped.startswith(SKIP_PREFIX):
        return False
    if "=>" in stripped:  # a match arm, keep arms tight
        return False
    first = re.match(r"[A-Za-z_!]+", stripped)
    return not (first and first.group() in ITEM_KW)


def space_blocks(src: str) -> str:
    mask = code_mask(src)
    lines = src.split("\n")
    # Character offset of each line start, to look braces up in the mask.
    starts = []
    off = 0
    for ln in lines:
        starts.append(off)
        off += len(ln) + 1

    out = []
    for k, line in enumerate(lines):
        out.append(line)
        stripped = line.strip()
        if stripped != "}" or (len(line) - len(line.lstrip())) == 0:
            continue
        brace = starts[k] + line.index("}")
        if not mask[brace]:  # a '}' living inside a string or comment
            continue
        if k + 1 >= len(lines):
            continue
        if next_starts_new_block(lines[k + 1].strip()):
            out.append("")
    return "\n".join(out)


def main(argv: list[str]) -> int:
    check = argv and argv[0] == "--check"
    files = argv[1:] if check else argv
    if not files:
        print("usage: tools/space_blocks.py [--check] <files...>", file=sys.stderr)
        return 2
    stale = []
    for path in files:
        with open(path, encoding="utf-8") as f:
            src = f.read()
        spaced = space_blocks(src)
        if spaced == src:
            continue
        if check:
            stale.append(path)
        else:
            with open(path, "w", encoding="utf-8", newline="\n") as f:
                f.write(spaced)
    if check and stale:
        print("blocks need spacing (run tools/fmt):", file=sys.stderr)
        for p in stale:
            print(f"  {p}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
