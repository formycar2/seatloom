# disp-wL0t1amBBqYeqTv-bwelk 交付说明 —— 撤了两张卡重开,修掉分支名拼写

**派单**:`disp-wL0t1amBBqYeqTv-bwelk`(Lyra 发)
**提交**:无代码改动(仅本文件)

---

## 1. 先实测,Lyra 说得对,这是我造成的

```
git rev-parse --verify track/infa-foundation
  fatal: Needed a single revision
git rev-parse --verify track/infra-foundation
  ab444080200df78fc4da295625a865842ca78cc1
```

两张卡里的错误拼写处数:

| 卡 id | plan 里 `infa-foundation` 处数 |
|---|---|
| `acc-disp-CACpK8qlLm96WQY08itl-` | **3** |
| `acc-disp-HlxnnSKW2lX-rMR02AFk2` | **5** |
| `acc-disp-B4Y5oAohMQwtXm5Vva1fw` | **0**(不动) |

Lyra 提到的两张全部命中,第三张(变异测试卡)不涉及分支名,确认 0 处后未动。

## 2. 这个错是怎么来的,值得记

**我在命令行里一直用正确的 `track/infra-foundation`** —— 本回合所有 bash 调用
都是对的。错只出现在**用 codemode 写卡的 JSON 那一刻**。

也就是说:我不是不知道正确名字,而是**在同一个回合里并存了两个版本,复制时抓错了**。
这比"从头就拼错"更值得警惕,因为所有其他证据(命令输出、脚本结果)都显示我用的是对的,
唯一错的地方恰好是操作员唯一会照做的那份文本。

## 3. 做了什么

### 3.1 撤卡

`cancel_acceptance`,两条,reason 写明:分支名拼错(少一个 r)、`git rev-parse`
报 fatal、判据未变、只是说明书拼错。附了实测结果。

### 3.2 重开前重新核 verdict ref

派单要求"已核过仍是有效 pass 且点名该 dispatch"。重核:

| 派单 | verdict | actor | 点名自己 |
|---|---|---|---|
| `disp-CACpK8qlLm96WQY08itl-` | pass | `seat:seat-flux-001` | YES |
| `disp-HlxnnSKW2lX-rMR02AFk2` | pass | `seat:seat-flux-001` | YES |

沿用原 verdict ref,没有换。

### 3.3 用同一个 acceptance_id 重开

`request_acceptance({ acceptance_id: 原 id, ... })`,工具返回
"re-opened ... the previously cancelled card is pending again under the same id"。

`branch` 仍 `packet/mira/knowledge-panel`,`base` 改对成 `track/infra-foundation`。

## 4. 改过哪些字段

**只改分支名拼写,判据一字未动。**

| 卡 | 改了什么 |
|---|---|
| `acc-disp-CACpK8qlLm96WQY08itl-` | plan 的 3 处 `steps[].input` 与 `expected[].output` 里的 `track/infa-foundation` → `track/infra-foundation`;`how_to_verify` 里 1 处同样改 |
| `acc-disp-HlxnnSKW2lX-rMR02AFk2` | plan 的 5 处(`steps[].input` 3 处 + `expected[].output` 2 处)同样改;`how_to_verify` 里 1 处同样改 |

派单特别要求"plan 里 `steps[].input` / `expected[].output` 都要查,别只改一处"。
我两张都逐处查了(3 处和 5 处,与派单实测数一致),并额外在 `preconditions` 里
加了一句"分支名是 infra 不是 infa,拼错会在第一步 fatal" —— 让照做的人在第一行
就能看到正确拼法。

**没有改的**:scenario、steps 的动作本身、expected 的判据、preconditions 的原有条目、
verdict ref、branch、dispatch id。

## 5. 派单的三条判据,逐条实测

| 判据 | 实测 |
|---|---|
| 重开后卡里再 grep 不到 `infa-foundation` | `acc-disp-CACpK8qlLm96WQY08itl-` → plan 计数 **0**,`how_to_verify` 含 `infa` = **false**;`acc-disp-HlxnnSKW2lX-rMR02AFk2` → 同样 **0 / false** |
| `git rev-parse --verify track/infra-foundation` 在 worktree 里跑通 | **OK**(ab44408…) |
| `check-cards-clickable.sh` 仍打印 OK | **OK: all 31 pending cards are clickable** |

三张卡状态都是 `pending`。

## 6. 额外复核

`scripts/check-verdict-refs.sh` → **`good=31 bad=0`**,我这两张重开的卡通过守卫。
我的卡 0 张在 `check-cards-clickable.sh` 的失败列表里。

## 7. 明确没做的

- **没有改判据本身** —— 派单明令,我也认为不该:坏的是说明书,不是验收标准。
- **没有合并进 track** —— 那是操作员的 Accept。
- **没有动别人席位的卡。**
- **没有动第三张卡**(`acc-disp-B4Y5oAohMQwtXm5Vva1fw`)—— 实测 0 处错误拼写,
  动它反而是无谓的撤销重开。

## 8. 一条给 Lyra 的回复

Lyra 在派单里写:「这个拼写不是你的实现缺陷,你的分支确实是同步的(我已实测
merge-base 通过)—— 坏的是卡的说明书,所以只修卡不动代码。」

这个区分是对的,我照做了。补一句我这边观察到的东西:**我的错误只存在于
操作员唯一会照做的那份文本里**,而我自己所有的命令输出都显示我用的是对的。
这意味着一份"看起来已经被验证过的说明书"和一份"没人验过的说明书"在我这里
看起来没区别 —— 我写它的时候没有把其中的分支名拿去 `git rev-parse` 跑一次。

**如果卡里出现的每一条命令都值得先跑一遍,这类错本可以在开卡前拦住。**
这不是请求新工具,是说我下次开卡前该自己做这件事。
