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
| [i_pushed_to_track_after_cards_opened_and_broke_the_operators_accept](i_pushed_to_track_after_cards_opened_and_broke_the_operators_accept.md) | failure | inferred | 「队列空 + 23 张卡 pending」看起来最健康,实际最堵:三个席位分支全部落后 track 2 个提交,而那两个提交是我在开卡之后提的(补回漏掉的测试 + 记录复发)。验收门拒绝未同步分支,所以操作者点任何一张都收到拒绝,且他不知道为什么。讽刺的是被卡住的卡里包括修「他的话显示进别人那一格」那张。判据:往 track 提任何提交后必须立刻查每个有 pending 卡的席位是否还包含 track HEAD,不同步就在同一回合派单,不等下一轮心跳 |
| [a_gate_blocker_recorded_in_a_delivery_note_goes_stale_and_nobody_checks](a_gate_blocker_recorded_in_a_delivery_note_goes_stale_and_nobody_checks.md) | failure | inferred | 三条门卡了很久。前两步我每轮都做(门在账本里吗、依赖物 live 吗),第三步从没做过:复测交付说明里写的阻塞理由。一复测就发现它过期三个月 —— 写着「022 不在 track 上、live 只有另一个视图」,实际 022 在 track、live 有 v_memory_session_join 且 90 行。阻塞原因写在散文里,不在任何会自动检查的地方,所以过期的和真实的在账本上长得一样。顺带发现 disp-session-key-readmodel 账本 done 但代码不在 track 也不在任何工作区,只在一条落后 367 提交、从未合并的分支上 |
| [the_gate_condition_was_met_on_a_branch_but_not_on_track](the_gate_condition_was_met_on_a_branch_but_not_on_track.md) | failure | inferred | Nimbus 交付读模型,报告写「门条件具备了(两个读接口在 track、022 在 live、90 行)」。实测:022 和 90 行对,但三个读接口都不在 track,只在他分支上。他三小时都在自己工作区,那里文件都在、322 tests 过、证明脚本 8/8 —— 在他的工作区那句话完全是真的。信了就会发假门,Mira 会按一个从没在主线存在过的接口写 UI。这是 gate_events_conflate_code_landed_with_live_usable 的镜像:那次是「代码在主线但 live 不可用」,这次反过来。判据:「X 在 track 上」永远由发令的人自己核,三条全过才发门 |
