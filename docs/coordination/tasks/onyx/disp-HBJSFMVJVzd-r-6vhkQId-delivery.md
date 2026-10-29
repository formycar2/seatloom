# disp-HBJSFMVJVzd-r-6vhkQId — delivery

Onyx. 2026-10-17. 只列清单,零改动(按派单要求)。

**查的基准分支:`track/infra-foundation`** —— 席位脚本在那儿,不在我这个
`packet/onyx/topic-opened-019` 工作区分支上。我这条分支只有 2 处命中(见 D)。

---

## A. `.local/bin/pi` 在 track 上的全部 7 处

| # | 文件:行 | 内容 | 该不该改 | 为什么 |
|---|---|---|---|---|
| 1 | `scripts/seat-launch.sh:28` | `PI="${SEATLOOM_PI_BIN:-$HOME/.local/bin/pi}"` | **不该改** | 已经认 env 了。这就是标准写法,别的脚本照它抄就行 |
| 2 | `scripts/restart-aegis.sh:37` | 同上 | **不该改** | prepare-seat-switch.sh 改过的(交接文档 §2.4) |
| 3 | `scripts/switch-aegis-to-pi.sh:25` | 同上 | **不该改** | 同上(交接文档 §2.4) |
| 4 | `scripts/saih-verify-seat.sh:33` | `SAIH_BIN="${SEATLOOM_PI_BIN:-…/saih/bin/saih}"` | **不该改** | 认 env,而且默认值本来就指向 saih |
| 5 | `AGENTS.md:33` | 文档 —— 「22 字节符号链接,裸 grep 抓不到」那条教训 | **不该改** | 本来就该是 pi。这是排障经验,不是运行路径 |
| 6 | `docs/…/pi_upgrade_changes_disk_not_running_processes.md:40` | 文档,引用 seat-launch.sh:28 | **不该改** | :28 不改,这条就还是准的 |
| 7 | `docs/coordination/tasks/aegis/AEGIS-HEARTBEAT.md:48` | 文档,同一条符号链接教训 | **不该改** | 本来就该是 pi |

### 直接回答:硬编码的运行时脚本 = 0

把 track 上每一个 `scripts/*.sh` 都扫了一遍,找 `PI=` / `SAIH_BIN=` 赋值里
**没有** `${SEATLOOM_PI_BIN:-…}` 包一层的 —— 一个都没有。四个解析运行时的脚本
(seat-launch / restart-aegis / switch-aegis-to-pi / saih-verify-seat)全都认 env。

所以「已经改过两个」这句的准确说法是:**该改的四处本来就都在改好的状态里**
(其中两处是这次 prepare-seat-switch.sh 动手的,另两处一开始就是对的)。

> 顺带一句准确性:交接文档其实描述了**三**处改动 —— §2.3 的 aegis-self-wake.sh,
> §2.4 的 restart-aegis.sh 和 switch-aegis-to-pi.sh。派单里说的「改过两个
> (aegis-self-wake.sh 和另一个)」和文档自己的分节对不上。

---

## B. 同一形状、另一种拼法(字面 grep 抓不到的)

| # | 位置 | 是什么 | 该不该改 |
|---|---|---|---|
| 8 | `~/.pi/agent/bin/pi` | **启动器的真实路径**,和 `~/.local/bin/pi` 是两码事。只出现在注释里:`scripts/restart-aegis.sh:4`、`docs/…/pi_upgrade…md:28` | 不该改。但要知道:`~/.local/bin/pi` 就是这个符号链接(`→ ../../.pi/agent/bin/pi`,实测 22 字节) |
| 9 | `scripts/aegis-self-wake.sh:70` | `node\|bun\|pi\|saih` —— 进程**名**白名单,不是路径 | 不该改,已经含 saih |
| 10 | `scripts/restart-aegis.sh:40` 和 `:196` | `$HOME/.pi/agent/install/current-version`、`stat -f %m …/releases/$diskver` —— pi 安装树里的路径 | 不该改(交接文档 §5:pi 继续装着)。但 restart-aegis.sh 整个脚本的目的就是「让 Aegis 换上磁盘上新版 pi」,迁完之后它需要一个 saih 版孪生脚本,否则它是死的 |

---

## C. 两个真问题(不是硬编码路径,但会让迁移验真本身失效)

### C1. `scripts/saih-verify-seat.sh` 用了 `pgrep -P` —— 正是项目已经踩过并绕开的那个坑

`restart-aegis.sh` 文件头 2026-10-16 白纸黑字记着:

> macOS 的 pgrep **静默看不见调用者的祖先进程**。Aegis 的 bash 命令是 pi 的子进程,
> 所以 `pgrep -P <aegis 的 pane_pid>` 永远返回空 —— 而 pane 里明明跑着 pi。
> ……症状是"查不到",不是报错 —— 一个静默的假阴性。

它还给了绕法(`pane_children()` 用 `ps -axo pid,ppid,command | awk '$2==P'`),
注释写着「刻意不用 pgrep -P」。

**但 `saih-verify-seat.sh:56` 用的就是 `pgrep -P`**,没抄这个绕法。

我实测了(3/3 稳定,不是偶然):

- `pgrep -P 3408`(我这个席位的 pane)→ **空**
- `ps -axo pid,ppid,comm | awk '$2==3408'` → `26257 pi`,进程明明在
- 换一个席位 `pgrep -P 2194` → 正常返回 `3870`
- 我的 bash 是 `26257 pi` 的子进程,`26257` 是 `3408 -zsh` 的子进程 —— 也就是说
  **我要查的进程是我自己的祖先**

