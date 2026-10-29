# Aegis 自醒心跳协议

> **切换日的待命条件,已于 2026-10-12 01:50 解除。**
>
> 起因:从 Claude Code 切到 pi 时,旧的 Aegis 会话可能还在账本上动作,
> 两个 Aegis 同时写会互相干扰,所以那段时间每一轮只读。
>
> **已核实解除**:Aegis-Supervisor 那个面板的 shell 从 10-10 起一直在,
> pi 于 10-12 01:21 才在其中启动 —— 旧会话在切换前就已经退出。
> 机器上确实还有 8 个 `claude` 进程,但它们的工作目录全是**别的项目**
> (cn-gpu-infer-fabric、ai-assistant、sf-code、fde…),没有一个是 seatloom 的。
>
> 🔴 **这个判据曾经写错过一次,别再写错**:最早写的是
> 「`pgrep -fl claude` 还有输出 = 旧会话活着 = 只读轮」。**这是错的** ——
> 那些进程属于张先生别的项目,与 Aegis 无关。照那条执行,每一轮心跳都会
> 误判自己还在待命,于是永远只读 —— 那正是「项目不能中断」的反面,
> 而且是一个看起来完全正常的堵点。
>
> **以后要判断「有没有另一个 Aegis 在动」,唯一可靠的判据是**:
> Aegis-Supervisor 面板里跑着的是不是 pi(我),以及
> 有没有别的进程以 seat-aegis-001 身份在写账本。
> 看进程的工作目录,不要看进程名。

这是 `scripts/aegis-self-wake.sh`(系统 crontab,每小时 :00)叫醒 Aegis 时发给它的指令内容。
(2026-10-20 由每 30 分钟一次改为每小时一次,张先生的要求。间隔写在两处:
`scripts/install-aegis-self-wake.sh` 的 CRON_MINUTE 与 `scripts/fleet-status.sh` 的
判死阈值 75 分钟。只改一处会让健康检查误报「自醒死了」。)
改心跳行为改这个文件,不用改脚本、不用重装 crontab。

## 它为什么存在

每个席位都有 `seatloom-waker` 叫醒,**Aegis 没有** —— 唤醒服务的日志明说
`NOTIFY for unmapped seat seat-aegis-001 — ignoring`。于是任何经过 Aegis 的环节
(门事件、跨席位裁决、迁移落库)都会卡到张先生出现为止。2026-10-10 与 2026-10-11
三次停摆全是这一条。

旧 Aegis 用的是会话内定时任务(`CronCreate`),它随会话死掉;切到 pi 之后这个工具
根本不存在。所以自醒改成了系统 crontab + `tmux send-keys`,和 `seatloom-waker`
叫醒别的席位是同一条通道。

**验证自醒活着的方式**:看 `/tmp/seatloom_aegis_waker.log` 有没有按 :00 长出
`POKED` 行。没有行就是自醒死了 —— 那是比任何单点故障都严重的事,先修它。

## 每一轮按顺序做

每一步都必须**能报错**。`... && echo OK || echo OK` 这种永远成功的检查比没有检查更有害。

