# disp-ViTA4p7UyTMsoZFZMJmUi 交付说明 —— 依赖栏那件的卡

**派单**:`disp-ViTA4p7UyTMsoZFZMJmUi`(Aegis 发,优先级 6)
**提交**:无代码改动(仅本文件)

---

## 1. 开了

```
acc-disp-blocked-on-whom-ui
  dispatch      = disp-blocked-on-whom-ui
  flux_verdict  = ev-verdict-disp-blocked-on-whom-ui
  branch        = packet/mira/knowledge-panel
  base          = track/infra-foundation
  status        = pending operator decision
```

## 2. Aegis 说它自己核了,我还是核了一遍

派单 `history_so_you_know_why_this_took_three_rounds` 说:

> I verified the payload.dispatch_id myself this time rather than taking it
> from a query result.

**这正是它前两轮没做、我照着做错的那一步。** 它这次做了。

但我仍然自己核了一遍 —— 不是不信它,是这几轮付的学费:

| 轮次 | 谁指定 verdict | 谁核 `payload.dispatch_id` | 结果 |
|---|---|---|---|
| `disp-iVshq1brrmpS5UIGs1ous` | Aegis | 我没核 | 我开出坏卡,被守卫抓 |
| `disp-SI-Qrw0EXuoYLVdKpIoIB` | Aegis | 我核了 | 4 件里 2 件不合格,没开 |
| 本轮 | Aegis(**并说明它核了**) | **我又核了** | 合格,开 |

核到的:

```
id                  = ev-verdict-disp-blocked-on-whom-ui
verdict             = pass
actor               = seat:seat-flux-001
payload.dispatch_id = disp-blocked-on-whom-ui     ← NAMES ITSELF: YES
dispatch author     = seat-mira-001              ← 第三方: YES
occurredAt          = 2026-10-28T19:12:46
```

**派单说的每一条我都复现了。** 这一轮可以照做,是因为我核过之后确认它是对的,
不是因为它说了。

## 3. 守卫从 bad=3 变成 bad=0

开之前 `check-verdict-refs.sh` 报 `good=5 bad=3`,其中:

```
BAD  acc-disp-reemit-stalled-announcements
     verdict_ref=ev-verdict-integration-round-v2
     the verdict does not name THIS dispatch
BAD  acc-disp-b5VbRor1fHhP3nN4YZI_l   verdict names disp-ySsxkvHxKurEgKpt7Q_Hm
BAD  acc-disp-ibABeNk-nkEDAkI6Qj2sG   verdict names disp-ySsxkvHxKurEgKpt7Q_Hm
```

**开之后是 `good=7 bad=0`** —— 那 3 张已被各自席位处理掉。
我前两轮报告里点了其中一张(`acc-disp-reemit-stalled-announcements`
引用的正是我撤掉的那条 verdict),现在它不再报了。

## 4. how_to_verify 是按"张先生能照做"写的

派单两次强调这一点(`he has told me twice he cannot follow seat shorthand`)。
这一版的写法:

> 打开应用的派单页,看每一条派单。如果某一条正在等一件还没发生的事,
> 它下面要写清在等谁、那件事是什么。点一下那条说明里提到的派单编号,
> 应该能跳到那一条。两步都做到即可接受;如果只显示「等待中」而不说在等什么,
> 即不接受。

没有 seat id、没有 dispatch id、没有分支名。步骤是"打开、看、点"。

## 5. 终态

```
check-verdict-refs.sh      → good=7 bad=0
check-cards-clickable.sh   → OK: all 7 pending cards are clickable
我的卡在失败列表           → 0 张
```

## 6. 未做的事

- **没有 merge**(派单 `do_not`)。合并是操作员的 Accept。
- 零代码改动,未跑门禁。

## 7. 遗留(未擅自修)

1. `ui/src/stores/useConversationStore.test.ts:13` —— 1 个 lint error。
2. `ui/src/lib/dto-to-v1.ts:159` —— `isPinned: true` 硬编码。
3. Cmd+K / Shift 快捷键在输入中抢焦点。
4. 真人 GUI 验证:P0 scrollback、滚轮、链接打开、会话详情那一节、第一屏默认页、
   **以及本卡的依赖栏跳转**。
5. `origin`(github.com/formycar2/seatloom)推不上去;`gitlab` 正常。
