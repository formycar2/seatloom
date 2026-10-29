---
title: "My checker said all 7 cards were clickable one minute before he screenshotted all 7 rejected"
category: knowledge
topic: acceptance-integrity
tags: [acceptance-gate, repo-root, cwd, false-negative, open-a, checker-blindness]
created: 2026-10-29T06:00:00+08:00
source_kind: canonical_event
source_ref: ev-gate-could-not-see-a-repository-and_my_checker_agreed_it_was_fine
confidence: inferred
curation: authored
---

# 我的检查说 7 张卡全可点,一分钟后他截图过来说全都不能点

## 症状

2026-10-29 11:28,张先生发来截图:7 张待验收卡**全部**是灰色的
「验收被拒 / 分支未包含验收基准,请把分支同步到最新」。

而 11:26 我的 `check-cards-clickable.sh` 刚打印:

```
OK: all 7 pending cards are clickable
```

## 根因:门看不见仓库,一张卡都过不了

验收门的判据在 `dispatch.rs:1808`:

```rust
let synced = seatloom_core::git::ops::branch_contains(&repo_root, &args.branch, &base)
```

`branch_contains` 跑 `git merge-base --is-ancestor <base> <branch>`,
工作目录是 `repo_root`。而 `repo_root` 来自 `resolve_repo_root()`:

```rust
if let Some(root) = std::env::var_os("SEATLOOM_REPO_ROOT") ... { return root }
std::env::current_dir()
```

**我用 `open -a` 启应用。`open -a` 不传环境变量,进程 cwd 是 `/`。
我没设过 `SEATLOOM_REPO_ROOT`。所以门在 `/` 下跑 git:**

```
$ cd / && git merge-base --is-ancestor packet/x track/infra-foundation
fatal: not a git repository (or any of the parent directories): .git
```

`branch_contains` 区分了三种退出码(0=是祖先,1=不是,其它=git 本身失败),
把第三种返回 `Err`。**门把 `Err` 映射成 `BranchNotSynced`。**

于是:分支全部同步(behind 0),7 张卡全被拒,理由是「分支未包含验收基准」。
**门说的每一句话都是错的,而它不知道自己错了 —— 它以为自己在说分支没同步。**

这不是 10-28 那类"我 push 了所以卡住了"。从 10-28 22:25 应用重启起,
这 7 张卡的验收门就一直是坏的,只是没人点过。

## 为什么我的检查从不可能发现

`scripts/check-cards-clickable.sh` 第 19 行:

```bash
cd "$(dirname "$0")/.." || exit 2
```

它把自己 cd 进主检出,然后在那里跑 `git merge-base`。**它修好了自己的运行环境。**

**一个会自己修好工作目录的检查,对它本该抓住的那个失败是瞎的。**

它测的问题是"分支含不含 track",而真正坏掉的问题是"门能不能看见一个仓库"。
**两个不同的失败,我的检查只覆盖了前一个。**

## 修法

### 1. 启动脚本设并验 `SEATLOOM_REPO_ROOT`

`scripts/launch-seatloom.sh`:

```bash
REPO_ROOT="${SEATLOOM_REPO_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
# 断言它真是含集成分支的仓库 —— 指错的 root 比没有更危险
git -C "$REPO_ROOT" rev-parse --verify "refs/heads/$INTEGRATION_BRANCH" || exit 2
SEATLOOM_REPO_ROOT="$REPO_ROOT" SEATLOOM_OPERATOR_REF="$OPERATOR_REF" open -a "$APP"
```

然后**验证它真的到了进程里**(`ps -E`),再跑一次门自己的判据。
一个只在 `open` 那行设了、却没到达进程的变量,等于没设。

### 2. 检查脚本先问"门能看见仓库吗",再问分支同步

在任何分支计数之前,先解析门会用的 repo root(应用进程的环境变量,否则应用的 cwd),
`git -C "$GATE_REPO_ROOT" rev-parse --git-dir`。失败就 FAIL 并退出:

```
FAIL: the acceptance gate resolves its repository to '/', which is NOT a git repository.
  Every card will read 验收被拒 / 分支未包含验收基准 no matter how synced the branches are.
```

分支检查也改用 `git -C "$GATE_REPO_ROOT"` —— 门在哪测,就在哪测。

### 3. 两个方向都要验

修完必须验两个方向,否则只是把数字改小了:

| 状态 | 期望输出 |
|---|---|
| 应用无 repo root | `FAIL: ... resolves to '/' ... ` + 根因 + 修法 |
| 应用有正确 repo root | `gate repo root: ... (is a git repository)` + `OK: all 7 clickable` |

再加一层:直接跑门的真实谓词
`git -C $REPO_ROOT merge-base --is-ancestor track/infra-foundation <branch>`,
四个分支全 PASS。**环境变量到位不等于门能判对,这一步才是。**

## 规矩

### 1. 一个进程的行为取决于它的 cwd 和环境变量 —— 检查必须在那个上下文里跑

在主检出里跑 git,验的是主检出,不是应用。**"我在正确的目录里测出了正确的结果"
和"系统现在是对的"是两件事。**

### 2. 错误映射会把一种失败伪装成另一种

`Err("not a git repository")` → `BranchNotSynced`。用户看到的症状
(分支没同步)和真实原因(门找不到仓库)毫无关系。
**看到症状先问:这个错误是不是别的原因被映射过来的?**

### 3. 修好自己的运行条件的检查,对它要抓的失败是瞎的

`cd $(dirname $0)/..` 让脚本在任何机器上都能跑,也正是它让脚本永远发现不了
「门跑在了一个非仓库目录里」。**检查必须复用被测对象的运行条件,包括它的 cwd。**

### 4. `open -a` 不带环境变量

macOS 上从 Dock/Finder/`open -a` 启的 app,cwd 是 `/`,没有自定义环境变量。
任何依赖环境变量的桌面功能,都必须有一条带环境变量的启动路径,
并且**验证变量到达了进程**。