1. **看全局**:`bash scripts/fleet-status.sh`
   - 队列:有没有派给 `seat-aegis-001` 的 `ready` 单
   - 席位存活:查 pane 子进程(`pgrep -P $(tmux list-panes -t <会话> -F '#{pane_pid}')`),
     **不要裸 `ps | grep pi`** —— `~/.local/bin/pi` 是 22 字节符号链接,裸 grep 抓不到,
     会误报"全队宕机"。而且 pane 里可能跑着完全不同的 agent(重启后 Onyx 的面板
     被 `gemini --resume` 占过),别假设"席位会话"就等于"pi"。
   - 领单年龄:用**数据库自己的时钟**算 ——
     `round(extract(epoch from (now()-started_at))/60)`,减法在库内完成。
     别拿库的时间戳去跟宿主机 `date` 比(podman 虚拟机时钟会在宿主机休眠后停摆,
     实测慢过 137 分钟,那样算出来的"挂了 10 小时"是假的)。
   - 进程活着是存活信号,**不是进度信号**。账本最后一条事件的时间比面板可靠:
     `canonical_events` 空白 17 小时而面板显示"6/6 在线"真的发生过。
   - ⚠️ `pgrep -P` 在 macOS 上会**漏子进程**,不漏的时候才准。2026-10-12 实测:
     `pgrep -P 2193`(tmux)列出的 pane 进程里没有 2194,而 `ps -o ppid=` 明确显示
     66508 的父进程就是 2194。所以:
     - 空结果**不等于**没有子进程,只能说"pgrep 没看到"
     - 要确认某个 pid 的父进程,用 `ps -o pid=,ppid= -ax | awk '$2==<pid>'`
     - 我的 `scripts/fleet-process-census.sh` 在 pgrep 返回空时报 ❌ 而不是静默跳过,
       就是为了这个 —— 宁可吵一点,不要悄悄看漏
   - **卡的点得动性**:`bash scripts/check-cards-clickable.sh`
     它查每张 pending 卡的分支是否还包含它的 base。
     `fleet-status.sh` 原来只查 base 分支**存在**,于是 23 张全点不动的卡被打成
     "✅ 可以点" —— 张先生点任何一张都收到拒绝,而他不知道为什么。
     **卡开出来了不等于卡点得动。**
   - **verdict ref 的有效性**:`bash scripts/check-verdict-refs.sh`
   - **验法本身是不是废的**:`bash scripts/check-card-verifications.sh`
     它筛两件事:验法是不是在让操作者跑 shell 命令,以及验法有没有提到
     任何他能看的界面。**两个数都不是判定,是提问。**
     2026-10-29 抓到一张真坏的:`acc-disp-session-key-ui` 的验法是
     "跑 check-verdict-refs.sh 和 check-cards-clickable.sh,两条都绿即可接受" ——
     那两个脚本是我为了抓自己的账本错误和分支同步错误写的,
     和 Mira 的会话详情界面毫无关系,而且**即使她的功能被删光也还是绿的**。
     他会跑完看到两个绿勾,接受一张他没看过的卡。
     🔴 **2026-10-29 11:00 这个工具本身被抓到假阴性,别只信它的 0。**
        它报 `shell-based=0`,而同时有四张卡的 `acceptance_plan` 在让张先生
        敲 `psql`、跑两遍 `apply-db.sh`,其中 `acc-disp-accplan-lane-c-onyx`
        的第 4、5 步是 **`INSERT INTO acceptances`** —— 用写账本的动作来验 schema。
        两个 bug 叠在一起:(1) 只读 `how_to_verify`(席位写得整齐的那栏),
        shell 在 `acceptance_plan`(产品真正渲染的那栏)里;
        (2) 连接符用 `chr(10)`,换行把 `while read` 的行切碎了。
        已修成 `-F'\037'` + 空格连接,现在报 `shell-based=4`。
        **规矩:计数为 0 时,先拿一个确定应该命中的样本去撞它。
        修好之后必须能对那张已知的卡报出 SHELL,否则说明工具还是坏的,
        不是卡变干净了。**
     **能永远通过的验法比没有验法更糟** —— 它把"我没看"变成了"我看过了"。
     它把每张 pending 卡的 `flux_verdict_ref` join 回 `dispatches`,
     确认那条 verdict 事件真存在、且点名的就是这张卡的派单。
     2026-10-28 实测发现一条 verdict 的 `dispatch_id` 是
     `disp-B4Y5oAhMQwtXm5Vva1fw`(数字 0),而真 id 是
     `disp-B4Y5oAohMQwtXm5Vva1fw`(字母 o)—— 一个字符,而 nanoid 字符集里两者都在。
     它看起来完全是绿的第三方 verdict,指向 0 行。
     **verdict 事件存在不等于它是关于这张卡的。**

2d. **分支落后就让 owner 合并 —— 这一步 2026-10-29 补,之前它只在我的检查里。**
    🔴 结构缺口实例:2026-10-29 06:00 和 07:00,Onyx 的三个分支各落后 track 1 个提交,
       他的 4 张卡因此点不动。而他**没有任何派单** —— `seatloom-waker` 只对
       "有 ready 派单且空闲"的席位发叫醒,所以它正确地不叫他。
       于是"你的卡点不动"这件事只存在于我的 `check-cards-clickable.sh` 输出里,
       没有任何机制把它变成他的派单。他空闲、什么都不知道、卡一直点不动。

    **命令**(先看,再派):
    ```bash
    bash scripts/check-cards-clickable.sh        # 列出每张点不动的卡和它的分支
    ```

    判据:**每个有 pending 卡且分支落后的席位,都必须有一件派单告诉他合并。**
    没有派单 = 他不会知道 = 卡会一直点不动,而 `fleet-status.sh` 只会显示"⛔ 点了会被拒"。

    派单内容必须包含:
    - 他哪些分支落后、各落后几个提交(不要只说"落后 1 个",要说清是哪些分支)
    - 哪些卡因此点不动(点名卡号)
    - `git merge track/infra-foundation` 然后 push,**不要 rebase**
    - 一句"这不是你的错"—— 落后通常是我提交造成的,而他只是没被告知

    🔴 **一个分支一张卡也不派**:分支落后但没有 pending 卡时,同步不紧急,
       派单是浪费他的上下文。落后数本身不说明严重程度,**那个分支上有几张卡才说明**。

