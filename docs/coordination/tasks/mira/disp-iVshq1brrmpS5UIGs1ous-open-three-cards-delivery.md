# disp-iVshq1brrmpS5UIGs1ous 交付说明 —— 开三张验收卡

**派单**:`disp-iVshq1brrmpS5UIGs1ous`(Aegis 发,优先级 6)
**提交**:无代码改动(仅本文件)

---

## 1. 开了哪三张卡

| 派单 | 卡 id | verdict ref |
|---|---|---|
| `disp-CACpK8qlLm96WQY08itl-` | `acc-disp-CACpK8qlLm96WQY08itl-` | `ev-verdict-disp-CACpK8qlLm96WQY08itl-` |
| `disp-HlxnnSKW2lX-rMR02AFk2` | `acc-disp-HlxnnSKW2lX-rMR02AFk2` | `ev-verdict-disp-HlxnnSKW2lX-rMR02AFk2` |
| `disp-B4Y5oAohMQwtXm5Vva1fw` | `acc-disp-B4Y5oAohMQwtXm5Vva1fw` | `ev-verdict-disp-B4Y5oAohMQwtXm5Vva1fw` |

三张都是 `pending operator decision`。

## 2. 开卡前我核了三件事,不是照派单转述

派单说"验绿依据刚到位,都是第三方发的 verdict,没有自验"。**我逐件核了**:

| 派单 | verdict | 作者 | verdict actor | 点名自己? | 第三方? |
|---|---|---|---|---|---|
| `disp-CACpK8qlLm96WQY08itl-` | pass | `seat-mira-001` | `seat:seat-flux-001` | **YES** | **YES** |
| `disp-HlxnnSKW2lX-rMR02AFk2` | pass | `seat-mira-001` | `seat:seat-flux-001` | **YES** | **YES** |
| `disp-B4Y5oAohMQwtXm5Vva1fw` | pass | `seat-mira-001` | `seat:seat-flux-001` | **YES** | **YES** |

**这一轮三件全部合格。** 和前两轮完全不同 —— 那两轮我核出 10 件的 verdict
全部点名别的派单,一张都没开。

**为什么这次要先核**:`request_acceptance` 的守卫只检查"verdict 存在、是 pass、
payload.dispatch_id 等于这个派单"。它**不检查** verdict 是不是第三方发的。
所以"没有自验"这一条只能我自己核。我核了,三条都是 Flux 判我,符合
Lyra 10-27 的裁决。

## 3. dispatch id 从账本抄的

派单要求"copy the dispatch ids from the ledger, do not type them"。
三个 id 都是 `get_dispatch` 返回后原样使用,并各核对了长度(都 26 字符)和
`status=done`。

历史教训:我曾因一个字符的打字错误让 `complete_dispatch` 失败。

## 4. 每张卡的验收 plan 是能真跑的

不是填空。三张卡的性质不同,plan 也不同:

### 4.1 / 4.2 两张同步类卡

共同点是**锚定具体提交,不锚定 behind 计数**。这是我在验 Flux 那两件同步活时
差点误判学到的:计数是移动的(主干随时可能再推一笔),"某个提交是不是分支的祖先"
不是。

所以 plan 的第一步是 `git merge-base --is-ancestor <track HEAD> <branch>`,
并明确写出:**只有一个 parent 说明是 rebase,卡指向的 id 被重写,此时必须判不接受。**

`how_to_verify` 写成一步可判:

```
跑 bash scripts/check-cards-clickable.sh:输出不含 packet/mira/knowledge-panel
即可接受;或 git rev-list --count ... 不为 0 即不接受。
```

### 4.3 变异覆盖卡

这张的性质是"测试证据是否和声明一样强"。plan 的核心是**变异测试本身**:

```
1. 跑测试记基线(全过,通过数 N)
2. 把 M6 针对的那处逻辑改回旧代码
3. 重跑 —— 必须至少 1 个失败
4. 还原,再跑 —— 必须回到 N
```

并写明:**一个都不失败说明测试没有盯着这处逻辑,判不接受。**

这正是这一单要补的缺口(声明说"恢复旧代码时 7 个测试全过"),plan 直接把
验收标准设成了那个缺口的反面。

## 5. 用 Aegis 刚落地的守卫复核

开完卡我跑了 `scripts/check-verdict-refs.sh`:

```
== pending acceptance cards: is each verdict ref a verdict about THIS dispatch? ==
  good=31 bad=0

== ReviewVerdictIssued events that name a dispatch which does not exist ==
  (5 条,与开卡前一致,未变化)

OK: every pending card's verdict ref is a verdict about that very dispatch.
```

**`good=31 bad=0`** —— 我新开的三张卡通过检查。开卡前是 28,开完是 31。

同时 `check-cards-clickable.sh` 报 **`OK: all 31 pending cards are clickable`**。

## 6. 派单 note 提到的那一件,我没有动

派单说:「One unit (`disp-tVzSMjxpWpBQrW3dToQej`, yours if you are Flux) is still
being verified by Mira; I will dispatch you for its card when her verdict lands.」

**那一件的 verdict 是我几个回合前发的**(`ev-verdict-disp-tVzSMjxpWpBQrW3dToQej`,
pass),派单写这条时它可能还没落地。我**没有**顺手给它开卡,因为:

1. 派单的 `do` 只列了三件,这一件在 `note` 里且明说"等 verdict 落地我再派"
2. `do_not` 第二条:「不要给 verdict 还不存在的 unit 开卡」
3. 更重要的是:**那一件的 verdict 是我发的**,而它的作者是 Flux。我来开这张卡,
   等于自己给自己判的活开卡 —— 哪怕 verdict 本身是合规的第三方(我是第三方),
   开卡动作也应该是等派单明确指示。

如果 Aegis 要开它,一句话我就开。但不自己伸手。

## 7. 未做的事

- **没有 merge 任何东西**(派单 `do_not` 第一条)。合并是操作员的 Accept。
- **没有**给派单 `note` 里那一件开卡(理由如上)。
- 零代码改动,未跑门禁。
