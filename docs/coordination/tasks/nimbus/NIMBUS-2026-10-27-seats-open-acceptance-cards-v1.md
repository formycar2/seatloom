# NIMBUS-2026-10-27-seats-open-acceptance-cards-v1 — 交付说明

派单 `disp-lu24NPShI-80fqWuvNZeq`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-27-seats-open-acceptance-cards-v1.md`。
分支 `packet/nimbus/project-scoped-reads`。

## 判据达成

```
select dispatch_id, status, branch, base, flux_verdict_ref from acceptances
where dispatch_id in ('disp-VWK5z1k7Jcu8-iQFrGVrb','disp-NMcTvureVxwPh_3bFAGxx');

        dispatch_id         | status  |               branch               |          base          |     flux_verdict_ref
---------------------------+---------+------------------------------------+------------------------+--------------------------
 disp-NMcTvureVxwPh_3bFAGxx | pending | packet/nimbus/project-scoped-reads | track/infra-foundation | ev-hh9xak8wlN5fJst6107cy
 disp-VWK5z1k7Jcu8-iQFrGVrb | pending | packet/nimbus/project-scoped-reads | track/infra-foundation | ev-hh9xak8wlN5fJst6107cy
```

**两行 pending,每张 3 steps / 3 expected / 3 preconditions。**

## 开卡之前发生的事(按顺序,因为有两件会影响卡的有效性)

### 1. 先核 verdict,没有直接信任务包

`ev-hh9xak8wlN5fJst6107cy` 实测:`verdict=pass`,`refs` 里
**同时**挂着 `dispatch:disp-VWK5z1k7Jcu8-iQFrGVrb`、`dispatch:disp-NMcTvureVxwPh_3bFAGxx`、
`dispatch:disp-i_ZLKc4E2S0Bm78K-Zmdy`。所以一条 verdict 覆盖两个单,任务包说的是对的。

### 2. 分支落后 65 个提交,而且**命中判据**

`request_acceptance` 要求「分支包含 base」,实测 `git merge-base --is-ancestor` 不成立。
按新判据(范围,不是数字)核那 65 个提交:

```
118 docs/coordination      100 ui/src        26 src-tauri/src
 11 crates/seatloom-core    9 crates/seatloom-mcp-emit
```

**`ui/src` 和 `src-tauri/src` 都在区间里** —— 不是可跳过的文档同步,必须合。

合并有两个冲突,都是我在意的文件:

- **`docs/coordination/knowledge/INDEX.md`**:纯追加撞车(我一行、track 两行)。**两行都留**。
- **`scripts/verify-tauri-bundle.sh`**:add/add 冲突。track 那版**改的是我写的注释**。

### 3. Flux 纠正了我一个错误结论,我接受了

我在 tauri 那一单里写:「`isComposing` 和 `proposeDimensions` 来自 @xterm/addon-fit,
**不是** SeatLoom 的代码」。Flux 在 track 上把它改成了:

```
isComposing      → ui/src/hooks/useGlobalShortcuts.ts:31 (SeatLoom 自己的)
proposeDimensions → @xterm/addon-fit (SeatLoom 只在一个测试里断言它)
```

我复核实测:Mira 的 shift fix 在那之前合进 track,把 `isComposing` 带进了 SeatLoom 源码。
**我的原话错了一半** —— `isComposing` 是 SeatLoom 自己的,`proposeDimensions` 才是 xterm 的。

所以这次冲突我**取 track 那版**,没有保留我那段错注释。
值得记的是:那个错误不是笔误,是**我在写它的时候没查 `ui/src`,只查了 `ui/node_modules`** ——
而我那一单的交付说明里还专门写了「这两个串证明的是 xterm 不是 SeatLoom」。
一个没核就写下的结论,比没写更糟。

### 4. `dompurify` 没装,21 个测试文件起不来

合并后跑 `pnpm vitest run`,2 个文件直接 FAIL:

```
Error: Failed to resolve import "dompurify" from "src/lib/sanitizeHtml.ts"
```

`ui/package.json` 里有 `"dompurify": "^3.4.17"`,但 worktree 的 `node_modules` 里没有
(worktree 是独立 checkout,没继承主仓那次 `pnpm install`)。装上之后:

```
Test Files  23 passed (23)
Tests  278 passed (278)
```

(还有 14 个 "Unhandled Errors" —— 实测**与我的改动无关**:`App.restartRestore.test.tsx`
故意抛一个 `unmocked Tauri invoke` 来测错误路径。stash 掉我的改动重跑,数字一模一样。)

## 两件活合并后都还在(逐一核过,不是假定)

| 件 | 核法 | 结果 |
|---|---|---|
| `~/` 假路径修复 | `grep "path: d.homePath" ui/src/lib/dto-to-v1.ts` | 在(158 行) |
| 同上 | `homePath: string \| null` in `types-dto.ts`、`home_path: Option<String>` in `dto.rs` | 都在 |
| tauri 验证脚本 | `scripts/verify-tauri-bundle.sh` | 在,且跑出来 **PASS**(8/8 逐字节) |
| favicon | `ui/public/vite.svg` | 在 |

## 验证

| 检查 | 结果 |
|---|---|
| `cargo build --workspace` | 0 issue |
| `cargo test --workspace` | 0 failed |
| `cargo clippy --workspace --all-targets` | 0 issue |
| `cd ui && pnpm tsc --noEmit` | 干净 |
| `cd ui && pnpm vitest run` | **23 files / 278 tests 全过** |
| `bash scripts/verify-tauri-bundle.sh` | **PASS**(8/8 逐字节嵌入) |
| 判据 SQL | **两行 pending** |

## 关于 flux 那一单(`disp-i_ZLKc4E2S0Bm78K-Zmdy`)

任务包也问了 flux 要不要为自己那张开卡。**那不是我的单,我没有代开** ——
`request_acceptance` 只允许完成该单的席位调,Aegis 代开会被 `NotCompleted` 拦,我代开同样会被拦。
如果 Flux 认为验单不产代码、不需要卡,请它自己回复理由;我不替它答。

## 需要上报

1. **两张卡已 pending,张先生现在点得到了。**
2. **`~/networkorchestrator` 那张卡要重新构建 + 安装才看得到效果**(Rust 的 `ProjectDto` 和
   UI 的 mapper 都改了)。装不装仍等他一句话,我没动 `/Applications`。
3. **`scripts/verify-tauri-bundle.sh` 的注释现在是 Flux 修正过的版本**,和我 tauri 交付说明里
   写的那段不一致(说明里说我当时查的是 `ui/node_modules`)。以代码里的为准;
   交付说明是历史记录,不改写,但**读者该知道它那段关于 xterm 的结论是错的**。
4. **worktree 的 `node_modules` 和主仓不同步**这次以 `dompurify` 的形式暴露出来。
   每个 worktree 都要单独 `pnpm install`,而 `pnpm-lock.yaml` 是共享的 —— 这类问题会
   以「测试起不来」的形式反复出现,值得写进建 worktree 的清单。

## 提交

`docs(nimbus): open the two missing acceptance cards`(绝对哈希见 `git log --oneline -1`),
含两个 merge 提交(`44723e5`、`0fffd86`)与冲突解决。
