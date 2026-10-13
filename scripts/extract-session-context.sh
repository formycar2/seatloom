#!/usr/bin/env bash
# extract-session-context.sh — 把原始 session 文件抽成「完整对话上下文 + 原始行号」。
#
# 为什么需要它:原始 session 文件 98% 是流式事件噪音(event_msg 占 70%)。
# 一个 104MB 的 codex session,真实对话文本只有 354KB。席位没法直接读那 104MB。
# 这个脚本抽出对话原文,每一轮都标注它在**原文件的第几行** —— 这样既能读,
# 也能回溯(L2 行号锚点指向原文件,原文只有一份,在磁盘上)。
#
# 这是「B 方案」的抽取半边:原文留磁盘,数据库/索引存指针。不复制 262MB 文本进库。
#
# 三种 harness 的内容位置不同(都实测过):
#   codex      payload.content
#   claude-code message.content      ← 第一版漏了这个,15 轮真实对话被漏掉
#   stepcode    只有 session.start/hook.trigger/session.end 遥测,没有对话内容
#
# 用法:
#   bash scripts/extract-session-context.sh <session文件> [更多文件...]
#   bash scripts/extract-session-context.sh --index resume-contexts/INDEX.csv
#
# 输出:resume-contexts/<项目>__<文件名>.md,每轮带「原文件第 N 行」。
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$REPO/resume-contexts"
mkdir -p "$OUT"

python3 - "$@" <<'PY'
import json, os, sys, csv, datetime

def text_of(j):
    """三种 harness 的内容位置都覆盖。返回 (role, text)。"""
    pl = j.get('payload') or j
    msg = j.get('message') or {}
    role = pl.get('role') or msg.get('role') or j.get('role')
    c = pl.get('content') or msg.get('content') or j.get('content')
    t = ''
    if isinstance(c, list):
        for x in c:
            if isinstance(x, dict):
                t += x.get('text', '') or ''
            elif isinstance(x, str):
                t += x
    elif isinstance(c, str):
        t = c
    return role, (t.strip() if t else '')

def project_of(cwd):
    if not cwd: return None
    for t in ('sf-code/stepmind/brainpp','sf-code/storage','sf-code/brainpp',
              'cn-gpu-infer-fabric','sf-code/stepatlas'):
        if t in cwd: return t
    if '/sf-code/' in cwd: return 'sf-code/其它'
    return None

def find_cwd(path):
    try:
        with open(path, encoding='utf-8', errors='ignore') as fh:
            for line in fh:
                if '"cwd"' not in line: continue
                try: j = json.loads(line)
                except Exception: continue
                c = j.get('cwd') or (j.get('payload') or {}).get('cwd')
                if c: return c
    except Exception:
        pass
    return None

def extract(path, project=None):
    cwd = find_cwd(path) or ''
    proj = project or project_of(cwd) or '未知归属'
    turns = []
    with open(path, encoding='utf-8', errors='ignore') as fh:
        for i, line in enumerate(fh, 1):
            if '"content"' not in line and '"text"' not in line: continue
            try: j = json.loads(line)
            except Exception: continue
            role, t = text_of(j)
            if role in ('user', 'assistant') and t:
                turns.append((i, role, t))
    safe = proj.replace('/', '__')
    fn = os.path.join(os.environ.get('OUT_DIR', '.'), f"{safe}__{os.path.basename(path)[:44]}.md")
    chars = sum(len(t) for _, _, t in turns)
    with open(fn, 'w', encoding='utf-8') as o:
        o.write(f"# 完整会话上下文 · {proj}\n\n```\n")
        o.write(f"来源文件   {path}\n")
        o.write(f"原始规模   {sum(1 for _ in open(path,encoding='utf-8',errors='ignore'))} 行 / "
                f"{os.path.getsize(path)/1e6:.1f} MB\n")
        o.write(f"对话轮数   {len(turns)} 轮(原文全文,未总结)\n")
        o.write(f"对话文本   {chars:,} 字符\n")
        o.write(f"工作目录   {cwd or '<无 cwd,归属靠内容判断>'}\n")
        o.write(f"最后修改   {datetime.datetime.fromtimestamp(os.path.getmtime(path)):%Y-%m-%d %H:%M}\n```\n\n---\n\n")
        for i, role, t in turns:
            o.write(f"## [原文件第 {i} 行] {'你' if role == 'user' else '助手'}\n\n{t}\n\n")
    return fn, len(turns), chars, proj

args = [a for a in sys.argv[1:] if not a.startswith('--')]
files = []
if '--index' in sys.argv:
    idx = sys.argv[sys.argv.index('--index') + 1]
    for r in csv.DictReader(open(idx)):
        files.append((r['file'], r['project']))
else:
    files = [(a, None) for a in args]

os.environ['OUT_DIR'] = os.environ.get('OUT_DIR', '.')
total_t = total_c = 0
for path, proj in files:
    if not os.path.isfile(path):
        print(f"  ❌ 不在: {path}"); continue
    fn, n, c, p = extract(path, proj)
    total_t += n; total_c += c
    print(f"  {'✅' if n else '· '} {n:5d} 轮 {c:>10,} 字符  {p:24s} {os.path.basename(fn)[:52]}")
print(f"\n合计 {len(files)} 个 session,{total_t} 轮,{total_c:,} 字符")
PY
