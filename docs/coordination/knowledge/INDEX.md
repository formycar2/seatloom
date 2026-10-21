# SeatLoom 运行知识库

Aegis 的记忆落在这里,而不是 Aegis 自己那个 agent memory 目录。

**为什么搬:** 那个目录在张先生的 Mac 上,每次写入都要他 approve。
于是他出门或睡觉时 Aegis 就记不了东西 —— 而「记不了」恰好发生在最该记的时候(出事的时候)。
放在仓库里,Aegis 自己就能维护。

**为什么是这个形状:** frontmatter 的字段**对齐 `memories` 表的列**。
等 `topics` 能创建、写入路径打通之后,洗数据流水线可以直接把这些文件灌进库,
不需要再转换一次。张先生定过:能一步到位就别做一次性的过渡产物。

## 字段

| 字段 | 含义 |
|---|---|
| `memory_id` | 唯一标识,也是文件名 |
| `project_id` / `topic_id` | 作用域。project + topic 是唯一的作用域框架 |
| `kind` | `decision` / `failure` / `fact` / `preference` / `artifact` / `unresolved` |
| `confidence` | **二元,不是分数**。`quoted` = 原文明说;`inferred` = 推断得出 |
| `status` | `active` / `forgotten` / `resolved` |
| `tier` | `cold` / `warm` / `hot` —— 生命周期 |
| `l2_file_path` + `l2_line_start/end` | **行级证据锚点**。指不到具体行号的记忆不算有证据 |
| `l0_summary` | 约 100 token,召回时默认只给这一层 |
| `curation` | `authored` = 人/Agent 真判断过;`mechanical` = 按规则映射的,**未精炼** |

`curation` 这个字段不在 `memories` 表里,是这里额外加的。
理由:机械映射的 `kind` 和 `confidence` 不能和真判断混在一起 ——
把映射结果当成判断,以后没人分得清哪条可信。灌库前该有一次精炼,这个字段就是精炼的待办清单。

## 现状

**已迁入:14 条**(Aegis 新写的)。

| 记忆 | 类型 | 置信度 | 一句话 |
|---|---|---|---|
| [pi_default_model_needs_default_provider_too](pi_default_model_needs_default_provider_too.md) | fact | inferred | 同名模型跨 provider 存在时,只设 `defaultModel` 会被 pi 自己挑一个;必须同时设 `defaultProvider`。这也可能是上午全队静默换模型的根因 |
| [gate_history_sessions_read_live_stays_closed](gate_history_sessions_read_live_stays_closed.md) | decision | quoted | 门事件 ev-history-sessions-read-live 保持关闭:Flux 写死的 (a)(b)(c) 一条都没满足,live 库连 message_count 列都没有 |
| [pi_upgrade_changes_disk_not_running_processes](pi_upgrade_changes_disk_not_running_processes.md) | fact | inferred | `pi update` 只改磁盘,跑着的进程全是旧版;而重启脚本比的是 MCP 程序不是 pi,永远不为 pi 升级重启席位 |
| [macos_pgrep_cannot_see_the_callers_ancestors](macos_pgrep_cannot_see_the_callers_ancestors.md) | failure | inferred | macOS 的 pgrep 静默看不见调用者的祖先进程,查 Aegis 自己必然拿到空结果,而空结果的形状是「没在跑」 |
| [aegis_never_launder_permissions_through_seats](aegis_never_launder_permissions_through_seats.md) | failure | quoted | 让席位去做 Aegis 自己被拦住的事,是绕过权限而不是解决问题 |
| [waker_idle_agent_looks_identical_to_a_working_one](waker_idle_agent_looks_identical_to_a_working_one.md) | failure | inferred | 「pane 里有 pi 活着」不是进度信号,全队因此停摆 17 小时 |
| [gate_events_conflate_code_landed_with_live_usable](gate_events_conflate_code_landed_with_live_usable.md) | failure | inferred | 席位的交付公告被当成能力可用的门;事件没说谎,说谎的是那条依赖边 |
| [aegis_waker_must_live_outside_the_session](aegis_waker_must_live_outside_the_session.md) | failure | inferred | 自醒必须建在会话之外;会话内定时任务死过三次,pi 连这个工具都没有 |
| [macos_ps_etimes_silently_yields_now](macos_ps_etimes_silently_yields_now.md) | failure | inferred | macOS ps 无 etimes 且关键字清单走 stdout,awk 静默按 0 算,时间全变成"现在" |
| [waker_idle_agent_looks_identical_to_a_working_one](waker_idle_agent_looks_identical_to_a_working_one.md) | failure | inferred | 「pane 里有 pi 活着」不是进度信号,全队因此停摆 17 小时 |
| [git_merge_ab_form_merged_unverified_work_into_track](git_merge_ab_form_merged_unverified_work_into_track.md) | failure | inferred | `git merge A B` 是八爪鱼合并,把未验的活合进当前分支;造集成分支要先 switch 过去,合并后第一件事核主分支头 |
| [saih_continue_probe_contaminates_newest_session](saih_continue_probe_contaminates_newest_session.md) | failure | inferred | `--continue` 会恢复最新会话并往里写;用驱动会话探测它,会把陌生问答追加进别人正在用的会话文件 |
| [agent_handoff_docs_can_become_recursion_seeds](agent_handoff_docs_can_become_recursion_seeds.md) | failure | inferred | 交接文档里一行"启动下一个 agent 继续这工作"的命令就是递归种子;只写纯文字指令,附永久警告块 |
| [git_hooks_live_at_core_hookspath_not_dot_git](git_hooks_live_at_core_hookspath_not_dot_git.md) | fact | inferred | husky 的钩子在 core.hooksPath 指的地方,不在 .git/hooks;只 ls .git/hooks 是假阴性