**后果(实测,不是推的):我从自己席位里跑 `bash scripts/saih-verify-seat.sh Onyx-data-seatloom`:**

```
########## 第 1 层:面板里跑的是 saih ##########
❌ 验真中断(第 56 行)
--- exit: 1 ---
```

它连「跑的是 '' 不是 'saih'」都没打出来。机制:`pgrep -P` 返回空 → `child` 是空串 →
`ps -o comm= -p ""` 在 `set -euo pipefail` 下非零退出 → 脚本在第 56 行直接死。
我单独验过这条链路:空 pid 时子 shell exit=1,给真 pid 时 `comm='pi'` 正常。

**所以:迁移成功之后,一个席位也没法自验真** —— 第 1 层照样崩在第 56 行。
Aegis 从自己面板跑是好的(他不是我这个席位的祖先),这一点不受影响;
但「席位自己确认自己迁上了」这条路是断的,而且断得很安静。

**修法(我没动):** 把 `restart-aegis.sh` 的 `pane_children()` 那两行抄过来替换第 56 行,
或者至少在第 55 行加一个 `[ -z "$child" ] && { echo "❌ 读不到面板子进程 —— pgrep -P 看不见调用者的祖先,换 ps+awk"; exit 1; }`。
现在这个崩法给人的信息量是零。

### C2. `.gitignore` 在 44 个分支上缺 saih 那几条 —— 这才是会弄脏迁移 diff 的那个

`track/infra-foundation` 的 `.gitignore` 有这 5 组(第 33-48 行):

```
__pycache__/   *.pyc
resume-contexts/
saih/                          # 564 MB
.saih/                         # 席位级 MCP 配置
*.pre-saih.bak                 # prepare-seat-switch.sh 的回滚备份
```

**`main` 加全部 43 个 packet 分支,这 5 组一条都没有。** 我这条
`packet/onyx/topic-opened-019` 就在名单里,证据是 `.saih/mcp.json` 在我这儿
`git status` 里是未跟踪的(`git check-ignore` 说没有规则覆盖它)。

影响分两层:

- **眼前是安全的**:主检出在 `track/infra-foundation` 上,564MB 的 `saih/` 和三个
  `.pre-saih.bak` 在那儿都被正确忽略(我逐个 `git check-ignore` 过了)。
- ** latent 的**:任何一个在 packet 分支上的席位工作区,只要哪天 `saih/` 或
  `.saih/` 落到盘上,一次 `git add -A` 就会把它们收进去。而 AGENTS.md 的完成协议
  写的正是 `git add -A && git commit`。

**这条是清单里唯一「该改」的条目,而且它正好对应派单里那句「改了会让迁移那天的
diff 混进别的东西,看不清」** —— 只不过混进来的不是我的改动,是忽略规则没跟上。

---

## D. 我自己这条分支上的 2 处(顺带)

| 位置 | 内容 | 该不该改 |
|---|---|---|
| `packet/onyx/topic-opened-019:AGENTS.md:26` | 同一条符号链接教训 | 不该改 |
| `packet/onyx/topic-opened-019:scripts/seat-launch.sh:24` | `PI="${SEATLOOM_PI_BIN:-…}"` | 不该改,和 track 上同源 |

---

## E. 三条附带实测(都不是要改的,但迁移那天会用上)

1. **`SEATLOOM_PI_BIN` 现在没设。** 而兜底值 `~/.local/bin/pi` 是指向
   `~/.pi/agent/bin/pi` 的符号链接。所以「env 没设」不等于「安全」,
   等于「符号链接指到哪就跑哪」。迁移真正生效的前提是
   `source saih/seatloom-saih.env` 被执行过。
2. **普查脚本是安全的。** `fleet-status.sh:63-64` 和 `seat-status.sh:100-102` 用的都是
   `pgrep -P <pane_pid>`(按父 PID,不按名字),而 `seat-status.sh:105` 只判断
   `zsh`/`bash`。saih 不会被漏掉。(例外是从席位自己里面跑,见 C1。)
3. **交接文档 §3 第 3 层给的验真命令本身是个不会报错的检查。**
   它让人跑 `grep -n 'node|bun|pi' …aegis-self-wake.sh`,指望看到
   `node|bun|pi|saih`。但这是个**子串**匹配:改前改后都命中。
   人眼看输出能发现不对,自动化 `grep -q` 永远通过。
   `saih-verify-seat.sh:79` 写对了(搜完整串 `node|bun|pi|saih`)—— 以它为准。

---

## 我这一席位的迁移前基线(Aegis 跑验真时会用到)

- 席位 tmux 会话:`Onyx-data-seatloom`(和 `dispatch-seats.tsv` 一致,
  `saih-verify-seat.sh Onyx-data-seatloom` 这个命令本身没写错)
- 工作区:`~/Documents/GitHub/seatloom-worktrees/onyx`
- 工作区里 `.saih/mcp.json` 已在,`SEATLOOM_ACTOR_REF = seat:seat-onyx-001`
  (和 `.pi/mcp.json` 一致,prepare-seat-switch.sh 没搞错人)
- 当前 pane 子进程:`pi`(pid 26257),它的子进程是 `seatloom-mcp-emit`
  —— 所以第 1 层**现在**应该失败,失败是对的;迁完之后才该变 ✅
- 我从席位内自跑第 1 层会崩(C1),Aegis 从自己面板跑不会
