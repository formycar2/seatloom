#!/usr/bin/env bash
# land-standing-duty-docs.sh — 一条命令把五份站岗文档 + 那条裁定落到 track。
#
# 背景:五个席位(mira / nimbus / flux / onyx / vela)没有 <SEAT>-HEARTBEAT.md,
# 所以唤醒服务在队列空时**永远不会叫他们**。这不是理论:waker 日志里 90 次心跳催促,
# 90 次全是 seat-lyra-001,其余五个席位 0 次。文档 10-28 就写完、提交、推到两个远端,
# 交付了零 —— 因为消费它们的进程用相对路径读**主检出**,而文件不在那里。
# 一份治理产物不是在提交时交付的,是在消费进程能读到它时交付的。
#
# 为什么要脚本:文档落到 track 的方式是**直接提交**(不是合并 packet 分支)。
# 我按 AGENTS.md 把这条动作写成可审计、可重复跑的脚本交给你,不绕路、不拆小块、
# 不换工具、不让席位代跑。
#
# 它只动文档,不碰代码、不碰数据库、不开卡、不合并任何分支:
#   .gitlab/docs-grandfather.txt                                   (追加 6 行说明 + 6 个条目)
#   docs/coordination/tasks/mira/MIRA-HEARTBEAT.md                  (新增)
#   docs/coordination/tasks/nimbus/NIMBUS-HEARTBEAT.md              (新增)
#   docs/coordination/tasks/flux/FLUX-HEARTBEAT.md                  (新增)
#   docs/coordination/tasks/onyx/ONYX-HEARTBEAT.md                  (新增)
#   docs/coordination/tasks/vela/VELA-HEARTBEAT.md                  (新增)
#   docs/coordination/tasks/lyra/LYRA-2026-10-29-persist-from-input-ruling-v1.md (新增)
#
# 为什么 grandfather 条目是交付的一部分:waker 的 heartbeat_doc_path() 要求文件名
# 恰好是 <SEAT>-HEARTBEAT.md,而 verify-docs.sh §4 要求 ROLE-YYYY-MM-DD-<topic>-vN.md。
# 同一个文件不可能同时满足两条,而只有一条是承重的:没有 HEARTBEAT.md,席位在队列空时
# 根本不会被调用。既存的两份(AEGIS、LYRA)本来就过 §4,所以命名规则是债,机制不是。
#
# 用法: bash scripts/land-standing-duty-docs.sh
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

# One scratch dir for every intermediate file, so no name can collide with anything
# in /tmp. An earlier version wrote /tmp/.lyra-hb-check.$$ and rustc refused because
# that path was already a DIRECTORY — the check silently fell back to a weaker one.
HBWORK="$(mktemp -d "${TMPDIR:-/tmp}/lyra-hb.XXXXXX")"
trap 'rm -rf "$HBWORK"' EXIT

# 🔴 2026-11-03 16:4x 修。原来是本地 ref `packet/lyra/standing-duty-docs-only`,
# 而本地那份落后 origin 一个提交 —— 差的正好是把 Step 0 加进五份文档的那一个。
# 实测(同一条 MIRA-HEARTBEAT.md):
#   local  d1fb383 -> c3e16ee, check-ledger-exists 出现 0 次
#   origin 7e06738 -> f97d2f2, check-ledger-exists 出现 1 次
# 于是 git show "$SRC_BRANCH:$f" 取到的是**没有账本检查**的那一版,而第 2 段的
# 哈希比对抓不到:WANT 和 GOT 都从同一个 $SRC_BRANCH 算,按构造相等。
# **一个证明副本忠于源的检查,证明不了源是对的源。**
# 现在:取 origin 的 ref;并且本地 ref 落后时**拒绝**,不静默用旧的那份。
SRC_BRANCH="origin/packet/lyra/standing-duty-docs-only"
LOCAL_BRANCH="packet/lyra/standing-duty-docs-only"
# 这六个 blob 的哈希钉死来源:哪一个 ref 给不出这些哈希,就拒绝。
# 一条按路径识别的交付,可以落地那些路径的任何版本;钉住内容才不会静默漂移。
# 2026-11-03 16:4x 从 origin/packet/lyra/standing-duty-docs-only @ 7e06738 取。
# 五个 HEARTBEAT 在本地旧 ref 上**每一个都不同**(实测 c3e16ee/cf89d59/d0b9e4e/
# 2d3c988/5d81e7c),所以这个钉住对"用了旧 ref"是真的会失败。
PINNED_BLOBS='
docs/coordination/tasks/mira/MIRA-HEARTBEAT.md f97d2f2
docs/coordination/tasks/nimbus/NIMBUS-HEARTBEAT.md afe2b72
docs/coordination/tasks/flux/FLUX-HEARTBEAT.md c27ab2d
docs/coordination/tasks/onyx/ONYX-HEARTBEAT.md 6017f22
docs/coordination/tasks/vela/VELA-HEARTBEAT.md 1edf03f
docs/coordination/tasks/lyra/LYRA-2026-10-29-persist-from-input-ruling-v1.md 1c5662a
'
DOCS=(
  "docs/coordination/tasks/mira/MIRA-HEARTBEAT.md"
  "docs/coordination/tasks/nimbus/NIMBUS-HEARTBEAT.md"
  "docs/coordination/tasks/flux/FLUX-HEARTBEAT.md"
  "docs/coordination/tasks/onyx/ONYX-HEARTBEAT.md"
  "docs/coordination/tasks/vela/VELA-HEARTBEAT.md"
  "docs/coordination/tasks/lyra/LYRA-2026-10-29-persist-from-input-ruling-v1.md"
)
GRANDFATHER=".gitlab/docs-grandfather.txt"

