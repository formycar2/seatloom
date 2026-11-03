# Aegis 心跳交付 — 2026-10-30 19:00


| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-10-30-1900-heartbeat-false-positive |
| status | delivered |
| author | aegis |
| date | 2026-10-30 |
| version | v1 |
| tags | aegis |
## 变了什么

### 1. 又修了一个自己的假阳性,而且是同一个病的第三次

上一轮我发现 `check-verdict-defects.sh` 会把「查询坏了」读成「没有待验收的卡」,
加了哨兵行修好了。这一轮它开始对一张真卡报缺陷,我去读那条 verdict,发现:

**它在冤枉一份写得非常详尽的 verdict。**

`ev-15UEVJ08G4I0anhIpCjUJ`(Flux 的重新验收)带着:
- `evidence`:375 个测试全过、typecheck 干净、读写字段逐一对上
- `mutations`:三个变异,每个都写了结果(两个红、一个存活)
- `directions`:我给他的四个方向,每个都回答了
- `not_verified`:四条如实列出没验什么
- `product_code_unchanged`:证明这次只加测试、没动产品代码

它被判「没有摘要也没有 finding,和'我没看'分不出来」,只因为
**没有任何字段 literally 叫 `the_honest_summary`,也没有字段叫 `findings`** ——
Flux 写的是 `directions`。

今天早上我刚把「字段名没想到,和字段不在,是同一件事」写进知识库,
同一个下午又在同一条规则上把它重建了一次。

改法:判据从「有没有我叫得出名字的那个字段」改成「有没有任何实质性记录」,
SELECT 增加 `evidence` / `directions` / `mutations` / `criteria` /
`product_code_unchanged` 五列。

### 2. 顺带修掉自己新造的两个 bug

- **read 循环的字段顺序和 SELECT 不一致。** `evidence` 落进了 `farr`(findings 数组)
  的槽位,于是三份 verdict 全被判成「点名了一个缺陷」。全是假阳性。
- **哨兵行少了两个空串列。** union 两边 13 对 14,整条查询语法错误。

这两个都是我自己在「修假阳性」的过程中造出来的。修一个检查,
如果不跑它、不拿真数据撞它,就会变成这样。

### 3. 变异测试(五种坏法,全部正确 FAIL)

| 变异 | 结果 |
|---|---|
| SQL 引用不存在的列 | FAIL |
| SQL 表名写错 | FAIL |
| pending 过滤条件不可能成立 | FAIL |
| union 两边列数不齐 | FAIL |
| 字段顺序打回原样 | FAIL(正好复现我这次的假阳性) |

### 4. Mira 已经把那张卡指向了新 verdict

`acc-disp-faEWBt8kSGgv-6pJs_Rqw` 原来指着 `ev-Dq2vTW8WjIU_hQufDDp2F`(测的是
`0f420bb`),现在指着 `ev-15UEVJ08G4I0anhIpCjUJ`(测的是 `c3484a0`,
也就是分支现在的尖)。

**你点那张卡时,依据的现在是当前代码的验收,不是旧代码的。**

我独立核过 Flux 的两条关键说法:
- `d2fea84` 只改了一个文件(`App.restartRestore.test.tsx`,74 行新增),
  `App.tsx` 一行没动 —— `git show --stat` 核过。
- `git diff 0f420bb..HEAD -- ui/src/App.tsx` 是空的。

所以他不是「换了实现再验一遍」,是「同一个实现,补上了那条缺失的断言」。

## 你会看到什么

`bash scripts/fleet-status.sh` 显示 **3 件待验收**,三张都点得动。

**只有一张我的检查判为不通过:**

`acc-disp-TbGRGr5cvpJFgCeaayu7S`(主题能关闭)。它的 verdict 明说:
「事务原子性,这个单元的核心主张,没有任何测试钉住它。」

我去核了:`repositories.rs` 里 12 个测试没有一个碰 `close_topic`,
也没有集成测试。**这条是真的。**

而且 Nimbus 自己在卡的验收说明里写了更狠的一句:
「这件活在当前的 SeatLoom 里点不出来……如果前两件不可能做到,
那么这一条的拒绝条件是:功能上线了但没有操作者能执行它,就不要点接受。」

我核了他的三个测量,全部属实:
- live 的事件类型清单到 `TopicOpened` 就结束,没有 `TopicClosed`
- 跑着的二进制是 10-28 22:16,比 `close_topic` 存在还早
- 应用里搜 `close_topic`,0 次

**所以这张卡现在的正确动作是不点。** 他说的是实话。

另两张可以点:
- `acc-disp-faEWBt8kSGgv-6pJs_Rqw` — 退出重开落在「派单」
- `acc-disp-E06O6IAaWefF-BOW-CNJd` — 同一份裁定的另一张卡

## 需要你做什么

**还是那一件事:**

```bash
bash scripts/land-023-memory-dedup.sh --apply
```

默认空跑,先带你逐组看那 5 组重复,不替你决定哪条是真的。

## 一个我要说清楚的判断

这一轮我的心跳**没有新发现产品问题**,修的全是我自己的检查。
这不算进展,这是把上一轮的修错补上。真正等你的还是那 5 组重复记忆。
