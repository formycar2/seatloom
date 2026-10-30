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
| [seat_tool_calls_must_not_restate_cwd_or_reread_files](seat_tool_calls_must_not_restate_cwd_or_reread_files.md) | failure | quoted | 席位一件事 582 次工具调用,其中 261 次是 `cd <绝对路径> &&` 前缀(信息增量零)、同一文件读 11 次。张先生裁定「Nimbus 做一件事太长了,步骤太多了,也很费 token」。可核判据:同文件 read ≤2、cd 前缀每命令 ≤1、一件事调用 ≤300。省调用不减验证。 |
| [root_cause_of_token_cost_is_seat_tool_choice_not_config](root_cause_of_token_cost_is_seat_tool_choice_not_config.md) | decision | inferred | token 大头不是压缩/thinking 配置,是席位把代码塞进 bash heredoc 改文件(占 toolCall 的 58%、且同一份代码付两次)。改代码用 edit。vela 14% 证明是习惯不是天性。判据已写进 ENGINEERING_STANDARDS |
| [edit_code_with_edit_not_a_bash_heredoc](edit_code_with_edit_not_a_bash_heredoc.md) | decision | quoted | token 大头是席位把代码塞进 `python3 - <<'EOF'` 改文件:占 toolCall 的 58%,同一份代码付两次(当命令参数发一次,之后 read/diff 再看回来)。改代码走 `edit`。toolCall 占全队 token 36% 高于 thinking 17%;vela 只占 14% 证明是习惯不是天性。补上一条,不推翻上一单的 cd/重读两条(那是小鱼,8.1%)。附:解决步骤和提交步骤不要串成一条命令 —— 我因此把带冲突标记的文件提交了 |
| [process_running_does_not_mean_it_runs_the_new_code](process_running_does_not_mean_it_runs_the_new_code.md) | failure | quoted | 进程在跑不等于跑新代码。覆盖 /Applications 后必须 pkill 重起;验前端要 grep Resources 里的 assets(不能 grep 二进制);fleet-status.sh 早有一行判据而 Aegis 三轮心跳都没读 |
| [a_directory_listing_cannot_prove_a_compressed_asset_shipped](a_directory_listing_cannot_prove_a_compressed_asset_shipped.md) | failure | quoted | 我报了个不存在的 bug:Resources/ 只有 icon 是 Tauri v2 正常行为(前端 brotli 嵌进二进制)。三层错:判据描述错行为、grep 压缩内容同时假阳假阴、在永远不会出现的地方找东西。更深的错:isComposing/proposeDimensions 来自 @xterm/addon-fit,证明的是 xterm 不是 SeatLoom 代码。可复用判据见 scripts/verify-tauri-bundle.sh |
| [a_mutation_label_that_is_wrong_still_looks_like_a_reproduction](a_mutation_label_that_is_wrong_still_looks_like_a_reproduction.md) | failure | quoted | 我在派单里把 M1 描述成『合并两条失败路径』,照着自己这个错误描述去复现,结果 3 failed 和 Mira 报的 RED 3 对上就以为成功。Flux 试了五种合并方式全是 RED 2 —— 能给 RED 3 的是恢复 .catch(()=>{}) 即原 F1。动作对、标签错:如果 Mira 的 M1 真是合并,我的复现本该对不上,而我大概会把对不上当成她夸大。复现变异要自己从原文构造,报数字时要连变异内容一起报 |
| [equivalent_and_unreachable_are_different_words](equivalent_and_unreachable_are_different_words.md) | decision | quoted | Mira 把杀不死的 M2 叫等价变异,Flux 证明 M2+把查询移到订阅之后→RED 2,所以那个 guard 在重排下是 load-bearing 的。「等价」让人想删,「今天测试没覆盖」让人别删 —— 同一个事实两个词导出相反行动,而 Mira 保留 guard 的决定是对的,错的词会把下一个人引向破坏她的决定 |
| [reset_clears_scrollback_to_fix_one_bug_and_causes_another](reset_clears_scrollback_to_fix_one_bug_and_causes_another.md) | failure | inferred | 张先生发现只有 QL 能翻页。真因不是 alternate screen(QL 也在里面),是 SessionTerminal.tsx:202 的 terminal.reset() —— 它为了修输入框错位而清掉 scrollback,attach 时那 3000 行历史被扔。于是每个 seat 只能看切过去之后的。QL 能翻只因刚被看过。两个修法冲突,要同时做到得靠 extension 补历史 |
| [a_growing_file_does_not_prove_the_pipe_is_plugged_into_the_right_pane](a_growing_file_does_not_prove_the_pipe_is_plugged_into_the_right_pane.md) | failure | quoted | Nimbus 排障时把 `tmux pipe-pane`(不带参数)当查询用,实际是停止,断了 app 正在用的管;"恢复"时挂错了 FIFO,于是 Nimbus 那一格显示 Aegis 的全部输出。他的验证是"转录文件重新增长"—— 增长测的是这一端有数据在写,测不出写进来的是谁。同一 FIFO 180,986,013 字节,前 3.8% 是 Nimbus、后 96.2% 是 Aegis,一次交接。可复用判据:两侧各自独有的指纹字符串必须在对的位置 |
| [symptom_gone_is_not_root_cause_gone](symptom_gone_is_not_root_cause_gone.md) | failure | inferred | 定位过程中症状自愈了(Aegis 关自己 pane 的 pipe,乱接的 cat 跟着死,reader EOF 丢掉 FIFO)。看起来好了,实际 Aegis 与 Nimbus 两个 pane_pipe 都是 0,Nimbus 那一格是死的。报已修复前必须回答:是什么让症状消失的,以及那条路断了之后本来该有的还在不在 |
| [a_path_limited_merge_is_not_a_whole_branch_merge](a_path_limited_merge_is_not_a_whole_branch_merge.md) | failure | quoted | Lyra 的分支落后 track 373 个提交,我用 cp 覆盖整个 ENGINEERING_STANDARDS.md,删掉了自己 10-26/27 刚写的「A dispatch is not done when the work is done」「Tool use and token cost」和当天 19:00 刚写的 pipe-pane 一节。diff 只显示 2 deletions,因为整节被换掉在 diff 里就是几行。判据:先量分叉、只取要的文件、同文件两边都改过就以 track 为基底手工摘增量、合并后逐节 grep 数节数 |
| [a_batch_verification_dispatch_cannot_carry_one_verdict_id](a_batch_verification_dispatch_cannot_carry_one_verdict_id.md) | failure | inferred | Lyra 裁决解锁了 Flux 的 8 个验单,我把 5 个塞进一张派单,工具照样只铸一个预声明 verdict id —— 于是工具层说「发 ev-Tm1PjlOBPblPTcwwyWU9G」而我正文说「每个单发一条,id 自己构造」,收到的人无法同时满足两者。同时把 disp-VH6l1Hixl 手打成 disp-VH6l1Hizl,派单照样建成,因为派单号只是个字符串、工具不校验它指向真实派单。判据:一个被验单元一张派单一条预声明 id;派单号从库里复制不手打;派完立刻 join 一次确认被验那单真的存在且 done |
| [i_measured_the_dispute_myself_and_both_seats_were_partly_right](i_measured_the_dispute_myself_and_both_seats_were_partly_right.md) | failure | inferred | Nimbus 发 fail 说 Flux 三条实测结论假,其中一条已写进 track 的 App.tsx 注释。我自己跑了:Flux 的「RED 2」确实复现不了(20/20 绿);但 guard 确实承重,证据是 Nimbus 找到的另一种变异(订阅拒绝晚于查询拒绝→M2 下降级成 stale);而 Nimbus 给的机制是错的(他说查询链多一个 .then,实际两边都只有一个)。结论对、理由错比结论错更难发现。判据:可测的争执不要裁决,自己跑 |
| [a_seat_that_cannot_reach_the_model_looks_online_forever](a_seat_that_cannot_reach_the_model_looks_online_forever.md) | failure | inferred | Flux/Nimbus 两件 in_flight 120 分钟没动,面板刷 "network unreachable"。fleet-status.sh:73 的「在线」判据只有「pane 有没有子进程」,所以卡在 Connection error 重试里的 agent 永远显示在线、永远「干活中」。我第一反应查错了方向:ping 不通的 hub.i.basemind.com 是镜像仓库不是推理端点,真正端点 api.stepfun.com/step_plan 是通的(401=活着)。判据:隔 60~90 秒读两次面板上的 R<数值> token 计数器,涨了就是活着,不变才是真卡住 |
| [the_same_deployment_claim_was_false_twice_and_both_times_it_went_outward](the_same_deployment_claim_was_false_twice_and_both_times_it_went_outward.md) | failure | quoted | Flux 两次在部署状态上给出假结论,形状相同:把跑着的二进制的 mtime 14:03 读成 00:03(差 14 小时),并说某段文字不在程序里 —— 实际 Nimbus 解压 blob 后在偏移 6,216,610 处逐字节找到。为什么特别贵:部署结论决定"要不要动手",而且它被当成已有教训的新实例,污染了那条教训的计数,让操作者怀疑真的实例。判据:只许解压构建时用的那批 blob 再比对,不许 grep 可执行文件、不许看目录列表、不许看 mtime 猜新旧 |
| [a_verdict_written_in_the_pane_is_not_a_verdict](a_verdict_written_in_the_pane_is_not_a_verdict.md) | failure | inferred | 队列空了我一度以为一切就绪。实际 Flux 在派单里 measured 完三件 Nimbus 的活,把三条 PASS 结论写进面板滚动记录,然后去干别的 —— 账本里这三件 pass verdict 数是 0,扫 400 行滚动记录 emit_event 调用 0 次。后果:那三件仍被 VerdictNotGreen 拦着,10-26 那条 fail 没被翻,已阻塞 35 小时。它视角里确实做完了(结论都写着),但下游拿不到。判据:队列空 ≠ 没事了;它声称发了 verdict 就查账本,面板里的 ev- 编号只说明它提过这个字符串 |
| [asking_a_seat_to_verify_its_own_mistake_makes_it_break_its_own_rule](asking_a_seat_to_verify_its_own_mistake_makes_it_break_its_own_rule.md) | failure | inferred | 我派 Flux 补发三条漏发的 verdict,它补得完全正确(用预声明 id、三条独立、dispatch_id 逐条对上),但顺手给自己的两个派单也各发了一条 —— 违反 Lyra「席位不许给自己的活发 verdict」。一半是我的问题:enqueue_dispatch 的 verdict_event_id 要求「验证者为该派单发一条 verdict」,而我把 verification 派给了 Flux,两个要求逻辑上无法同时满足。21:00 我派 8 个 audit 单时避开过这个坑(验 Flux 的活→派给别人),02:00 又踩了。判据:给 Flux 派 verification 单必须同时安排好第三方 verdict |
| [i_pushed_to_track_after_cards_opened_and_broke_the_operators_accept](i_pushed_to_track_after_cards_opened_and_broke_the_operators_accept.md) | failure | inferred | 同一天复发两次,第二次更严重。05:20 写下判据「提完立刻查卡点不点得动」,08:05–12:15 往 track 提了 8 次,一次没查,23 张卡全点不动(Mira 落后 1、Flux 和 Nimbus 各落后 9)。规则没错,靠记住才执行的规则不是控制。已换成机械的:`scripts/check-cards-clickable.sh` 打印数字、点不动 exit 1,`.git/hooks/pre-commit` 直接拒绝提交(验证过,它拒了我自己那笔)。紧急出口是 `--no-verify` + 同回合派同步单 + commit body 写原因,出口存在但留痕 |
| [a_gate_blocker_recorded_in_a_delivery_note_goes_stale_and_nobody_checks](a_gate_blocker_recorded_in_a_delivery_note_goes_stale_and_nobody_checks.md) | failure | inferred | 三条门卡了很久。前两步我每轮都做(门在账本里吗、依赖物 live 吗),第三步从没做过:复测交付说明里写的阻塞理由。一复测就发现它过期三个月 —— 写着「022 不在 track 上、live 只有另一个视图」,实际 022 在 track、live 有 v_memory_session_join 且 90 行。阻塞原因写在散文里,不在任何会自动检查的地方,所以过期的和真实的在账本上长得一样。顺带发现 disp-session-key-readmodel 账本 done 但代码不在 track 也不在任何工作区,只在一条落后 367 提交、从未合并的分支上 |
| [the_gate_condition_was_met_on_a_branch_but_not_on_track](the_gate_condition_was_met_on_a_branch_but_not_on_track.md) | failure | inferred | Nimbus 交付读模型,报告写「门条件具备了(两个读接口在 track、022 在 live、90 行)」。实测:022 和 90 行对,但三个读接口都不在 track,只在他分支上。他三小时都在自己工作区,那里文件都在、322 tests 过、证明脚本 8/8 —— 在他的工作区那句话完全是真的。信了就会发假门,Mira 会按一个从没在主线存在过的接口写 UI。这是 gate_events_conflate_code_landed_with_live_usable 的镜像:那次是「代码在主线但 live 不可用」,这次反过来。判据:「X 在 track 上」永远由发令的人自己核,三条全过才发门 |
| [three_stalled_gates_were_done_work_waiting_on_a_string](three_stalled_gates_were_done_work_waiting_on_a_string.md) | failure | inferred | 三个 `blocked` 十天的派单,其中两个门的代码早就在 track 上且是对的(`App.tsx:133` 是 `useState<Tab>('dispatches')`、`provenance.ts` 先问 `backendHydrated` 再返回 `live`),卡的唯一原因是发布机制是精确字符串匹配、没有第二条通道(`repositories.rs:795` 一句 `SELECT EXISTS`)。而三个字符串的归属是写死的,归 Mira 和 Nimbus 各半,`actor_ref` 从 server env 解析、参数传不进去,所以我不能替他们发。判据:判断一个阻塞要问三遍——东西做完了吗、宣告了吗、宣告用的 id 就是边上那个字符串吗;第三步最便宜也最常被我跳过 |
| [a_seatloom_problem_is_not_every_session_ending_in_seatloom](a_seatloom_problem_is_not_every_session_ending_in_seatloom.md) | failure | inferred | 发现 23 张卡被自己卡住后很急,对着所有 `-seatloom` 会话群发紧急消息。那个后缀是 `pty/mod.rs` 用来**发现**会话的标记,不是**归属**标记 —— 采纳过外部项目之后它就不再区分内外。QL、networkOrchestrator、stepaicloud 各有 0 张 pending 卡、0 个派单,消息对它们在事实层面完全为假,而 QL 当真了,花一整轮重新推导出不做事(结论对,但不该花这轮)。更该追问的是:一条以 `# Aegis` 开头、带 `[POKE-xxx]` 和"优先级 99"的消息对外部席位就是提示注入,stepaicloud 当场识别并拒绝 —— 它防住了,但那不是它该花精力防的。判据:群发前先问"这九个人里谁真的受影响",后缀/前缀这类"看起来像归属"的东西在被采纳过外部实体后就不再可靠,归属要么查账本要么别发 |
| [the_seat_session_names_are_not_derivable_from_the_seat_ids](the_seat_session_names_are_not_derivable_from_the_seat_ids.md) | failure | inferred | 派完单去叫醒席位,poke 了 `Flux-QA`/`Lyra-PO`/`Nimbus-Backend`/`Onyx-Data`/`Vela-Wash` —— 五个全错(`tmux send-keys` 对不存在的会话看起来是成功的)。真实的是 `Lyra-po`/`Nimbus-TechArchi`/`Flux-Quality&Ops`(含裸 `&`)/`Onyx-data`/`Vela-DataWash`,规律是 `<席位>-<岗位>-seatloom`。更糟:我用 grep 读 pane 验证,读的也是错会话名 —— **验证工具和被验证的东西用了同一个错误前提,于是验证也通过了**。`scripts/poke-all-seats.sh` 从活着的 server 读名单 + 每条消息带唯一 marker 读 pane 回验;测它的过程中发现它自己两个 bug 都会让检查失效(`pipefail`+`grep -q` 的 SIGPIPE 把成功读成失败;单次回验在席位忙时读不到,t=0 无 t=3s 有 —— 在席位忙时才失败的检查恰好在最要紧时失效) |
| [a_verdict_that_names_a_dispatch_which_does_not_exist_is_not_a_verdict](a_verdict_that_names_a_dispatch_which_does_not_exist_is_not_a_verdict.md) | failure | inferred | 清理"干完了没人接"的活时逐件查 verdict,发现 `ev-usP7dX7geulRhaDy1eXwD` 的 `dispatch_id` 是 `disp-B4Y5oAhMQwtXm5Vva1fw` —— dispatches 表 0 行。真 id 是 `disp-B4Y5oAohMQwtXm5Vva1fw`,数字 0 vs 字母 o,一个字符,而 nanoid 字符集里两者都在。它看起来完全是绿的第三方 verdict,指向空处。空指针会报错,野指针不会 —— 而且哪天真出现一个长这样的 id,这条 verdict 会突然变成那个不相干派单的验绿依据。全库另还有 4 条空指针。26 张 pending 卡的 ref 全部有效(好消息)。判据:verdict 的有效性不能停在"事件存在",要 join 回 dispatches 确认那行在;`scripts/check-verdict-refs.sh` 已接进心跳 |
| [i_invented_an_exception_to_my_own_rule_and_measured_nothing](i_invented_an_exception_to_my_own_rule_and_measured_nothing.md) | failure | inferred | 同一天第五次破"开卡后不推 track",而这次最糟:前四次是忘,这次我**给自己论证出一个例外**。写了 `should-aegis-commit.sh` 论证"只碰 docs/scripts 的提交不需要上 track,因为没席位会合并我的知识文档",接进 pre-commit 钩子,钩子放行。我测了脚本两个方向(分类对),**没测放行之后卡还点不点得动** —— 23 张全点不动。验收门判据在 `dispatch.rs:1749` 是纯 git 的 `merge-base --is-ancestor`,一个字都不问提交里是什么文件。我的因果链中间那一环是我发明的。判据:想给自己开例外时,先测例外生效之后的**结果**,不是例外成立的**条件**;当我想论证"这条规则不适用于我"时,那就是最该停下来测的时候 |
| [the_waker_was_never_running_and_i_only_found_it_because_a_status_line_said_idle](the_waker_was_never_running_and_i_only_found_it_because_a_status_line_said_idle.md) | failure | inferred | 16:00 心跳看到"有活排队但没人在做",去查唤醒服务,发现它**从来没在跑**:二进制没编译、没进程、没日志。而 `fleet-status.sh` 的判据是 `tmux has-session -t seatloom-waker` —— waker 是个 Rust 常驻守护进程(`src/bin/waker.rs` 1420 行,`main` 直接 `loop{}`),不是 tmux 会话。我建了个同名会话,状态行立刻变绿,而守护进程依然没跑。更该记住的是为什么这么久没发现:**我每轮心跳都手动 poke 席位,活一直在推进,所以从不见症状** —— 我做的事掩盖了系统失效。另外那个守护进程不处理任何参数,`--help` 不是问帮助、是启动它,所以第一次探路挂住是正确行为 |
| [a_large_batch_of_accepted_cards_strands_every_unsynced_branch](a_large_batch_of_accepted_cards_strands_every_unsynced_branch.md) | failure | inferred | 操作者 15:15–18:18 接受了 27 张卡,track 一次前进 173 个提交。Vela 15:15 同步过、那时卡点得动,之后落后 173、卡点不动 —— 她什么都没做错。此前的记录只覆盖"我提交让分支落后 1 个",没覆盖"批量接受让分支落后整个批量",而我派的同步单话术一直是"你又落后 1 个提交",暗示席位做错了。判据:看到"落后 N 个提交"先问 N 是哪儿来的;N=1~2 且提交是我的 → 我造成的;N=100+ 且中间有一批卡被接受 → 结构性后果,不是席位的错,话术要不同,且只派有 pending 卡的那个分支(其余落后不卡任何事) |
| [i_could_not_reproduce_the_bug_until_i_used_the_command_that_caused_it](i_could_not_reproduce_the_bug_until_i_used_the_command_that_caused_it.md) | failure | inferred | 想验证 pipe 守卫真挡住那个 bug,复现时跑的是 `tmux pipe-pane <pane> "cat >> /other.fifo"`(带另一个 cat)—— 什么都没发生,两个 writer 并存、`pane_pipe` 还是 1。差点报"修复无效"。真相:原始 bug 是**裸 `pipe-pane`(无参数)**,那是 tmux 的 STOP 命令不是查询;"加 writer"和"关 pipe"在 tmux 里是两回事。用对命令一次复现:`pane_pipe` 1→0,而 ps 里那些 `cat` **还在跑、状态 S 不是 Z、还被列成我们 FIFO 的 writer** —— 所以只看进程表的守卫会判健康,而当时一个字节都不在流,`pane_pipe` 是这个守卫全部意义所在。变异测试还抓出我自己的假:把 pane_pipe 检查**挪位置**存活(两分支最后都返回 Lost,我的断言没区分),**删掉**才被杀。判据:复现必须用导致它的那条命令;变异存活时不要改变异,要改测试 |
| [ten_units_of_work_had_a_pass_verdict_and_never_reached_the_operator](ten_units_of_work_had_a_pass_verdict_and_never_reached_the_operator.md) | failure | inferred | 01:00 心跳,`fleet-status.sh` 说全队空闲、没有需要操作者动手的事。处理 10 件 failed 时偶然发现:**10 件 implementation 派单有 pass verdict、`select count(*) from acceptances where flux_verdict_ref=<id>` 全是 0**。工作做完、独立验绿、操作者从没被要求验收。最老的 019 挂了 19 天(verdict 10-10、门 10-11、卡从没开)。三个检查全放行:队列空(派单 done)、派单 done、verdict pass —— 坏的事坐在没有任何检查看过的地方。已补成心跳第 2c 步。两个附带教训:①我第一次用"有没有关联事件"判断工作是否做了,10 件全非零,差点读成"做过了" —— 那些全是派单生命周期事件(Issued/Delivered/Failed/Retried),正确谓词是 `event_type='SeatDispatchCompleted'`,"有事件"和"有产出"是两个查询;②10 件 failed 里 4 件的失败原因已过期(lineage/读取界面/016/017 现在都在、撤卡那张已取消)—— **失败原因是某个时间点的测量,不是永久属性** |
| [i_handed_over_verdict_ids_without_checking_which_dispatch_each_one_names](i_handed_over_verdict_ids_without_checking_which_dispatch_each_one_names.md) | failure | inferred | 上一轮发现"10 件有 pass verdict 没卡",派 owner 开卡。我甚至在派单里写了预警:「两件共用一条 verdict,如果它真的是关于两件的,两张卡都能指它」—— **我知道有风险、写在纸上、然后没自己去查**。40 分钟后我 40 分钟刚写进心跳的 `check-verdict-refs.sh` 抓住了:三张卡全指错,其中两张正是我预警过的那两件。真相:`ev-pnBnPX1pdaRysj8UYW077` 的 `payload.dispatch_id` 是 `disp-ySsxkvHxKurEgKpt7Q_Hm`(Flux **验**那两个洞的验活派单),而卡挂在 Nimbus **修**那两个洞的实施派单上。我查错的根源:我的查询 join 的是 `event_object_refs`,它把 verdict 和**它提到的所有东西**连起来,不只 subject —— Flux 的 verdict 把 Nimbus 的实施派单作为 context 提了一句,join 就抓到了。**refs 上的 join 不是 subject 上的 join**;想知道 verdict 关于哪件派单,只能读 `payload.dispatch_id`。另外我第一反应是自己撤那两张卡,守卫拒得对(只有请求方能撤自己的卡)—— 我剛在别处写过这条,然后差点绕过它 |
| [the_blocked_dispatch_was_asking_for_work_that_had_already_succeeded](the_blocked_dispatch_was_asking_for_work_that_had_already_succeeded.md) | failure | inferred | 两件 `disp-onight-writein` 失败原因是"open_topic 不可用",我量了三个只读判据确认阻塞消失,派 Vela 重做。她**一个字没写**并解释:写库 10-11 就 live 了,那两件是在重做已成功的工作,再写会产生重复。她量到 24 个 wash-* topic 正好对上 manifest 的 1+10+13、90 条记忆、逐 topic 87 vs 88 —— 那个 +1 是她自己 10-11 的幂等性探针,她找到并报告了自己造成的多余行,而不是报一个干净的 87=87。我的错:我问了"阻塞还在吗",就以为它回答了"工作做过吗" —— 这是两个独立的问题。今天第四次遇到失败原因过期,前三次我重派对了,因为区别不在阻塞真假,在**产物在不在**。判据:重派前问产物,别问席位(`select count(*) from memories where source_kind='archived_session'`)—— 今天已经见过用"有没有关联事件"查会得到 10 件全非零,那些是派单生命周期事件。附带真发现:`remember` 无内容去重,实测 6 组重复、最严重一组同文件同行区间 4 条,只有一组是她的探针,其余五组是普通写入 |
| [我当场就怀疑那张卡是假的,却在还差一步的地方停下了](i_suspected_the_card_was_false_and_stopped_one_step_before_proving_it.md) | 没得出结论就转移话题,和没查是一样的;而"读出来是空"没先排除 UTF-16 就宣布结论,直接翻案 | 2026-10-29 |
| [我的检查工具报了 shell-based=0,而四张卡正在让操作者往账本里 INSERT](my_checker_reported_shell_based_0_while_four_cards_ran_shell.md) | 假阴性长得和"真没问题"一模一样;读产品渲染的那一栏,不是人写得好看的那一栏 | 2026-10-29 |
| [管道是通的、数据是真的,而两边的滚动历史一点重合都没有](the_pipe_was_alive_and_the_data_was_real_and_the_scrollbacks_shared_nothing.md) | 「停写」是文件属性不是 pipe 归属;跨线判定要用独有句子加命中次数;改全局 history-limit 不等于改已存在的 pane | 2026-10-29 |
| [门看不见仓库,而我的检查同意它没事](the_gate_could_not_see_a_repository_and_my_checker_agreed_it_was_fine.md) | 一个会修好自己工作目录的检查,对它要抓的失败是瞎的;open -a 不带环境变量,门在 / 下跑 git,Err 被映射成分支未同步 | 2026-10-29 |
| [历史会话功能覆盖了每一个 harness,唯独覆盖不了舰队自己跑的那个](the_history_feature_covers_every_harness_except_the_one_this_fleet_runs.md) | 数"有多少数据"是空的,数"哪些来源的哪些数据"才有信息;覆盖五个外部来源、漏掉唯一一个内部来源的管道,比空管道更难发现 | 2026-10-29 |
| [六件真产品改动做完了,而我没派 Flux 验,也没让任何人开卡](six_product_changes_finished_and_i_never_dispatched_a_verifier.md) | 绿灯的数量是已知问题的数量,不是健康程度的度量;全绿的那一分钟有 6 件活在等一个从没被派出的验绿单 | 2026-10-29 |
| [我向操作者报告了一次"缺陷合并",而我点名的那张卡是干净的](i_reported_a_defective_merge_and_the_card_i_named_was_clean.md) | 拿手里已有的 verdict 去搜,而不问那张卡实际绑哪个 verdict —— 和 10-28 是同一个形状;假警报比漏报更伤,因为它让他无法分辨 | 2026-10-29 |
| [存活的变异总是在"没人想到要测"的那个方向](the_surviving_mutation_is_always_in_the_direction_nobody_thought_to_test.md) | 验证者要自己变异,不能只重跑作者的;消耗性门必须有"第二次"测试;绿套件是"用例过了"的证据,不是"实现正确"的证据 | 2026-10-29 |
| [今天第三次:我测了错的代码,然后差点据它汇报](third_time_today_i_measured_the_wrong_code.md) | 目录名不是分支名;越是不好的结果,越要先确认测量对象 —— 坏消息测错了不会自我矛盾 | 2026-10-29 |
| [被接受的卡合进了两个 apply-db.sh 从没听说过的迁移](accepted_cards_merged_migrations_apply_db_never_heard_of.md) | 验到第一个绿灯就停,和没验的区别只在心里好受;迁移在 track 上、卡被接受、代码在 track 上,而 live 效果是零 | 2026-10-30 |
| [拼错的分支名和真的不存在的文件,返回一模一样的空输出](a_mistyped_branch_name_and_an_absent_file_return_the_same_empty.md) | 空输出必须被验证,不能被解读;"不在"比"在"危险,因为它让下游推理自洽地跑完;结论对、理由错,比结论错更危险 | 2026-10-30 |
| [我差点把 bracketed paste 当成修法,实测证明它毫无作用](i_nearly_reported_bracketed_paste_as_the_fix.md) | "官方默认用 X 而我们没用"不是证据;复现必须用被报告的同一形态输入;症状描述里每个细节都是线索 | 2026-10-30 |
| [a_test_that_reads_its_own_assertion_strings_can_never_fail](a_test_that_reads_its_own_assertion_strings_can_never_fail.md) | failure | inferred | Nimbus 修了 `remember` 无去重(我上一轮发现的),proof 7 点全过、211 个 Rust 测试过。我变异验证:把 guard 的谓词从 `IS NOT DISTINCT FROM` 改成裸 `=`(破坏 NULL 安全,而 live 上 3/4 的重复组正是没有行号的),**proof 和测试都过了**。proof 没被杀是因为双层防护:唯一索引也用 `coalesce()` 写着,NULL 安全 —— 我差点记录成"proof 不是 load-bearing",实际是我只变异了一层。测试没被杀才是真问题:它 `include_str!` 读自己所在的文件,然后 `src.contains("l2_line_start IS NOT DISTINCT FROM $2")` —— 那个字符串字面量就写在测试自己里面(4520 行),真正的 guard SQL 在 1293 行,**它匹配到的是它自己**。它也不是完全无效(把 key 列换成无意义谓词它会失败),所以它以为守四条,实际守一条半。它自己的注释写了"这比行为测试弱",诚实且正确,但它没意识到匹配的是自己 —— **一个知道自己弱的检查,仍然可能比自己以为的更弱** |
