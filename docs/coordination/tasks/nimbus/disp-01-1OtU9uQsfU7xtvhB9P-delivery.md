# disp-01-1OtU9uQsfU7xtvhB9P — 给 5 件没被验收就上了 track 的工作补开验收卡

## 同步

落后 15,合并干净(0 冲突、0 未合并、无冲突候选文件)。合并里包含 track 把我两个
迁移文件改名了(`023_memory_dedup.sql` → `025_...`,`024_topic_close.sql` → `026_...`),
已按新名字核。

`behind=0`、track-is-ancestor PASS、362 rust 测试绿、已推 `052f6ff`,远端=本地。

## 5 件的实际状态(逐件量,不猜)

先说结论:**5 件里 3 件已经有卡了**(其中 2 件已被你接受),**1 件我开了卡**,
**1 件不能开卡**。

| commit | 是什么 | 卡的状态 |
|---|---|---|
| `b6ef3aa` | 镜像滚动条报真实深度 + 标注回填 | **已被你接受** `acc-disp-H7pVqxTHO077Rlp6GrmzX` |
| `01517af` | 去重测试不再自证 | **已被你接受** `acc-disp-pQ6kiHvOb-IrrfFW1m1Jx` |
| `57bda95` | 主题能关闭 | **本次新开** `acc-disp-TbGRGr5cvpJFgCeaayu7S` |
| `e011c0f` | clamp 真被调用 + unknown 不显示成 0 行 | **已有卡**,和 `b6ef3aa` 同一张 |
| `ee2d03b` | remember 拒绝重复写入 | **不能开卡**,见下 |

`e011c0f` 我一开始也以为是没卡的,但 `acc-disp-H7pVqxTHO077Rlp6GrmzX` 的
`how_to_verify` 里已经写了「tmux 答不上话时不得显示 0 行」这条,它覆盖的就是
`e011c0f` 的用户可感知部分。所以 5 件实际只有 3 件缺卡,不是 5 件。

## 开出来的那张:`acc-disp-TbGRGr5cvpJFgCeaayu7S`

verdict `ev-U9LHdjblV_bkLtnfjuNzh` 这次**正确指向我的实施派单**(通过
`context | dispatch` 那一行 ref),门禁直接收了 —— 之前三次被拒就是因为 verdict
只指向 Flux 的验单。

但我没有把它写成"你现在就能点"的验收,因为**它现在一步也执行不了**。四条实测:

1. 应用里**没有任何关闭入口**。`src-tauri/` 和 `ui/` 搜 `close_topic` = **0 处**。
   关闭只能由席位通过 MCP 工具发起,操作者点不到。
2. **026 迁移没应用**。live 的 `canonical_events_event_type_vocab` 到
   `TopicOpened` 就结束了,没有 `TopicClosed` —— 直接调工具会被 CHECK 约束拒。
3. **在跑的二进制不含它**。`seatloom-mcp-emit` 是 10-27 19:39 起的,我的主题
   关闭代码 10-29 才落地。
4. live 51 个主题全是 `open`,关一个都关不了。

所以卡的 `preconditions` 第一句就写「⚠️ 当前跑着的 SeatLoom 是 10-28 构建的,
不含本改动」,并列出缺的三件事(重构建安装 / 应用 026 / 重启席位),
**写明这三件是操作者和数据库的动作,我不能代做**。预期里保留了两条真正承重的:
第二次关闭必须被拒**并说出是哪个状态**,以及该主题下的记忆一条都不动。

一张描述不了的动作的卡比没有卡更糟 —— 那是伪造验收。

## 不能开卡的那件:`ee2d03b`(remember 拒绝重复写入)

不开卡,理由是一件产品改动上线了但**用户感知不到**:

- **025 迁移没应用** —— live 上 dedup 索引 **0 个**
- **live 仍有 5 组重复**(同文件同行区间同摘要),每组 2 条

也就是说这个改动现在**没有改变任何用户可见的行为**:重复写入照旧发生,因为
守卫依赖的那个索引不在库里。等 025 应用之后它才真的有用户可感知效果,
那时开卡的 `how_to_verify` 才是真的(写一条重复记忆,看它被拒而不是被复制)。
现在开等于让操作者验收一件他看不出区别的事。

## 关于 `b6ef3aa` / `e011c0f` 的运行态

顺手核了一下,值得记:**在跑的 SeatLoom 是 10-28 22:15 构建的**(MD5
`28753f61...`,和 `target/release` 里那个完全相同),它内嵌的前端**不含**我的
改动。我用 `scripts/verify-tauri-bundle.sh` 核的,没有用裸 grep ——
Tauri 把前端 brotli 压进可执行文件,裸 grep 对压缩资产既会误报也会漏报
(这个脚本自己的注释就记着这个坑)。我从自己的分支重新构建后,
`paneHistoryLimit` / `history-limit 未取得` / `行(回填)` 三个串都在
`dist/assets/main-*.js` 里,说明代码是对的,只是没装到运行态。

所以这两件虽然卡被你接受了,**运行态还是旧二进制**。`scripts/land-the-mirror-fix.sh
--apply` 是装它的入口(会覆盖 `/Applications/SeatLoom.app`),这个决定在你。
