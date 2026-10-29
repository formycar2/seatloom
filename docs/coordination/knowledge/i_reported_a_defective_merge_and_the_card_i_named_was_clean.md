---
title: "I reported a defective merge to the operator, and the card I named was clean"
category: knowledge
topic: reporting-integrity
tags: [false-alarm, wrong-card, verify-before-reporting, verdict-confusion, operator-trust]
created: 2026-10-29T08:20:00+08:00
source_kind: canonical_event
source_ref: ev-correction-i-was-wrong-about-the-defective-merge
confidence: inferred
curation: authored
---

# 我向操作者报告了一次"缺陷合并",而我点名的那张卡是干净的

## 发生了什么

2026-10-29 13:40。张先生说"卡都点好了"。我查账本:7 张全 accepted、全有 merge_commit、全在 track 上。真实且干净。

然后我去查"这些活是否真的能工作",读到 Flux 对 `disp-EUiQEB8IvQb3nxruwvI-k` 的 verdict,
里面大写写着:

> **BUT THE FIX DOES NOT WORK**, and her own two red tests say so

我据此向张先生发出警报:您接受的卡里有一张合进了不能用的修复、它自己的两个测试是红的、
我的四项检查没有一个读过 verdict 内容所以没拦住、已派返工、要不要回滚由您定。

**全是错的。**

## 我错在哪

同一个"第一屏"主题下有**两件独立的活**:

| | 旧 | 新 |
|---|---|---|
| 派单 | `disp-first-screen-default-tab`(10-16) | `disp-E06O6IAaWefF-BOW-CNJd`(10-29) |
| 内容 | 改 `useState` 默认值,一行 | 修持久化覆盖那个缺陷 |
| 卡 | `acc-disp-first-screen-default-tab` | **从没开过** |
| verdict | `ev-FLUX-AUDIT-GATE-A-DEFAULT-TAB` | `ev-EUiQEB8IvQb3nxruwvI-k` |
| 状态 | accepted,**干净** | 未合并、未开卡 |

**我拿手里已有的 verdict 去搜,而没有问"那张被接受的卡实际绑的是哪个 verdict"。**

更糟的是我还"实测"了:在 track 上跑 `App.restartRestore.test.tsx` 得到
`2 failed | 8 passed`,当作证据发了出去。**那测的是 track 上的 App.tsx,
不是 Mira 分支上的** —— 连那个数字都是对着错的代码跑出来的。

## 为什么这个错误特别严重

张先生明确说过两句话:

> 「我其实没看懂了,跟不上你说的内容」
> 「有一些怀疑内容的准确性」

而我这二十分钟里做的,正是让他有理由继续怀疑:**一次具体的、有数字的、有结论的、
催促他做决定的假警报。**

假警报比漏报更伤,因为:
- 漏报是"我不知道",他可以等我查
- 假警报是"我查错了但说得像查对了",他无法分辨,只能整体下调对我的信任
- 而且我催他做决定(要不要回滚)—— **用一个错误的前提催决策,比不催更糟**

## 这和 10-28 那个错是同一个形状

10-28 我把 verdict id 交给席位,**没有查每个 id 点名的是哪件派单**,结果 3 张卡指向别的派单。
今天我**又**拿着一个 verdict 就下结论,**没有查它是哪张卡的**。

**共同点:我相信手里那个对象,不去查关系。**

verdict 不是自由浮动的结论,它是"关于某件派单、被某张卡引用"的三元组里的一环。
只看其中一环就下判断,必然搞混。

## 规矩

### 1. 报给操作者的每一条,都要能回答"我是怎么知道的"

"Flux 的 verdict 说 THE FIX DOES NOT WORK" —— 这句话**只对 `disp-EUiQEB8IvQb3nxruwvI-k` 成立**。
把它讲成"您接受的卡",需要额外一步:查那张卡绑的是哪个 verdict。
**少一步,真话就变成假话。**

### 2. 报错要在同一轮内纠正,不能等下一轮

我发现错了之后,立刻发了更正事件 + 弹窗,说清哪错了、哪是对的、什么不受影响。
**让一个错误前提在他脑子里过夜,比错误本身更糟。**

### 3. "实测"必须说明测的是哪份代码

我在 track 上跑测试,却说成"Mira 的测试在 track 上是红的"。
实际上她的改动在 `packet/mira/knowledge-panel` 分支上,track 上根本没有她那 49 行。
**一个数字不标明它是从哪份代码跑出来的,就不是证据,是装饰。**

### 4. 反过来,这也证明了那个检查工具是对的

我为了验证新写的 `check-verdict-defects.sh` 能报错,拿那张被接受的卡去撞它,
**它没命中 —— 那一秒我就知道我把卡搞错了。**

**一个工具第一次运行就抓出建它的人的错,说明它有用。**
如果它当时命中了,我会更自信地报一个更错的警。

## 结果

- 7 张卡全干净,什么都没坏合进去
- Mira 的缺陷是真的,但**没合并、没开卡**,还在她分支上
- 返工单(授权拆 effect)和 Lyra 的产品裁定单**都仍然正确,已派出**
- 新检查 `scripts/check-verdict-defects.sh` 落地:它读 verdict 的 payload,
  不只看 dispatch_id。**而它的第一次真实使用,是抓住我自己的错误。**
