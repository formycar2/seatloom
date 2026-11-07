#!/usr/bin/env bash
# worktree-root.sh — 权威地答出「席位工作区都在哪个目录下」。
#
# 为什么需要它(2026-11-07):
#   prepare-sandbox-build.sh 和 replace-sandbox.sh 都要知道「哪个目录下面挂着各个
#   席位的工作区」。两边的老写法都是同一句猜:
#       WROOT="$(cd "$REPO/.." && pwd)/seatloom-worktrees"
#   在**主检出**里跑时对;在**席位自己的工作区**里跑时错 —— 那时的 $REPO/.. 已经是
#   seatloom-worktrees,再拼一层就成了 .../seatloom-worktrees/seatloom-worktrees。
#
#   实测后果:从席位工作区跑 prepare-sandbox-build.sh --build-only 时,那个目录不
#   存在,于是循环一次都没进过,报出来的是「这条分支不是任何工作区检出的分支」——
#   **把人指向错误方向的失败**:分支确实被检出了,只是脚本找的地方不对。
#
#   replace-sandbox.sh 上有一个补丁式的兜底,而 prepare-sandbox-build.sh 没有 ——
#   同一个 bug,一处修了一处没修,这正是它复发的原因。现在两边共用这一个 resolver。
#
# 🔴 权威来源是 `git worktree list`,不是目录名。
#    猜目录名会在三种情况下错:席位工作区本身在跑(拼出双层)、工作区根换了名字、
#    路径里有空格。git worktree list 给出的是 git 自己记录的绝对路径。
#
# 用法(被 source,不是被执行):
#   . "$(dirname "$0")/worktree-root.sh"
#   WROOT="$(resolve_worktree_root "$REPO")" || exit 1
#
# 契约:
#   - 成功:stdout 打一行,是各个席位工作区的**父目录**;退出码 0。
#   - 失败:stderr 说明为什么定不下来;退出码非零。调用方必须 fail,
#     不能拿一个猜出来的目录继续。

# 所有工作区的父目录,一个一行,已去重。
# 用一个临时文件而不是管道里的 while —— 管道里的 while 跑在子 shell 里,
# 计数拿不出来(实测踩到:第一版用 heredoc 套 while,数出来永远是 0 或 1)。
_worktree_parents(){
  local repo="${1:-.}"
  local tmp
  tmp="$(mktemp)" || return 1
  git -C "$repo" worktree list --porcelain 2>/dev/null \
    | sed -n 's/^worktree //p' > "$tmp"
  # dirname 逐行做,空行跳过
  local wt
  while IFS= read -r wt; do
    [ -n "$wt" ] || continue
    dirname "$wt"
  done < "$tmp" | sort -u
  rm -f "$tmp"
}

# 某个父目录下面挂了多少个工作区
_count_under(){
  local parent="$1" repo="$2" tmp wt n
  tmp="$(mktemp)" || { echo 0; return; }
  git -C "$repo" worktree list --porcelain 2>/dev/null \
    | sed -n 's/^worktree //p' > "$tmp"
  n=0
  while IFS= read -r wt; do
    [ -n "$wt" ] || continue
    # 🔴 比的是**直接父目录**,不是前缀。
    #    第一版用 case "$wt" in "$parent"/*) —— 前缀匹配。于是
    #    /Users/.../GitHub/seatloom-worktrees/flux 也匹配父目录
    #    /Users/.../GitHub,数出来 9 比 8 大,选中的是 GitHub 而不是
    #    seatloom-worktrees。**一个差一层的计数让 resolver 稳定地返回错的目录。**
    #    实测,不是推理。正确做法:取 wt 自己的 dirname,和 parent 全等。
    if [ "$(dirname "$wt")" = "$parent" ]; then n=$((n+1)); fi
  done < "$tmp"
  rm -f "$tmp"
  printf '%s\n' "$n"
}

# 定出「席位工作区根」。
#
# 判定规则,按优先级:
#   1. SEATLOOM_WORKTREE_ROOT 环境变量,且目录真的存在 —— 显式指定优先,
#      因为只有调用方知道自己的部署形状。
#   2. git worktree list 里,**挂最多工作区**的那个父目录,且至少挂两个。
#      主检出的父目录通常只挂它自己;席位工作区根下面挂着好几个席位。
#      「至少两个」是承重条件:只有一个的话,它很可能就是主检出的父目录。
#   3. 老写法猜出来的那个目录,如果存在。放在最后,因为它正是出过问题的那个猜测。
resolve_worktree_root(){
  local repo="${1:-.}"
  local explicit="${SEATLOOM_WORKTREE_ROOT:-}"

  if [ -n "$explicit" ]; then
    if [ -d "$explicit" ]; then
      printf '%s\n' "$explicit"
      return 0
    fi
    echo "SEATLOOM_WORKTREE_ROOT 指向的目录不存在:$explicit" >&2
    return 1
  fi

  local parents
  parents="$(_worktree_parents "$repo")"
  if [ -z "$parents" ]; then
    echo "git worktree list 没给出任何工作区 —— 读不到 git,或者这里不是 git 仓库" >&2
    return 1
  fi

  local best="" best_n=0 d n
  while IFS= read -r d; do
    [ -n "$d" ] || continue
    n="$(_count_under "$d" "$repo")"
    if [ "$n" -gt "$best_n" ]; then best="$d"; best_n="$n"; fi
  done <<EOF
$parents
EOF

  if [ -n "$best" ] && [ "$best_n" -ge 2 ]; then
    printf '%s\n' "$best"
    return 0
  fi

  local guessed
  guessed="$(cd "$repo/.." && pwd)/seatloom-worktrees"
  if [ -d "$guessed" ]; then
    printf '%s\n' "$guessed"
    return 0
  fi

  echo "定不下席位工作区根。git worktree list 的父目录:" >&2
  printf '  %s\n' $parents >&2
  echo "猜的 $guessed 也不存在。用 SEATLOOM_WORKTREE_ROOT 显式指定。" >&2
  return 1
}
