#!/usr/bin/env python3
"""LAW-16: reject CJK/non-English text in staged added lines (English-only).

Default rule (English-only, LAW-16):
  A staged added line may not contain CJK ideographs or CJK punctuation/symbol
  ranges at all, regardless of whether the same wording exists in the base
  tree. New and re-touched lines alike must be English. Locale/translation
  resources are exempt (see LOCALE_EXEMPT_PREFIXES) because CJK is the product
  there; binding documents additionally get a whole-file check.

One-commit waiver (2026-11-16 maintainer instruction, single use, expired):
  While the commit's base tree is LEGACY_BASE_SHA the gate falls back to
  comparing against the base tree, so a line only fails when it carries CJK
  wording that is not already present verbatim in that file at the base. This
  existed solely to land the governance squash onto fdb8eb1, whose inherited
  base body is still Chinese. The waiver keys off that exact base commit, so it
  is inert for every commit made on top of the squash: those are strict again.
  Anything else -- new files, new wording, binding docs -- was never waived.
"""
import re
import subprocess
import sys

CJK = re.compile(r"[\u3400-\u4DBF\u4E00-\u9FFF\uF900-\uFAFF\u3000-\u303F\uFF00-\uFFEF]")

# Paths where CJK is allowed as translated UI locale content.
LOCALE_EXEMPT_PREFIXES = (
    'desktop-tauri/ui/i18n/',
    'desktop-tauri/i18n/',
    'desktop-tauri/mobile/public/locales/',
)

# Base commit of the one-time squash waiver described in the module docstring.
LEGACY_BASE_SHA = 'fdb8eb1d81e54f14121dd9a4f181b43956c3caa1'


def run(args):
    return subprocess.run(
        args,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding='utf-8',
        errors='replace',
    )


def norm(path):
    return path.replace('\\', '/')


def is_binding_doc(path):
    return norm(path).startswith('binding-docs/')


def is_locale_exempt(path):
    p = norm(path)
    return any(p.startswith(x) for x in LOCALE_EXEMPT_PREFIXES)


def cjk_runs(text):
    """Maximal runs of CJK characters, in order of appearance."""
    runs = []
    cur = []
    for ch in text:
        if CJK.match(ch):
            cur.append(ch)
        elif cur:
            runs.append(''.join(cur))
            cur = []
    if cur:
        runs.append(''.join(cur))
    return runs


_BASE_CACHE = {}


def base_runs(path):
    """CJK runs already present in this path at the base tree (HEAD)."""
    if path not in _BASE_CACHE:
        show = run(['git', 'show', 'HEAD:' + path])
        _BASE_CACHE[path] = cjk_runs(show.stdout) if show.returncode == 0 else []
    return _BASE_CACHE[path]


def new_runs(body, known):
    """CJK runs of this line whose wording does not occur in the base file."""
    return [r for r in cjk_runs(body) if not any(r in k for k in known)]


def main():
    head = run(['git', 'rev-parse', 'HEAD']).stdout.strip()
    legacy = head == LEGACY_BASE_SHA
    if legacy:
        print('LAW-16: one-time waiver active for base commit %s (squash only).'
              % LEGACY_BASE_SHA[:7], file=sys.stderr)

    diff = run(['git', '-c', 'core.quotepath=false', 'diff', '--cached', '-U0',
                '--no-color', '--diff-filter=AM'])
    if diff.returncode != 0:
        print(diff.stderr, file=sys.stderr)
        return 2

    current = None
    violations = []
    for line in diff.stdout.splitlines():
        if line.startswith('+++ b/'):
            current = line[6:]
            continue
        if line.startswith('+++') or line.startswith('---'):
            continue
        if not line.startswith('+'):
            continue
        if not current or is_locale_exempt(current):
            continue
        body = line[1:]
        if not CJK.search(body):
            continue
        fresh = new_runs(body, base_runs(current)) if legacy else cjk_runs(body)
        if fresh:
            violations.append((current, body.strip()[:200], fresh))

    if violations:
        label = 'newly introduced CJK/non-English text' if legacy else 'CJK/non-English text'
        print('LAW-16: %s in staged added line:' % label, file=sys.stderr)
        for path, body, fresh in violations[:20]:
            print(f'File: {path}', file=sys.stderr)
            print(f'  text: {" | ".join(fresh[:6])}', file=sys.stderr)
            print(f'  line: {body}', file=sys.stderr)
        if len(violations) > 20:
            print(f'... and {len(violations) - 20} more line(s)', file=sys.stderr)
        print('Fork development is English-only: write English, or keep the inherited'
              ' base line untouched.', file=sys.stderr)
        return 1

    names = run(['git', '-c', 'core.quotepath=false', 'diff', '--cached', '--name-only'])
    if names.returncode != 0:
        print(names.stderr, file=sys.stderr)
        return 2
    for path in [x.strip() for x in names.stdout.splitlines() if x.strip()]:
        if not is_binding_doc(path):
            continue
        show = run(['git', 'show', ':' + path])
        if show.returncode != 0:
            continue
        m = CJK.search(show.stdout)
        if not m:
            continue
        lineno = show.stdout.count('\n', 0, m.start()) + 1
        lines = show.stdout.splitlines()
        snippet = lines[lineno - 1][:200] if lineno - 1 < len(lines) else ''
        print(f'LAW-16: CJK/non-English text detected in binding document {path}:{lineno}',
              file=sys.stderr)
        print(snippet, file=sys.stderr)
        return 1

    return 0


if __name__ == '__main__':
    sys.exit(main())