2. **有派给我的活就领**:派给 `seat-aegis-001` 的 `ready` 单,`get_next_dispatch` 领来做。
   空闲是免费的,堵住不是。

2c. **查"验绿了但没人验收"的活 —— 这一步 2026-10-29 补,和 2b 是同一个病的另一半。**
    🔴 实例:2026-10-29 01:00 逐件查 `ReviewVerdictIssued`,发现 **10 件 implementation 类
       派单有 pass verdict、而 `select count(*) from acceptances where flux_verdict_ref=<id>`
       全是 0**。工作做完了、独立验绿了、操作者从没被要求验收过。
       019 那张卡从 10-10 挂到 10-29,19 天,就是因为这个。
       这不是"队列空"能发现的 —— 队列空、派单 done、verdict pass,三样都正常,
       而事情停在一个没有任何检查看过的地方。

    ```sql
    -- 有 pass verdict、verdict 没被任何卡引用、该派单也没有任何卡
    select d.id, d.target_seat_id, e.id as verdict
    from canonical_events e
    join event_object_refs r on r.event_id=e.id
    join dispatches d on d.id = r.ref_id
    where e.event_type='ReviewVerdictIssued' and e.payload->>'verdict'='pass'
      and d.payload->>'kind'='implementation'
      and not exists (select 1 from acceptances a where a.flux_verdict_ref=e.id)
      and not exists (select 1 from acceptances a where a.dispatch_id=d.id)
    order by d.completed_at desc;
    ```

    **三条都要查,只查一条会漏**:
    - "派单 done" → 有活没人接(2b)
    - "verdict pass 但没被卡引用" → 验绿了没人验收(2c)
    - "卡 pending" → 卡点不点得动(第 1 步的 `check-cards-clickable.sh`)

    查到就派 owner 开卡,**verdict 用 pass 那条**;有 fail 和 pass 两条时,
    fail 那条往往发在修复落地之前(实测:`ev-verdict-integration-round` fail 在 pin `575d85d`,
    pass 的 `ev-verdict-integration-round-v2` 才是关于现在的代码)。

2b. **查"干完了但没人接"的活 —— 这一步 2026-10-21 补,之前是空的。**
    🔴 空洞实例:净化单 `disp-FzldOwJuxCfxc2TGq9dMw` 04:15 完成、 Mira 同一秒
    发了 SupervisorMessage,但我随后 8 轮心跳每轮都只查"有没有 ready 派单",
    于是每次报"队列空、无需动作"。**那单在账本上躺了 8 个多小时,直到张先生
    来告诉我"Mira 完成了"。** 一个需要操作者来通知的流程,和停摆没有区别。
    命令(不只看 done,要看 done **且没有验收卡**的):
    ```sql
    select d.id, d.target_seat_id, d.completed_at
      from dispatches d
     where d.status='done'
       and not exists (select 1 from acceptances a where a.dispatch_id=d.id)
       and d.completed_at > now() - interval '48 hours'
     order by d.completed_at desc;
    ```
    查到就该接:写交付说明、派 Flux 验绿、验过就开卡给张先生。
    **只查 48 小时内的** —— 更早的要么已处理,要么是要单独裁决的历史遗留。

3. **有席位等门事件,就按「live 可用」核真实条件**:
   门的条件是"能不能真的跑成功",**不是**"代码合没合进来"。
   迁移在 track 上 ≠ 迁移在跑着的库里;工具编进二进制 ≠ 工具能成功调用一次。
   条件没满足**绝不发门事件** —— 向账本谎称某事已发生,比卡住更糟,下游会基于假前提开工。

4. **有陈旧 `in_flight` 就处理**:它会让 `seatloom-waker` 永久跳过那个席位
   (唤醒服务看到"有 in_flight"就认为它在忙、会自排空)。叫醒它是安全的兜底;
   **绝不**替它把状态改写成 `failed` 或 `ready` —— 那是改账本,不是它的权限。

5. **有需要张先生决策/授权的,汇总给他**,只讲三件事:
   **变了什么 / 你会看到什么 / 需要你做什么。**
   - 禁用自造缩写(G1–G5、O1–O3、Lane A/B/C、S0–S5、D1/D2 一律不要)
   - 不要打印席位的派单标题给他看
   - 被权限拦住的动作:写成 `scripts/` 下一条可审计、可重复跑的命令给他,
     **不拆成小块绕、不换工具绕、不让席位代跑**。

## 它刻意不做的事

- 不改任何账本行、不跑迁移、不注册席位 —— 那些是被权限封顶的动作
- 不打断正在干活的席位(洗数据席位一直在干活,要等它自己交付完再重启)
- 不在没核真实条件的时候发门事件