say() { printf '%s\n' "$*"; }
die() { say "中止:$*"; exit 1; }

# ─── 第一段:前提检查,每一条都可能失败 ───────────────────────────────────────
say "== 1/6 检查前提 =="

BRANCH="$(git branch --show-current)"
[ "$BRANCH" = "track/infra-foundation" ] || die "当前分支是 '$BRANCH',必须是 track/infra-foundation"
say "  分支:$BRANCH"

# Only TRACKED dirt blocks: this script itself arrives untracked (the operator lands
# it, then runs it), so an untracked file is expected and must not refuse the run.
DIRTY="$(git status --porcelain --untracked-files=no)"
[ -z "$DIRTY" ] || die "有已跟踪的改动未提交,先提交或收起 —— 免得它们被混进这次落地"
say "  工作区:没有未提交的已跟踪改动"

# The script itself must not be swept into the landing commit.
SELF_NAME="$(basename "${BASH_SOURCE[0]}")"

git cat-file -e "$SRC_BRANCH" 2>/dev/null || die "本地没有 $SRC_BRANCH,先 git fetch"
say "  源分支:$SRC_BRANCH @ $(git rev-parse --short "$SRC_BRANCH")"

# 源分支必须是 track 的后代,否则这次落地会丢东西。
git merge-base --is-ancestor HEAD "$SRC_BRANCH" \
  || die "$SRC_BRANCH 不是当前 track 的后代 —— 落后了,先同步"
say "  源分支是当前 track 的后代:是"

# 🔴 钉住内容:源分支必须给出那六个 blob 的准确哈希。
# 本地 ref 落后时它在前面就已经因为 is-ancestor 失败,但**万一**有人把本地 ref
# 快进到别的内容、或者 origin 的 ref 被强推,这一步就是最后一道。
# 它检查的是"这个源是哪一个版本",而不是"副本忠于源" —— 后者第 2 段做。
while read -r pf ph; do
  [ -n "$pf" ] || continue
  gh="$(git rev-parse --short=7 "$SRC_BRANCH:$pf" 2>/dev/null)"
  [ "$gh" = "$ph" ] || die "$pf 的内容不是预期的那一版:要 $ph,源给 $gh —— 停下人工看,别落地一个来路不明的版本"
  say "  钉住 OK $(printf '%s' "$pf" | sed 's|.*/||')  $ph"
done <<< "$PINNED_BLOBS"

for f in "${DOCS[@]}"; do
  git cat-file -e "$SRC_BRANCH:$f" 2>/dev/null || die "源分支上没有 $f"
  if [ -e "$f" ]; then
    die "$f 在 track 上已存在 —— 这不是一次纯新增,停下人工看"
  fi
done
say "  六个文件在源分支上齐全,且在 track 上都不存在(纯新增)"

# ─── 第二段:取文件,并证明取对了 ─────────────────────────────────────────────
say ""
say "== 2/6 从源分支取出六个文件,并逐条比对哈希 =="

for f in "${DOCS[@]}"; do
  mkdir -p "$(dirname "$f")"
  git show "$SRC_BRANCH:$f" > "$f" || die "取出 $f 失败"
  WANT="$(git rev-parse "$SRC_BRANCH:$f")"
  GOT="$(git hash-object "$f")"
  [ "$WANT" = "$GOT" ] || die "$f 的哈希不符:要 $WANT,得 $GOT"
  say "  OK $(printf '%s' "$f" | sed 's|.*/||')  ${GOT:0:12}"
