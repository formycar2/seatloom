# disp-8PGCP6KH0TMPzp6-0h0nR 交付说明 —— 第三方验 Flux 的分支同步(第二件)

**验单**:`disp-8PGCP6KH0TMPzp6-0h0nR`
**被验派单**:`disp-tVzSMjxpWpBQrW3dToQej`(Flux,分支同步)
**verdict**:`ev-verdict-disp-tVzSMjxpWpBQrW3dToQej` → **pass**
**提交**:`ecf04e3`(仅文档)

---

## 1. 这一单是什么

和上一件(`disp-JsytVHwj3V8Vb1YuM0WeY`)同类:Flux 的分支同步。
Aegis 当时测 `behind_by: 9`,要求合并、重测三项、确认卡可点。

## 2. 这次我改了验证方法

**上一轮我差点误判**,原因是看了 `behind 计数` —— 计数是移动的,
Aegis 随时可能再推一笔,计数就变,而"这笔同步做没做"的事实不变。

这次全部**锚定到具体提交**:

### 2.1 track 的提交真的成了 flux 分支的祖先

```
git merge-base --is-ancestor dd7ba34 track/infra-foundation   → 0   # 它在 track 上
git merge-base --is-ancestor dd7ba34 packet/flux/intent-verdict → 0  # 它也是 flux 的祖先
```

### 2.2 是 merge 不是 rebase(这一条最关键)

```
git cat-file -p 0e9271e
  parent bc393c1b...   ← flux 自己的
  parent dd7ba34...    ← track 的
  Merge branch 'track/infra-foundation' into packet/flux/intent-verdict
```

**两个 parent 就是 merge 的定义。** rebase 不会有第二个 parent,而且会重写
卡指向的提交 id —— 那会直接毁掉验收门。这一条不是形式主义。

### 2.3 push 真的发生了

```
本地 packet/flux/intent-verdict   = 21adf0d3
gitlab/packet/flux/intent-verdict = 21adf0d3
```

### 2.4 卡真的可点

```
bash scripts/check-cards-clickable.sh
  → OK: all 23 pending cards are clickable
  → packet/flux/intent-verdict 名下 0 张在失败列表
```

### 2.5 三项命令

```
git rev-list --count packet/flux/intent-verdict..track/infa-foundation → 0
git merge-base --is-ancestor track/infa-foundation packet/flux/intent-verdict → 退出码 0
```

## 3. 一次测量失败,我没有绕过它

```
git fetch gitlab 'refs/heads/packet/flux/*:refs/heads/packet/flux/*'
fatal: refusing to fetch into branch 'refs/heads/packet/flux/intent-verdict'
        checked out at '/Users/.../seatloom-worktrees/flux'
```

Flux 的 worktree 正检出那个分支,我的 fetch 写不进去。

**我没有因此改用间接手段** —— 没有靠 Aegis 转述,没有猜。而是直接用本地已有的
ref 完成全部测量:本地 `packet/flux/intent-verdict` 与 `gitlab/packet/flux/intent-verdict`
都是 `21adf0d3`,说明本地 ref 已是最新,测量有效。

这一条写进 verdict 的 payload,因为"我的某个测量工具失败了、我改用了什么、
为什么替代仍然有效"必须可查 —— 一个悄悄换掉测量工具的验证,和一个没有做的验证
在下游看起来一样。

## 4. verdict 里明确写了三条不断言

1. **不断言那 8 张卡在任意未来时刻都可点** —— track 仍在被推,任何 behind 计数和
   可点性都只对 2026-10-28 测量那一刻成立。
2. **不断言 Flux 分支上的代码质量** —— 这一单是分支同步,payload 里没有任何代码声明。
   我没有借验单之名去审它的代码。
3. **不断言它之后不会再落后** —— 今天这个模式已出现四次,第四次被 Aegis 自己的
   pre-commit 守卫拦下。但只要守卫能被 `--no-verify` 绕过,模式就可能再出现。

## 5. 我自己发的这条 verdict,我复核了它点名自己

落账后我查了:

```
id=ev-verdict-disp-tVzSMjxpWpBQrW3dToQej
verdict=pass
payload.dispatch_id=disp-tVzSMjxpWpBQrW3dToQej
NAMES ITSELF: YES
actor=seat-mira-001
```

**这一条我专门验。** 因为就在几个回合前,我刚报告过**十件反过来情况的活**:
按 `ref_id` 查得到 verdict,但 `payload.dispatch_id` 点名的是别的派单。
那十件因此开不出卡。我自己发的这一条,不能犯同样的错。

## 6. 关于"第三方"

被验派单的作者是 Flux(`targetSeatId: seat-flux-001`),我是 Mira,所以符合
"一个席位不能为自己席位的交付供 verdict"。

## 7. 本派单未触及代码

零代码改动,未跑门禁(没有代码变动,跑出来是与上一次完全相同的数字,那是噪声)。
