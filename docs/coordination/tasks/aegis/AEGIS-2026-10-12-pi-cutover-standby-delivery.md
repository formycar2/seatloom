# 交付说明 — 2026-10-12 pi 切换日的待命轮(只读)

不是派单交付:这一轮**没有领任何派单、没有写任何账本**。
旧的 Claude Code Aegis 会话还在跑,按交代只建自醒、自报、待命。

## 改了哪些文件

| 文件 | 作用 |
|---|---|
| `scripts/aegis-self-wake.sh` | 自醒 cron 的目标。系统 crontab 每 :11/:41 跑,往 Aegis-Supervisor 面板发一条短的读文件指令 |
| `docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.md` | 心跳协议本体。改心跳行为改这里,不用改 cron。**带待命条件** |
| `scripts/fleet-process-census.sh` | 逐席位进程普查(只读),Lyra 要的那份 |
| `docs/coordination/knowledge/aegis_waker_must_live_outside_the_session.md` | 自醒必须建在会话之外(死过三次的根因) |
| `docs/coordination/knowledge/macos_ps_etimes_silently_yields_now.md` | macOS ps 无 etimes 且关键字清单走 stdout,被 awk 静默吞掉 |
| `docs/coordination/knowledge/INDEX.md` | 登记上面两条 |

提交:`4fc7f0d`(自醒)、`6485ac3`(普查)。工作树干净。

## 怎么验证的

- **自醒真的能跑**:用 cron 的裸环境(`env -i PATH=/usr/bin:/bin`)实跑,
  `/tmp/seatloom_aegis_waker.log` 长出 `POKED Aegis-Supervisor (pane=node)`。
  不是"脚本能执行",是"在 cron 会用的那个环境里能跑通"。
- **crontab 装上了**:`crontab -l` 读回 `11,41 * * * *`。
- **不往 shell 里塞文本**:pane 跑的是 `node`(pi)才发;跑着 zsh 时脚本 SKIP 并记日志。
- **每一步都能报错**:找不到 tmux → exit 1;心跳文件不在 → exit 1;
  `ps` 取不到时间 → 报"未知"并计入需处理,不参与比较。
- **普查的两个坑都修了**:`ps -o etimes=` 在 macOS 不存在且关键字清单走 stdout,
  被 awk 静默按 0 算,导致所有时间戳都变成"现在"——那会让四个旧镜像席位被判成最新。
  改用 `lstart` + `date -j -f` 解析,并校验结果是纯数字。
- **工具数的语义改诚实了**:报的是**盘上二进制**对该席位可见的工具数,
  不是它活着的子进程里的工具数(后者从外部问不出来)。旧镜像席位明确写"未知,只可能 ≤ 18"。

## 有什么需要上报的

1. **旧 Claude Code 会话还活着**(多个 stepgo 托管的 claude 进程)。
   在它关掉之前我不写账本 —— 这是交代,也是我自己认定的必要约束。
   心跳协议里已写成待命条件,自动触发的那一轮也只会读。
2. **Lyra 要的两件已备好,未交付**:v3 单发射者归谁的裁决、舰队进程普查。
   交付动作是 `tmux send-keys` 捅别的席位面板,旧 Aegis 还在时不做。
   普查结果:`6485ac3` 的提交说明里有,一句话 —— 只有 lyra 和 vela 在当前镜像上。
3. **待命解除后第一件事**:确认 crontab 仍在(它不该随会话死,但要核),
   然后把上面两件交付掉,再按队列领活。
