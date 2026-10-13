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
# ⚠️ cursor 和 gemini 不是这个形状,单独处理:
#   cursor   不是文件,是**目录** —— 每项目一个目录,内含 state.vscdb(SQLite)+
#            chat_dump.json(prompts + generations)。glob 只取文件会把 13 个全漏掉。
#   gemini   只有 .project_root 标记文件和一个 GEMINI.md,**没有对话内容**,0 轮正确。
#
# 用法:
#   bash scripts/extract-session-context.sh <session文件> [更多文件...]
#   bash scripts/extract-session-context.sh --index resume-contexts/INDEX.csv
#   bash scripts/extract-session-context.sh --cursor      # 抽 cursor 的 13 个目录
#
# 输出:resume-contexts/<项目>__<文件名>.md,每轮带「原文件第 N 行」。
#
# 🔴 索引生成器必须输出**绝对路径**。这个错犯了两次:索引在 AgentHistory/raw 下生成,
#    路径是相对的,抽取器在仓库下跑就什么都找不到 —— 而失败样子是「0 轮」,很容易
#    误读成「这些 session 是空的」。
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

# ── cursor:目录形状,另一种解析 ───────────────────────────────────────
def extract_cursor(base, outdir):
    import glob as _g
    n = t = 0
    for d in sorted(_g.glob(os.path.join(base, '*/'))):
        f = os.path.join(d, 'chat_dump.json')
        if not os.path.isfile(f):
            continue
        proj = os.path.basename(d.rstrip('/')).split('__')[0]
        try:
            j = json.load(open(f, encoding='utf-8'))
        except Exception as e:
            print(f"  ❌ {proj}: {e}"); continue
        prompts = j.get('prompts') or []
        gens = j.get('generations') or []
        rows = []
        for i, pr in enumerate(prompts):
            a = pr.get('text') or pr.get('prompt') or (pr if isinstance(pr, str) else '')
            if str(a).strip(): rows.append(('你', str(a).strip()))
            if i < len(gens):
                g = gens[i]
                b = g.get('text') or g.get('response') or (g if isinstance(g, str) else '')
                if str(b).strip(): rows.append(('助手', str(b).strip()))
        fn = os.path.join(outdir, f"cursor__{proj}__chat_dump.md")
        with open(fn, 'w', encoding='utf-8') as o:
            o.write(f"# 完整会话上下文 · cursor/{proj}\n\n```\n")
            o.write(f"来源文件   {f}\n")
            o.write(f"格式       cursor chat_dump.json(prompts {len(prompts)} + generations {len(gens)})\n")
            o.write(f"对话轮数   {len(rows)} 轮(原文全文,未总结)\n```\n\n---\n\n")
            for who, tx in rows:
                o.write(f"## {who}\n\n{tx}\n\n")
        n += 1; t += len(rows)
        print(f"  ✅ cursor/{proj:24s} {len(rows):4d} 轮")
    print(f"\ncursor 合计 {n} 个项目,{t} 轮")
    return n, t

args = [a for a in sys.argv[1:] if not a.startswith('--')]
files = []
if '--index' in sys.argv:
    idx = sys.argv[sys.argv.index('--index') + 1]
    for r in csv.DictReader(open(idx)):
        files.append((r['file'], r['project']))
else:
    files = [(a, None) for a in args]

if '--cursor' in sys.argv:
    _base = os.environ.get('CURSOR_DIR',
        os.path.expanduser('~/Documents/GitHub/AgentHistory/raw/cursor'))
    extract_cursor(_base, os.environ.get('OUT_DIR', '.'))
    raise SystemExit(0)

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