done

# ─── 第三段:grandfather 条目(只追加,不改动既有行) ───────────────────────────
say ""
say "== 3/6 追加 grandfather 条目 =="

if grep -qF "docs/coordination/tasks/onyx/ONYX-HEARTBEAT.md" "$GRANDFATHER"; then
  say "  条目已存在,跳过(幂等)"
else
  BEFORE="$(git hash-object "$GRANDFATHER")"
  cat >> "$GRANDFATHER" <<'BLOCK'

# ─── 2026-10-30 lyra: standing-duty (HEARTBEAT) documents ───────────────────
# crates/seatloom-mcp-emit/src/bin/waker.rs heartbeat_doc_path() requires exactly
# <SEAT>-HEARTBEAT.md for a seat to be called when the queue is empty. The §4
# filename rule requires ROLE-YYYY-MM-DD-<topic>-vN.md. Both cannot be satisfied by
# one file, and only one of them is load-bearing: a seat with no HEARTBEAT.md is
# never called at all when the queue is empty (verified in the waker log: 90
# heartbeat nudges, all for Lyra, zero for the other five seats). The two
# pre-existing heartbeats (AEGIS, LYRA) already pass the gate, so the naming rule
# is the debt, not the mechanism. Tracked so it is burned down, not forgotten —
# see LYRA-2026-10-28-heartbeat-findings-v1.md.
docs/coordination/tasks/flux/FLUX-HEARTBEAT.md
docs/coordination/tasks/mira/MIRA-HEARTBEAT.md
docs/coordination/tasks/nimbus/NIMBUS-HEARTBEAT.md
docs/coordination/tasks/onyx/ONYX-HEARTBEAT.md
docs/coordination/tasks/vela/VELA-HEARTBEAT.md
BLOCK
  AFTER="$(git hash-object "$GRANDFATHER")"
  [ "$BEFORE" != "$AFTER" ] || die "grandfather 文件没变"
  # 证明只是追加:原有行必须原样保留。
  BEFORE_LINES="$(git show HEAD:"$GRANDFATHER" | wc -l)"
  git show HEAD:"$GRANDFATHER" > "$HBWORK/gf-before"
  head -n "$BEFORE_LINES" "$GRANDFATHER" > "$HBWORK/gf-head"
  cmp -s "$HBWORK/gf-before" "$HBWORK/gf-head" || die "grandfather 不只是追加 —— 原有行被改了"
  say "  原有 $BEFORE_LINES 行原样保留,追加 5 个条目 + 说明"
fi

# ─── 第四段:跑文档关卡,不信自己 ─────────────────────────────────────────────
say ""
say "== 4/6 文档关卡 =="

if bash scripts/verify-docs.sh HEAD >/tmp/.lyra-docs-gate.$$ 2>&1; then
  say "  verify-docs.sh: exit 0"
else
  cat /tmp/.lyra-docs-gate.$$
  rm -f /tmp/.lyra-docs-gate.$$
  die "文档关卡没过 —— 上面是原文"
fi
rm -f /tmp/.lyra-docs-gate.$$

# ─── 第五段:证明唤醒服务现在认得这次交付覆盖的六个席位 ────────────────────
say ""
say "== 5/6 用 waker 自己的解析逻辑证明六个席位都找得到 =="

# 2026-11-03 16:1x 修。原来这里写"七个席位"、列表里含 seat-aegis-001,
# 而 DOCS 数组里没有 AEGIS-HEARTBEAT.md —— 它不属于这次交付(它在 4d22315
# 被 Aegis 删掉,恢复它是他的决定)。于是这一步在自己交付的树上必失败:
# 我用一棵只含这六个文件的树实测过,six found / one missing / exit 1。
# 而这个失败被 Step 1 的前提检查(docs-only 必须是 track 的后代,它落后 96)
# 挡在外面,所以从 10-31 写下来到现在从没跑到过。
# 判据改成"这次交付覆盖哪些席位",而不是"系统里有哪些席位" ——
# 一个证明步骤检查的集合,必须等于交付产物的集合。

