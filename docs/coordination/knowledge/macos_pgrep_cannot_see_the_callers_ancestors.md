---
memory_id: macos_pgrep_cannot_see_the_callers_ancestors
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: failure
confidence: inferred
status: active
tier: hot
source_kind: file
source_ref: scripts/restart-aegis.sh
l2_ref_kind: file
l2_file_path: scripts/restart-aegis.sh
l2_line_start: 30
l2_line_end: 45
curation: authored
l0_summary: macOS 的 pgrep 静默看不见调用者的祖先进程。Aegis 的 bash 命令是 pi 的子进程,所以 `pgrep -P <Aegis 的 pane_pid>` 永远返回空 —— 症状是"查不到",不是报错。查席位不会踩到(席位不是祖先),查 Aegis 自己必然踩到。改用 `ps -axo pid,ppid,command | awk '$2==pane_pid'`。
---

## 发生了什么

写 `scripts/restart-aegis.sh` 时,按 AGENTS.md 第 2 条的老办法找 Aegis 自己的进程:
`pgrep -P $(tmux list-panes -t Aegis-Supervisor -F '#{pane_pid}')`。
**返回空。** 而 pane 里明摆着跑着 pi(pid 3870,`ps -o ppid=` 显示它的父进程就是那个 pane_pid)。

同一个 tmux 面板、同一条命令,从**另一个 tmux 会话**里跑就正常:
`pgrep -P 2194 -l` → `3870 node`。

差别只有一个:在 Aegis 面板里跑时,pgrep 自己是 3870 的后代;
在别处的会话里跑时,它不是。

**结论:macOS 的 pgrep 把调用者的祖先进程排除在外,而且不报错、不警告。**

## 为什么会改变未来动作

- **AGENTS.md 第 2 条那条规矩只对"查席位"成立。** 席位不是 Aegis 的祖先,
  所以 `pgrep -P <席位 pane_pid>` 一直好使 —— 这也正是这条 bug 藏了这么久的原因。
  但**任何 Aegis 查自己的脚本都会静默拿到空结果**,而空结果的形状是
  「Aegis 没在跑」,不是「检查失败」。
- 由此产生的错误结论有两种,都是静默的:
  1. 健康检查报「Aegis 没在跑」—— 人在不在都被报成不在。
  2. 更糟:一个重启脚本认为「pane 里没有 agent,不用重启」,于是**跳过重启**,
     而磁盘上新版、内存里旧版的中间态一直留着。
- **正确的枚举方式**:`ps -axo pid,ppid,command | awk -v P=<pane_pid> '$2==P {print $1}'`。
  awk 按空白分词,前导空格不影响,`$2` 就是 ppid。这条在两个场景下都对。
- **配套的坑(同一次写脚本时踩到)**:`ps -o command= -p <pid>` 返回的名字**带尾随空格**
  (`pi      `)。所以 `case "$(ps -o command= -p $k)" in pi|node)` **匹配不上** ——
  必须 `tr -d '[:space:]'` 或者用通配 `pi*`。这个 bug 让第一次 dry-run 直接失败,
  而 dry-run 本身就是为抓它加的。
- 这条和 [[macos_ps_etimes_silently_yields_now]] 是同一家子:
  **macOS 的进程工具在拿不到值的时候倾向于安静地给个空/错值,而不是报错。**
  凡是"枚举进程"的检查,都要额外问一句:枚举不到,是真的没有,还是工具看不见?

## 证据

- Aegis 面板内:`pgrep -P 2194` → 无输出,`$?=1`
- 另一个 tmux 会话(`pgrep-probe`)内:`pgrep -P 2194 -l` → `3870 node`;
  `pgrep -x pi -l` → 3870 + 6 个席位
- `ps -axo pid,ppid,command | awk '$2==2194'` → `3870  2194 pi`(两个场景下都对)
- `ps -o command= -p 3870` → `pi      `(带尾随空格),`case ... in pi)` 不匹配
- `DRY_RUN=1 bash scripts/restart-aegis.sh` 第一次因上述空格 bug 失败,
  修完第二次全绿通过 —— dry-run 是为验证前置段而加的,它确实抓到了东西
