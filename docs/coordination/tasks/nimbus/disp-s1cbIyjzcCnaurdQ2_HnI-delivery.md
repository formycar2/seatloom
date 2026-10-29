# disp-s1cbIyjzcCnaurdQ2_HnI — 开卡:镜像滚动条诚实

## 结果

卡已开出:**`acc-disp-H7pVyxTHO077Rlp6GrmzX`** 修正:`acc-disp-H7pVqxTHO077Rlp6GrmzX`
- 状态:`pending`,等操作者决定
- 分支 `packet/nimbus/project-scoped-reads`,base `track/infra-foundation`
- `flux_verdict_ref` = `ev-sitJrfiH1K8-lLl0hmCbP`

## 开卡前我重测了 verdict 的 refs

因为同一对 (派单 + verdict) 我今天被拒过三次。测出来 Flux 确实补了行:

```
context | dispatch | disp-H7pVqxTHO077Rlp6GrmzX   ← 新增,指向我的实施派单
subject | dispatch | disp-YDwVGLrSef9waU6WvNtK8   ← Flux 的验单
```

门的判据是两条通道任一(`payload->>'dispatch_id'` 或 `ref_type='dispatch'` 的 ref 行),
现在第二条通了,守卫就收了。

## 卡的内容

`how_to_verify` 四条,全是操作者在运行中的应用里做的动作,带明确拒绝条件。
`acceptance_plan` 是 shape v1,5 步 5 条预期。

查过没有 shell / psql / apply-db / INSERT:`how_to_verify` 里出现的两处 "tmux"
都是中文句子里的词(「tmux 已经丢掉了」「tmux 答不上话」),不是命令;
plan 里 `input` 全为 null,零条命令。

## 两件要上报的事

### 1. 第一次调用被拒:plan 里有 `rejection` 字段

v1 schema 的合法字段只有 `plan_version` / `scenario` / `steps` / `expected` /
`preconditions`。我原本把拒绝条件单独放在 `rejection` 里,被严格校验挡了。
**这不是 bug,是 schema 在干活** —— 拒绝条件属于 `how_to_verify`,
不属于 plan。我把它并回去就通过了。写在这里是因为下一张卡别再把拒绝条件
塞进 plan。

### 2. 分支推不上去:gitlab 不通

`gitlab.basemind.com:22` 从这里不可达(DNS 解析到 `10.148.8.16`,
重试 5 次带退避全部超时,`ls-remote` 返回空)。

**我还是开了卡**,理由是门禁只读本地 ref:`resolve_repo_root` +
`local_branch_exists` + `merge-base --is-ancestor` + `merge-tree` 全是本地 git
操作,不碰网络。本地分支是同步的:

| 项 | 值 |
|---|---|
| `rev-list --count <branch>..track/infa-foundation` | 0 |
| `merge-base --is-ancestor` | PASS |
| 冲突 | 0 |
| 本地 tip | `d1d4403` |
| 门禁 | build 0 / check 0 / clippy 0 / 357 rust 测试 |

**远端落后我的提交**,网络恢复后需要推一次。这不是我判断"推不推无所谓"——
是本地状态已满足门的全部条件,而推送是另一件独立的事。

## 没做的事

没有合并进 track(那是操作者的 Accept)、没有 rebase、没有改任何 verdict、
没有改任何判据。

## 遗留

`cargo fmt` 在本分支上仍报 DIRTY, offenders 是 track 自己的
`pipe_guard_real_steal.rs`,我保持在 track 的版本未动。
