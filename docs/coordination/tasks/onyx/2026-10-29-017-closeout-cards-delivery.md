# 017 closeout delivery — 3 个单位,2 张卡开成,1 张结构性卡住

Onyx. 2026-10-29. 派单 `disp-JvrAI-hsYT02Id1mdFY6_`(implementation,撤销了终局失败的
`disp-acclose-onyx` 之后重派的那张)。

**结论:015 和 016 的卡开成了;017 的卡开不了,原因不在活上,在派单的状态机上。**

| 单位 | 派单 | 派单状态 | 验绿 | 分支同步 | 卡 |
|---|---|---|---|---|---|
| 015 知识层 | `disp-d5-lane-a-onyx` | done | `ev-zCWuVawAa1nPL5HLgqu45` PASS | 已同步 | **✅ `acc-disp-d5-lane-a-onyx`** |
| 016 方案载体 | `disp-accplan-lane-c-onyx` | done | `ev-jWG35mnOsceuDoXmXqA7c` PASS | 已同步 | **✅ `acc-disp-accplan-lane-c-onyx`** |
| 017 作废路径 | `disp-acclose-onyx` | **failed**(终局) | `ev-WLLW6NuT1rbcTe4-2gCMw` PASS | 已同步 | **❌ 开不了,见 §2** |

跑项目自己的检查脚本:`OK: all 7 pending cards are clickable`。
我这张分支(`packet/onyx/topic-opened-019`)和 015/016/017 三条分支都 behind 0、
track 是祖先。

---

## 1. 先说三件和派单描述不一致的事(都量过)

### 1.1 「卡还开着」不成立 —— 一张都没有

派单标题写「卡还开着」。量的是:三张卡一条记录都没有
(`select ... from acceptances where dispatch_id in (三个) ` → 0 行)。
真实的说法是**卡从没开过**,和 019 那张一样。这是同一个病的第三例:
活干完、验绿 pass、门量过,卡这步没人做。

### 1.2 「两个迁移现在都在 live 了」—— 是三个,而且都在 track 上

派单说 016 和 017 在 live。我量的是 **015、016、017 三个都在 live**
(`memories` 表在、`acceptances.acceptance_plan` 列在、`acceptances_status_vocab`
含 `cancelled`),而且三个**都已经在 track 上了**,内容和我这三条分支逐字节相同。

三条分支的代码提交(`4520758` / `a6f2ac8` / `cddc6ce`)**都不在 track 的历史里** ——
track 上那条 `eceb264`(feat(schema): land migrations 015-018 on track)是独立落的。
所以和 019 一样:活已经以另一种方式在主线上了,缺的只是卡。

### 1.3 拦我的不是「二进制不带 plan」,是分支没同步

派单说旧 blocker 是二进制不支持 `acceptance_plan`。**那个 blocker 已经没了** ——
我这次调用带了 plan,工具收下了(015/016 两卡都成功开了)。

真正拦我的是工具的另一条守卫,实测原文:

```
request_acceptance rejected: branch "packet/onyx/acceptance-cancel-schema"
does not contain base "track/infra-foundation" — merge
track/infra-foundation into packet/onyx/acceptance-cancel-schema first;
accepting an unsynced branch would merge work that never saw the base's
later commits
```

三条分支各落后 track **589 个提交**。**先 merge,再开卡**,这是这单真正做的事。

## 2. 017 的卡为什么开不了(结构性,不是我绕不过去)

三条分支都同步完之后,015/016 的卡一次就开成了。017 仍然被拒,原文:

```
request_acceptance rejected: "disp-acclose-onyx" is not a completed dispatch
owned by "seat-onyx-001" (013/014: only a done dispatch of the calling seat
may be handed to the acceptance gate)
```

根因我读了源码确认,不是猜的:`crates/seatloom-mcp-emit/src/dispatch.rs:463`
明确只放行 `done` 的派单。而 `disp-acclose-onyx` 的状态是 **`failed`** ——
它在 10-10 用完了 013 的重试预算,终局失败。

**于是一个死结出现了:017 这件活已经交付完、Flux 验绿 PASS、迁移在 track 上、
live 也生效,但它的派单终局失败了,而验收门只收 `done` 的派单。所以它永远开不了卡。**

这不是我能绕的:`actor_ref` 从服务端取,我不能替别人重发派单;
`cancel_acceptance` 是终局且不删行,也不能把 failed 改成 done。

**需要 Aegis 决定**:要么为 017 重新派一张 `done` 的派单(它自己的 payload 也说了
`superseded_dispatch: disp-acclose-onyx`,方向是对的),要么放宽验收门让它接受
「派单 failed 但活已交付且有独立 PASS 验绿」这一种形状。我倾向后者更根本 ——
前者下次还会再撞。

## 3. 合并时解的两个冲突(都是 track 的文件)

### 3.1 `scripts/apply-db.sh`(三条分支各一次)

我这条分支的 POST_SEED 块只登记了自己那条,track 的块有 015-020/022。
**取 track 的** —— 它本来就含我这条的登记行和之后所有行。
三条分支都验了完备性守卫(GUARD PASSES),我那条迁移文件都还在。

### 3.2 `AGENTS.md`(add/add,只在 017 分支上)—— 这条要 Aegis 定,不是我能定的

`AGENTS.md` 是仓库根下的**同一个路径**,而每个席位都在往里写自己的指令:

| 分支 | 谁的 |
|---|---|
| `track/infra-foundation` | Aegis(141 行) |
| 我这三条 | Onyx(43 行) |
| `packet/lyra/*` | Lyra |
| `packet/vela/*` / `packet/mira/*` / `packet/nimbus/*` | Aegis(继承来的) |

一个路径只能有一份,所以**任何带自己 AGENTS.md 的席位分支合并时必冲突**。
我按其他席位分支的做法取了 track 的(Aegis 的),把 Onyx 那份留在分支历史里
(解决时也存了一份 `/tmp/onyx-agents-md.md`)。Onyx 的角色权威在
`docs/coordination/roles/ONYX.md`,那个文件不冲突,所以信息没丢。

**但这是设计问题,不是冲突问题**:要么 `AGENTS.md` 按席位分文件(比如
`AGENTS.onyx.md`),要么席位干脆不跟踪它、只读 `docs/coordination/roles/<SEAT>.md`。
这个该 Aegis 定,我没有擅自改结构。

## 4. 边界

- 没合并任何东西进 track ✅(三条分支都只是 merge track 进来,接受是张先生点的)
- 没 rebase ✅(三条都是 `--no-ff` merge,不改写卡指向的 id)
- 没动 live 库 ✅(全程只读)
- 没改 `crates/**` ✅
- 没给 017 硬开卡 ✅(工具拒了,而且理由是对的 —— 一张点不下去或不合规矩的卡比没有更坏)

## 5. 需要你的两件事

1. **017 的卡**:派单终局失败,验收门只收 `done`。要 Aegis 重派一张,或改门的这条规则。
2. **AGENTS.md 一个路径多个席位**:这次是我手动解的,下次别的席位同步还会撞。

## 6. 提交

三条分支各一个 merge commit,都已 push 到 gitlab 和 origin:
`b42c8c6`(015 分支)、`05cd6e4`(016 分支)、`c8e8bd5`(017 分支)。