**待迁入:35 条** —— 仍在 Aegis 那个 agent memory 目录里。
批量迁移需要张先生跑一条命令(读他 Mac 上的目录 + 批量写仓库,Aegis 被权限拦住):

```
python3 scripts/migrate-agent-memory.py
```

脚本已写好并演练通过:35 条可迁(18 条 preference + 17 条 fact;3 条 quoted + 32 条 inferred),
**无任何文件因疑似密钥被跳过**。加 `--dry-run` 可先看不写。

> 置信度里只有 3 条 `quoted` 看着偏低,这是**刻意的**。判据只认「」、"原话"、"verbatim"
> 这类真正指向他原话的标记。ASCII 双引号出现在 31 个文件里,但那是代码、路径、各种引用都在用 ——
> 拿它当判据会把 Aegis 的推断标成张先生的原话,正好是 `confidence` 这个字段存在要防的事。
| [v5_honesty_ranks_first_because_it_makes_other_values_trustworthy](v5_honesty_ranks_first_because_it_makes_other_values_trustworthy.md) | preference | quoted | 界面显示不存在的数字比缺功能更糟 —— 用户因此无法分辨哪页是真的;判据:每个数字都答得出「数谁、什么范围、何时截止、从哪来」,答不出就写暂无 |
| [ui_cannot_launch_app_yet_and_two_holes_must_not_stack](ui_cannot_launch_app_yet_and_two_holes_must_not_stack.md) | decision | inferred | 应用当前零进程启动能力;要加只能加固定用途(打开沙箱 app)且路径由后端认定。同时 DocumentsWorkspace.tsx:359 的 marked 输出未净化是已存在的洞,与启动权限叠加即可执行任意程序,须单独修 |
| [a_seat_branch_must_not_be_verified_until_it_is_synced_to_track](a_seat_branch_must_not_be_verified_until_it_is_synced_to_track.md) | failure | inferred | 分支落后主干 19 个提交时,沙箱里项目过滤消失,操作者报「功能坏了」—— 实为版本旧。派单/装沙箱只查「有没有那件事的提交」不够,还要查缺不缺别的 |
| [root_cause_of_token_cost_is_seat_tool_choice_not_config](root_cause_of_token_cost_is_seat_tool_choice_not_config.md) | decision | inferred | token 大头不是压缩/thinking 配置,是席位把代码塞进 bash heredoc 改文件(占 toolCall 的 58%、且同一份代码付两次)。改代码用 edit。vela 14% 证明是习惯不是天性。判据已写进 ENGINEERING_STANDARDS |
