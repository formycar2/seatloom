# disp-KMP7yTq6JRIzlq5vNacGY 交付说明 —— 卡换绑到干净的新判定

**派单**:`disp-KMP7yTq6JRIzlq5vNacGY`(Lyra 发)
**提交**:无新提交(派单 `do_not`:不动代码、不合 track)
**HEAD**:`2ee173e`(未变)

---

## 1. 先核新判定,没有只信派单

```
ev-flux-verdict-expect-anything-comment-corrected-03
  verdict            = pass
  actor              = seat:seat-flux-001          ← 第三方
  payload.dispatch_id = disp-Ra5YbN4t78bzdbYhYkQiV  ← 点名 ✓
  findings            = []                          ← 空数组
  no_new_findings     = 存在
  commit              = 2ee173e
```

**六项全中。**

## 2. 前置条件也核了

```
git status --short                              → 空(工作树干净)
git rev-parse --short HEAD                      → 2ee173e7
git merge-base --is-ancestor 2ee173e HEAD       → YES
git merge-base --is-ancestor track ...          → 退出 0  ✓
git rev-list --count HEAD..track               → 0
```

## 3. 撤卡 + 同一 id 重开,一次做完

Lyra 特别警告:撤卡和重开之间有一个窗口,那期间操作员从两张卡变成一张。
**我按她说的在一个 sitting 里做完了,中途没有失败、没有重试。**

### 撤卡

```
cancelled acceptance acc-disp-Ra5YbN4t78bzdbYhYkQiV
  for disp-Ra5YbN4t78bzdbYhYkQiV
  — the card is closed; no merge, no rework
```

理由写清是**换绑不是回退**,并说明旧判定不可改写、它记录的是 06:46 当时为真的事。

### 重开(同一 id)

```
re-opened acceptance acc-disp-Ra5YbN4t78bzdbYhYkQiV
  for disp-Ra5YbN4t78bzdbYhYkQiV
  branch packet/mira/knowledge-panel, base track/infra-foundation
  flux verdict ev-flux-verdict-expect-anything-comment-corrected-03
  — the previously cancelled card is pending again under the same id
```

**`acceptance_id` 显式传了**,所以操作员看到的是他早就被告知过的那个卡 id,
不是一个新的他没上下文的 id。

## 4. `acceptance_plan` 沿用,只改提交号

按 Lyra 要求,计划原样沿用(她说"it is good and it is honest"),
**只把 precondition 里的 `f26466f` 改成 `2ee173e`**。

并且把"为什么换绑"写进 precondition,而不是让它看起来无故重开:

> 本卡替换了此前绑在 `ev-flux-verdict-two-frontend-cases-02` 上的那一版。
> 旧判定不可改写,它记录的是 06:46 当时为真的事(交付说明里 `expect.anything()`
> 的理由是假的);那条注释已改、变异已复验,而修好的事实只能由一条新的独立判定
> 承载 —— 即本卡引用的这条。

## 5. 两个检查的退出码(她要的是码,不是描述)

```
$ scripts/check-verdict-defects.sh
OK: every pending card rests on a verdict that does not name a defect.
EXIT=0

$ scripts/check-cards-clickable.sh
OK: all 2 pending cards are clickable
EXIT=0
```

**两个都 exit 0。** 之前 `check-verdict-defects.sh` 是 exit 1 ——
它读的正是 `acceptances.flux_verdict_ref` 这一列,所以一条没被任何卡引用的
干净判定改变不了任何事。**换绑之后它才看见。**

## 6. 门禁数字(我自己的运行)

| 门禁 | 结果 |
|---|---|
| `pnpm test` | **391 passed / 31 files** |
| `pnpm typecheck` | clean,**exit 0** |
| `pnpm exec eslint src` | **145 problems(135 errors + 10 warnings)** = 主检出 baseline,零新增 |
| `cargo test` | **26 passed; 0 failed** |

**HEAD 未变(`2ee173e`),没有新提交** —— 派单 `do_not` 禁止动代码。
门禁数字与 17:00 那一次一致,因为代码没变。

## 7. 另一张卡

`acc-disp-XpAdx0a9YLL83JVoPesTj` **未动**,仍 pending,仍引用
`ev-flux-verdict-four-write-paths-01`(那条没有 finding)。

Lyra 说它不受影响,我核过:两张卡引用不同的判定,撤一张不会碰到另一张。

## 8. 边界

- **没有 merge into track**。
- **没有 rebase**。
- **没有重启沙箱或主版**(pid 3794 / 90805 均未触碰)。
- **没有写 live 库** —— 只有那两次验收调用。
- **没有碰旧判定事件** —— 它不可改写,也必须留在那里作为 06:46 当时为真的记录。
