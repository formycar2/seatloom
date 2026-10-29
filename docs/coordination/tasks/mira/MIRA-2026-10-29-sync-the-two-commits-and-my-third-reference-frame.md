# 同步交付说明 —— Aegis 那两笔,以及一个我差点再次误判的参照物

**触发**:POKE-1793211289 / 1793211705 + `disp-LrA3FWq3Nvx9rpJ7-VW4k` 的后续更正
**提交**:`fd10130`(merge commit),已 push 到 gitlab

---

## 1. Aegis 说对了,我确实落后 2 笔

上一单我结论是"无需合并",依据是远端 `git ls-remote` 给的 `b09ec6e1`。
**那个结论对远端成立,但漏了本地 track 分支上未推的 2 笔。**

| 项 | 值 |
|---|---|
| 落后 | **2**(`c4be522`、`0a7dbc0`) |
| 我的卡 | 2 张点不动(`check-cards-clickable.sh` 报 FAIL 2 of 5) |

合并后:

```
git rev-parse --short HEAD                    → fd10130
git rev-list --count HEAD..gitlab/track-real  → 0
git merge-base --is-ancestor ...              → 退出码 0
git push gitlab                               → 36d8ef4..fd10130
bash scripts/check-cards-clickable.sh         → OK: all 5 pending cards are clickable
我的 2 张在失败列表里                         → 0 张
```

门禁:369 passed / 31 files、typecheck 干净、eslint 135 errors(与 baseline 一致)、
`fd10130` 是 merge commit 不是 rebase。

---

## 2. 我上一单错在哪儿

上一单我用 `git ls-remote gitlab refs/heads/track/infa-foundation` 取远端真实 HEAD,
比出 behind 0,就下了"无需合并"的结论。

**问题:`git ls-remote` 只反映远端。而 Aegis 那 2 笔在本地 track 分支上、还没推。**

我在上一单的交付说明里还专门写了教训:"我看了一个看起来是那个东西的引用,
没有确认它指向远端真实状态"。**然后我用一个只覆盖远端的查询,重复了同一个错误。**

### 2.1 正确的参照物是什么

```
git ls-remote gitlab ...   → b09ec6e1   ← 只有远端
git rev-parse track/...    → 0a7dbc0    ← 只有本地,且领先远端 597 笔
git rev-list --count <mine>..track/infra-foundation  → 2   ← 这个才是我要的
```

**我该用的是"我分支相对本地 track ref 差多少",因为 Aegis 是在主检出提交的,
它的提交先落在本地 track 分支上。** 我上一单为了避开"本地 ref 过期"这个坑,
换成了纯远端视角,结果漏掉了"本地已有、远端还没有"那一类。

### 2.2 这又是一次"换了个查询方式,新方式有新的盲区"

| 轮次 | 我用的量 | 漏掉什么 |
|---|---|---|
| 上上一单 | 本地 `track` ref(显示 0) | 它过期,我差点以为 Aegis 没提 |
| 上一单 | 远端 `ls-remote`(显示 0) | 本地已有、远端未推的 2 笔 |
| 这一单 | `git rev-list --count <mine>..track/infra-foundation` | 无 —— 它直接回答"我差多少" |

**同一个问题上换了三个参照物,前两个各有一个盲区。** 第三个之所以对,
不是它更高级,而是它问的就是问题本身("我落后多少"),而不是问一个代理指标
("远端最新是什么"、"本地记录是什么")。

---

## 3. 合并进来的两笔

```
c4be522 fix(poke): put the verification token at the START, because the TUI truncates the end
0a7dbc0 docs(knowledge): index the verdict ids I handed over without checking what they name
```

第二笔正是 Aegis 记录**它自己**犯的错:它在派单里写了"其中两件共用同一条
verdict … 如果它真的是关于两件的,两张卡都能指它;如果只关于一件,说出来",
然后在纸面上标了风险,**没有自己去查**。40 分钟后它自己刚落地的
`check-verdict-refs.sh` 抓住了。

**这和我上一轮的经历完全同构**:我收到一条指定了 verdict id 的派单,
照着用,开出坏卡,被守卫抓住。我们俩都是"风险写在纸上了,然后没有自己去核"。

区别只在 Aegis 自己写了脚本去抓自己,而我当时没有。

---

## 4. 一个仍待处理的发现

`check-verdict-refs.sh` 现在报 `good=5 bad=3`(本次合并前后都如此):

```
BAD  acc-disp-reemit-stalled-announcements
     verdict_ref=ev-verdict-integration-round-v2
     the verdict does not name THIS dispatch - it names disp-integration-round-verify-v2
BAD  acc-disp-b5VbRor1fHhP3nN4YZI_l   verdict names disp-ySsxkvHxKurEgKpt7Q_Hm
BAD  acc-disp-ibABeNk-nkEDAkI6Qj2sG   verdict names disp-ySsxkvHxKurEgKpt7Q_Hm
```

`acc-disp-reemit-stalled-announcements` 引用的正是**我上一轮刚撤掉的同一条 verdict**。
这三张都不是我的卡,需要各自席位处理。

---

## 5. 遗留(未擅自修)

1. `ui/src/stores/useConversationStore.test.ts:13` —— 1 个 lint error
   (`SessionTurnEvent` import 未使用)。track 上别人的文件。
2. `ui/src/lib/dto-to-v1.ts:159` —— `isPinned: true` 硬编码。
3. Cmd+K / Shift 快捷键在输入中抢焦点(`useGlobalShortcuts.ts:70`、
   `App.tsx:366` 缺 `isTypingContext` 守卫)。
4. 真人 GUI 验证:P0 scrollback、滚轮、链接打开、会话详情那一节、第一屏默认页。
5. `origin`(github.com/formycar2/seatloom)推不上去;`gitlab` 正常。
   `ENGINEERING_STANDARDS` 第 49 行要求 push to both remotes,我不自己绕。
6. 上述 3 张 BAD 卡。
