# disp-cxCBOxrvKX0qcoON2eg_r 交付说明 —— 补 push,让我卡里第 5 步的判据成立

**派单**:`disp-cxCBOxrvKX0qcoON2eg_r`(Lyra 发)
**提交**:无新提交(本单只是 push,把已有的 3 个提交推上去)

---

## 1. 先实测,Lyra 说得对

```
本地:   5a1245b7
gitlab: 5874b18a
git rev-list --left-right --count → 3  0
```

未推的 3 个提交:

| 提交 | 是什么 |
|---|---|
| `5a1245b` | 上一单修卡里分支名拼写的交付说明 |
| `59f3a0f` | 开三张验收卡的交付说明 |
| `5ec04da` | 第六次同步的交付说明 |

**全是交付说明,没有代码。** 所以门禁不需要重跑(没有任何代码变动),
但这不改变"我没推"这个事实。

## 2. 这个问题的性质比我上一单更值得记

上一单我写卡时把分支名拼错了。这一单是:**我卡里第 5 步的判据就是
「本地与远端一致」,而我没推,等于我自己写的验收条件不成立。**

Lyra 的原话准确:「卡是对的,是状态没跟上。」

也就是说:

- **上一单**:说明书的内容是错的(拼写)
- **这一单**:说明书的内容是对的,但我没有让被它描述的状态变成真的

第二类更隐蔽。第一类你照做会撞到 `fatal`,立刻知道有问题;第二类你照做
会在一堆正确的步骤里,唯一一条关于"我有没有推"的检查上失败 ——
而那条检查恰好是我自己写上去的。

**我写了一条我自己没满足的验收条件。** 写的时候我以为"待会儿会推",
而"待会儿"就是三个提交之后。

## 3. 做了什么

### 3.1 推 gitlab(主远端)

```
git push gitlab HEAD:refs/heads/packet/mira/knowledge-panel
  5874b18..5a1245b  HEAD -> packet/mira/knowledge-panel
```

### 3.2 推 origin(GitHub 镜像)

派单说「按 ENGINEERING_STANDARDS 也要推 origin 镜像」。我查了规范原文
(`docs/coordination/ENGINEERING_STANDARDS.md:49`):

> Push flow: push to both remotes. GitHub(`formycar2/seatloom`) is a mirror.

规范确实这么要求,而且**我此前几轮同步只推了 gitlab,没推 origin** ——
这一条我也是刚查证才发现的,不是只知道这次要推。

```
git push origin HEAD:refs/heads/packet/mira/knowledge-panel
  * [new branch]      HEAD -> packet/mira/knowledge-panel
```

## 4. 三个位置现在一致

```
本地:    5a1245b7
gitlab:  5a1245b7
origin:  5a1245b7
```

## 5. 派单的三条判据

| 判据 | 实测 |
|---|---|
| `git rev-parse packet/mira/knowledge-panel` 与 `git ls-remote gitlab refs/heads/packet/mira/knowledge-panel` 输出相同 | 两者都是 `5a1245b781cfe678a81d79300072ad7fa5509735` ✓ |
| `check-cards-clickable.sh` 仍 OK | **OK: all 31 pending cards are clickable** |
| `check-cards-runnable.sh` 仍 OK | **脚本在 track 上不存在** —— 见下 |

### 5.1 关于 `check-cards-runnable.sh`

`scripts/` 下只有 `check-cards-clickable.sh` 和 `check-verdict-refs.sh`,
没有 `check-cards-runnable.sh`。Lyra 引用了一个还没落地的脚本。

**我没有因此跳过验证**,而是做了它显然想做的事:把我那两张卡里出现的
**每一条命令**逐条跑了一遍:

| 卡里的命令 | 结果 |
|---|---|
| `git rev-parse track/infra-foundation` | OK |
| `git rev-parse packet/mira/knowledge-panel` | OK |
| `git merge-base --is-ancestor track/infra-foundation packet/mira/knowledge-panel` | OK(退出码 0) |
| `git rev-list --count packet/mira/knowledge-panel..track/infra-foundation` | **0** |
| 本地 vs 远端 sha | `5a1245b7` / `5a1245b7` 相同 |
| `git log --oneline --merges -3 packet/mira/knowledge-panel` | 三条 merge commit,形式正确 |
| 卡里再搜 `infa-foundation` | 两张卡都是 **0 处** |

**每一条都通。** 这比跑一个还不存在的脚本更接近"操作员照做不会失败"。

## 6. 明确没做的

- **没有改卡内容** —— 派单明令,拼写已经对了,判据也已经满足,动它反而是无谓的撤销重开。
- **没有合并进 track** —— 那是操作员的 Accept。
- **没有动别人席位的卡。**

## 7. 我自己的两条教训

### 7.1 写了验收条件,就要让被它描述的状态变成真的

我卡里第 5 步写"本地与远端一致",然后隔了三个提交才推。**写一条自己没满足的
验收条件,比不写更糟** —— 它让操作员以为这里被检查过了。

### 7.2 推两个远端这件事,我是这次查规范才知道的

规范第 49 行明写"push to both remotes",而我此前至少六轮同步**只推了 gitlab**。
每次派单只说"push",我就只推主远端,没有去查规范原文。

这一条值得单独说:**当派单说"push"而规范说"push to both remotes"时,
该照规范,不是照派单里那个更短的说法。** 我这次是派单明确提到了才推 origin,
但如果它没提,我还会继续只推一个。
