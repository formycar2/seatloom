#!/usr/bin/env bash
# commit-aegis-work.sh — 提交 Aegis 在主检出里产出的文档和脚本。
#
# 为什么要脚本:Aegis 的 `git commit` 被权限分类器拦住了(和"改 live 数据库"同一类)。
# 这是反复出现的阻塞,所以做成一个可重复跑的通用脚本,而不是每次临时写一个。
#
# 刻意**不用 `git add -A`** —— 只加 Aegis 自己的产出范围:
#   AGENTS.md / scripts/ / docs/coordination/tasks/aegis/ / .seatloom/
# 席位的交付在它们自己的 worktree 和分支上,不归这里管。
#
# 加之前会扫一遍密钥样貌的串(ak-/sk-/ghp_/AKIA/Bearer/60+ 字符高熵),
# 命中就**停下不提交** —— 张先生的红线:绝不粘贴密钥、不复制到任何可外传位置。
#
# 用法: bash scripts/commit-aegis-work.sh
set -uo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 1

SCOPE=(AGENTS.md scripts/ docs/coordination/tasks/aegis/ .seatloom/)

cur="$(git rev-parse --abbrev-ref HEAD 2>/dev/null)" || { echo "❌ 不是 git 仓库"; exit 1; }
[ "$cur" = "track/infra-foundation" ] || { echo "❌ 当前在 ${cur},不是 track/infra-foundation"; exit 1; }

# ⚠️ 不能用 `mapfile` —— macOS 自带 bash 3.2 没有这个内建命令,它会静默读到 0 个文件,
#    然后脚本报"没有待提交的改动"并以 0 退出:一个**永远不会报错的假检查**,比没有检查更有害。
#    (第一版就是这个 bug,写完当场测出来的。)用 while read 循环,3.2 也能跑。
files=()
while IFS= read -r l; do
  [ -n "$l" ] && files+=("$l")
done < <(git status --porcelain -- "${SCOPE[@]}" | sed 's/^...//')

if [ "${#files[@]}" = "0" ]; then echo "没有待提交的改动,跳过。"; exit 0; fi

echo "待提交 ${#files[@]} 个文件:"
printf '  %s\n' "${files[@]}"
echo

echo "密钥扫描:"
hit=0
for f in "${files[@]}"; do
  [ -f "$f" ] || continue
  m="$(grep -nE '(ak-|sk-|ghp_|AKIA|Bearer )[A-Za-z0-9_-]{12,}|[A-Za-z0-9]{60,}' "$f" 2>/dev/null | head -3)"
  [ -n "$m" ] && { echo "  ⚠️ $f"; printf '      %s\n' "$m"; hit=1; }
done
if [ "$hit" != "0" ]; then
  echo
  echo "❌ 命中疑似密钥 —— **不提交**。把上面这段贴给 Aegis 判断。"
  exit 1
fi
echo "  ✅ 干净"
echo

git add -- "${SCOPE[@]}" || { echo "❌ git add 失败"; exit 1; }
n="${#files[@]}"
git commit -q -m "ops(aegis): supervisor-lane docs and scripts (${n} files)

Aegis cannot run git commit itself — the permission classifier blocks it, the
same way it blocks live database writes. Both blocks are correct, so this is
the sanctioned path: an auditable, repeatable script and one command for the
operator.

Scope is deliberately narrow — AGENTS.md, scripts/, the Aegis task folder and
the seat map — never git add -A, and every file is scanned for key-shaped
strings before staging. Seat deliveries live on their own branches and are not
touched here.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>" \
  && echo "✅ $(git log -1 --pretty='%h %s' | cut -c1-72)" \
  || { echo "❌ 提交失败"; exit 1; }