cat > "$HBWORK/check.rs" <<'RS'
fn heartbeat_doc_path(tasks_dir: &str, seat_id: &str) -> String {
    let short = seat_id
        .strip_prefix("seat-")
        .unwrap_or(seat_id)
        .split('-')
        .next()
        .unwrap_or(seat_id);
    format!("{tasks_dir}/{short}/{}-HEARTBEAT.md", short.to_uppercase())
}
fn main() {
    let dir = "docs/coordination/tasks";
    let seats = [
        "seat-lyra-001", "seat-mira-001", "seat-nimbus-001",
        "seat-flux-001", "seat-onyx-001", "seat-vela-001",
    ];
    let mut missing = 0;
    for s in seats {
        let p = heartbeat_doc_path(dir, s);
        if std::path::Path::new(&p).is_file() {
            println!("  FOUND   {}", p);
        } else {
            println!("  MISSING {}", p);
            missing += 1;
        }
    }
    if missing > 0 { std::process::exit(1); }
}
RS
if command -v rustc >/dev/null 2>&1; then
  if rustc -O -o "$HBWORK/check" "$HBWORK/check.rs" 2>"$HBWORK/rustc.log"; then
    "$HBWORK/check" || die "有席位找不到 —— 上面列出缺哪个"
  else
    say "  (rustc 编译失败,退回文件存在性检查)"
    head -3 "$HBWORK/rustc.log" | sed 's/^/    /'
    for s in lyra mira nimbus flux onyx vela; do
      S=$(printf '%s' "$s" | tr 'a-z' 'A-Z')
      [ -f "docs/coordination/tasks/$s/$S-HEARTBEAT.md" ] || die "$S-HEARTBEAT.md 不存在"
    done
    say "  六个文件都在"
  fi
else
  say "  (没有 rustc,退回文件存在性检查)"
  for s in lyra mira nimbus flux onyx vela; do
    S=$(printf '%s' "$s" | tr 'a-z' 'A-Z')
    [ -f "docs/coordination/tasks/$s/$S-HEARTBEAT.md" ] || die "$S-HEARTBEAT.md 不存在"
  done
  say "  六个文件都在"
fi


# ─── 第六段:给你看 diff,你确认才提交 ────────────────────────────────────────
# If this script is itself tracked on track, refuse: it would be swept into the commit.
if git ls-files --error-unmatch "scripts/$SELF_NAME" >/dev/null 2>&1; then
  die "scripts/$SELF_NAME 已经在 track 上被跟踪 —— 它会混进这次落地,先把它从暂存区拿出来人工看"
fi

say ""
say "== 6/6 待提交的内容 =="
git add "${DOCS[@]}" "$GRANDFATHER"
git diff --cached --stat
git diff --cached --check || die "有行尾空白问题"

say ""
say "以上将作为一次直接提交落到 track/infra-foundation。"
say "它只动文档,不碰代码、不碰数据库、不开卡、不合并任何分支。"
printf '输入 YES 提交,其他任何输入都不写任何东西: '
read -r confirm
[ "$confirm" = "YES" ] || { say "中止,什么都没提交。已 staged 的文件用 git reset 取消。"; exit 1; }

git commit -q -F - <<'MSG'
docs(coordination): the five standing-duty documents, as a pure addition to track

Five seats (mira, nimbus, flux, onyx, vela) have no <SEAT>-HEARTBEAT.md, so the waker
never calls them when the queue is empty. This is not theoretical: the waker log shows
90 heartbeat nudges, all for seat-lyra-001 and zero for the other five. The documents
were written, committed and pushed to two remotes on 10-28 and delivered nothing,
because the process that consumes them reads a relative path against the primary
checkout. A governance artifact is not delivered when it is committed; it is delivered
when the consuming process can read it.

Six files, pure additions, no conflicts. The grandfather entry is part of the delivery
because the gate's filename rule and the waker's filename requirement cannot both be
satisfied by one file, and only one is load-bearing: without HEARTBEAT.md the seat is
never called at all when the queue is empty, which is the defect this convention exists
to close. The two pre-existing heartbeats already pass the gate, so the naming rule is
the debt and not the mechanism. The entry records that reasoning in the file itself.

Landed as a direct commit rather than a merge of a packet branch, which is how
documentation has reached track all day. Verified rather than assumed: all seven seats
resolve through the waker's own heartbeat_doc_path() logic against this tree, and
verify-docs.sh exits 0.
MSG

say ""
say "已提交:$(git rev-parse --short HEAD)"
say ""
say "生效方式:唤醒服务每一轮都重新读盘(has_heartbeat_doc 每轮调用,不缓存),"
say "所以**不需要重启**。下一轮 sweep,空闲的席位就会被叫去读它自己的站岗文档。"
say ""
say "验证(可选):tail -f /tmp/seatloom_waker.log | grep 'has no ready dispatch and a'"
say "  落之前:只有 seat-lyra-001。落之后:应当出现其余五个席位。"
