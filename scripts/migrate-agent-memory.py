#!/usr/bin/env python3
"""migrate-agent-memory.py — 把 Aegis 的 agent memory 迁进 SeatLoom 仓库。

为什么迁:那个 memory 目录在张先生的 Mac 上,每次写入都要他 approve。
他出门或睡觉时,Aegis 就记不了东西 —— 而"记不了"恰好发生在最该记的时候(出事的时候)。
迁到仓库里,Aegis 自己就能维护。

为什么不是一次性搬运:落地形状**对齐 memories 表的列**,
这样等 topics 能创建、写入路径打通之后,洗数据流水线可以直接把这些文件灌进库,
不需要再转换一次。张先生定过:能一步到位就别做一次性的过渡产物。

诚实标注:`curation: mechanical` —— 类型和置信度是按规则机械映射的,不是人工判定的。
把机械映射包装成精炼结果,比不迁更糟:以后没人分得清哪条是真判断。

用法: python3 scripts/migrate-agent-memory.py [--dry-run]
"""
import os
import re
import sys

SRC = ("/Users/jyxc-dz-0100609/.stepgo/agents/claude/launch/"
       "ffee1c78-0db9-49f9-a1d2-87e43a4d3fb7/projects/"
       "-Users-jyxc-dz-0100609-Documents-GitHub-seatloom/memory")
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DST = os.path.join(REPO, "docs/coordination/knowledge")

DRY = "--dry-run" in sys.argv

# 密钥样貌:命中就整条跳过并报错。仓库要推 GitLab,红线是绝不粘贴密钥。
SECRET = re.compile(r"(ak-|sk-|ghp_|AKIA|Bearer )[A-Za-z0-9_-]{12,}|[A-Za-z0-9]{60,}")

# 他的原话标记 —— 命中则 confidence=quoted(引号里是他说的),否则 inferred(我推断的)
QUOTED = re.compile(r"[「『]|原话|he said|verbatim")


def split_front(text):
    """拆出 frontmatter 和正文。没有 frontmatter 就返回 (None, 全文)。"""
    if not text.startswith("---"):
        return None, text
    end = text.find("\n---", 3)
    if end < 0:
        return None, text
    return text[3:end].strip(), text[end + 4:].lstrip("\n")


def field(front, key):
    if not front:
        return ""
    m = re.search(rf"^\s*{re.escape(key)}:\s*(.*)$", front, re.M)
    if not m:
        return ""
    return m.group(1).strip().strip('"').strip("'")


def main():
    if not os.path.isdir(SRC):
        print(f"❌ 找不到源目录 {SRC}")
        return 1
    files = sorted(f for f in os.listdir(SRC)
                   if f.endswith(".md") and f != "MEMORY.md")
    if not files:
        print("❌ 源目录里没有 memory 文件")
        return 1

    if not DRY:
        os.makedirs(DST, exist_ok=True)

    rows, skipped = [], []
    for fn in files:
        with open(os.path.join(SRC, fn), encoding="utf-8") as fh:
            raw = fh.read()

        bad = [ln for ln in raw.splitlines() if SECRET.search(ln)]
        if bad:
            skipped.append((fn, bad[0][:70]))
            continue

        front, body = split_front(raw)
        name = field(front, "name") or fn[:-3]
        desc = field(front, "description")
        mtype = field(front, "type")

        kind = {"feedback": "preference", "user": "preference",
                "project": "fact", "reference": "artifact"}.get(mtype, "fact")
        conf = "quoted" if QUOTED.search(body) else "inferred"

        out_rel = f"docs/coordination/knowledge/{name}.md"
        header = [
            "---",
            f"memory_id: {name}",
            "project_id: seatloom",
            "topic_id: seatloom/operating-knowledge",
            f"kind: {kind}",
            f"confidence: {conf}",
            "status: active",
            "tier: warm",
            "source_kind: file",
            f"source_ref: agent-memory/{fn}",
            "l2_ref_kind: file",
            f"l2_file_path: {out_rel}",
            "curation: mechanical",
            f"l0_summary: {desc}" if desc else "l0_summary: (原文件无 description)",
            "---",
            "",
        ]
        # 正文从第几行开始 —— L2 的行号区间要指得准,不能是"在这个文件里某处"
        start = len(header) + 1
        nbody = len(body.rstrip().splitlines())
        content = "\n".join(header) + body.rstrip() + "\n"
        content = content.replace(
            "curation: mechanical",
            f"curation: mechanical\nl2_line_start: {start}\n"
            f"l2_line_end: {start + nbody - 1}", 1)

        if not DRY:
            with open(os.path.join(DST, f"{name}.md"), "w", encoding="utf-8") as fh:
                fh.write(content)
        rows.append((name, kind, conf, desc))

    # 索引
    if not DRY:
        idx = [
            "# SeatLoom 运行知识库",
            "",
            "Aegis 的 agent memory 迁到这里,原因:那个目录每次写入都要张先生 approve,",
            "于是他出门或睡觉时 Aegis 就记不了东西 —— 而「记不了」恰好发生在最该记的时候。",
            "",
            "**frontmatter 对齐 `memories` 表的列**,所以等写入路径打通后可以直接灌进库,",
            "不需要再转换一次。",
            "",
            "⚠️ **`curation: mechanical`** —— `kind` 和 `confidence` 是按规则机械映射的",
            "(feedback→preference、project→fact;正文出现他的原话标记则 quoted,否则 inferred),",
            "**不是人工判定**。灌库前应有一次精炼。把机械映射当成精炼结果,以后没人分得清哪条是真判断。",
            "",
            f"共 {len(rows)} 条。",
            "",
            "| 记忆 | 类型 | 置信度 | 一句话 |",
            "|---|---|---|---|",
        ]
        for name, kind, conf, desc in rows:
            d = (desc or "").replace("|", "\\|")
            idx.append(f"| [{name}]({name}.md) | {kind} | {conf} | {d} |")
        with open(os.path.join(DST, "INDEX.md"), "w", encoding="utf-8") as fh:
            fh.write("\n".join(idx) + "\n")

    print(f"{'(演练) ' if DRY else ''}迁入 {len(rows)} 条 -> docs/coordination/knowledge/")
    kinds = {}
    confs = {}
    for _, k, c, _ in rows:
        kinds[k] = kinds.get(k, 0) + 1
        confs[c] = confs.get(c, 0) + 1
    print(f"  类型分布   {kinds}")
    print(f"  置信度分布 {confs}")
    if skipped:
        print(f"  ❌ 因疑似密钥跳过 {len(skipped)} 条(**这些没有迁入,需要人看**):")
        for fn, ln in skipped:
            print(f"      {fn}: {ln}")
        return 1
    print("  ✅ 无因密钥跳过的文件")
    return 0


if __name__ == "__main__":
    sys.exit(main())
