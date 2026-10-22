# 开四张卡(`disp-RXID_FCaJe9hMQDlhlOJj`)— 开了三张,第四张没开

| Field | Value |
|---|---|
| dispatch | `disp-RXID_FCaJe9hMQDlhlOJj` |
| packet | `AEGIS-2026-10-22-mira-open-cards-2-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-22 |
| 结论 | **三张开卡成功,第四张(重启回原位)没开** —— 它只有旧 fail 验绿 |

## 一、第一张卡被拒,根因是分支落后

第一次 `request_acceptance` 直接被拒:

```
branch "packet/mira/knowledge-panel" does not contain base
"track/infra-foundation" — merge track/infra-foundation into
packet/mira/knowledge-panel first
```

这是新装的二进制起作用了(带删除豁免那版),所以拦我的不再是
`AppV2.tsx`,而是**分支同步**。按新判据先看区间内容:

```
$ git log --oneline packet/mira/knowledge-panel..track/infra-foundation
3b5a61b aegis: 在开卡派单里追加『豁免门已对你生效』+ 派 Mira 重开卡
030e16f aegis: 重启脚本支持指定席位(不必每次重动那四个)
2891e9e aegis: 重启脚本补上『退出后必须拉起』+ 用 saih 启动器
$ git log --name-only --format= <同区间>
.../AEGIS-2026-10-22-mira-open-cards-2-v1.md
scripts/restart-seats-for-new-binary.sh
```

区间里只有 Aegis 的文档和重启脚本,**没有 `ui/src` 或 `src-tauri` 改动**,合并安全。
合并(`5df01f4`)无冲突,`git status` 干净。**没有改任何代码。**

## 二、三张卡:已开,每张点自己的验绿

| 卡 | dispatch | flux_verdict_ref | 如实标注的核心 |
|---|---|---|---|
| 三处编造数字(V5) | `disp-RWEf5ATPvICLkGtF__fEY` | `ev-1ZDgqPoVkpeEB7T7O7hgY` pass | 预算健康度「—」、同步时间「未知(未记录)」沙箱可见;Flux 残留的悬浮说明编造数字与计数映射问题**不声称已解决** |
| 沙箱标识条 | `disp-ZqvAuQ1GANGLx8Y8FLz2T` | `ev-qHnyFs2tY8EtTvJcEiDXH` pass | 沙箱有橙条、主版没有;身份是**启动环境**读的,不是不可伪造的构建事实 |
| 文档净化 | `disp-FzldOwJuxCfxc2TGq9dMw` | `ev-ucN0gvx2vIB_MmDMiZEgz` pass | **界面上看不到**(AppV2 未挂载);图片被剥除、CSP 仍为 null 不声称已解决 |

三张都是 pending operator decision,没有合入、没有自己批准自己。

### 我在写卡时纠正了自己一个判据错误

派单和 Flux 原验绿都把「全局仪表盘」归进 AppV2 路径(不可达)。我实测后**这个说法不完整**:

```
$ rg -n 'GlobalDashboard' ui/src/App.tsx ui/src/app-v2/panel/SupervisorPanel.tsx
ui/src/App.tsx:29:import { SupervisorPanel } from './app-v2/panel/SupervisorPanel';
ui/src/App.tsx:634:<SupervisorPanel onClose={...} />
ui/src/app-v2/panel/SupervisorPanel.tsx:22:import { GlobalDashboard } from '../dashboard/GlobalDashboard';
ui/src/app-v2/panel/SupervisorPanel.tsx:521:<GlobalDashboard onSelectProject={...} />
```

`App.tsx` 挂 `SupervisorPanel`,SupervisorPanel 在 global+dashboard 模式下挂
`GlobalDashboard`。**AppV2.tsx 删了不等于 GlobalDashboard 不可达** —— 它经 ⌘K
监督浮窗是可达的。所以第一张卡我没有照抄「界面上看不到」,而是写了可达路径
(⌘K 浮窗 → 仪表盘 → 全局),并把它从第三项 expected 里改掉。

Flux 的验绿本身没写错(它说的是 `DocumentsWorkspace`/AppV2 不可达,那是文档净化那条);
是**派单表格把「AppV2 未挂载」当成了 GlobalDashboard 的不可达理由**。我按实测写,
不按派单抄。

## 三、第四张卡:没开,原因是它只有 fail 验绿

重启回原位(`disp-DFeH6Ut_aNfzIbuuR21V5`)的实现我 10-22 已经修到 246/246,
但**只有 Flux 10-21 的 fail 验绿** `ev-mkwI3idecZKlZBGXOoC31`:

```
$ query_ledger ReviewVerdictIssued ref=disp-DFeH6Ut_aNfzIbuuR21V5
1 条:ev-mkwI3idecZKlZBGXOoC31 verdict=fail
```

`request_acceptance` 只收 pass,且这个 fail 是**修之前**的。拿它去开就是拿 fail 当绿,
和 Aegis 记过的坑同型。所以这张**没开**。

我也查了有没有新的 pass 验绿:最近 15 条 ReviewVerdictIssued 里,
关于 `disp-DFeH6Ut_aNfzIbuuR21V5` 的只有这一条 fail。**没有 Flux 重验过。**

## 四、需要 Aegis 做的两件事(已上报)

`SupervisorMessage` `ev-mira-open-cards-3-three-opened`:

1. **请派 Flux 重验 `disp-DFeH6Ut_aNfzIbuuR21V5`**。派 Flux 不是我的职权
   (`enqueue_dispatch` issuer 只允许 Aegis/Lyra),所以我只能请求。
   重验通过后我自己开第四张卡,不需要重做别的。
2. **派单表格里「AppV2 未挂载」这个不可达理由,对 GlobalDashboard 不成立**
   (见第二节)。文档净化那条的不可达成立,三处编造数字那条的全局仪表盘不成立。
   这不影响已开的三张卡(我把可达路径写进去了),但下次派同类活别照抄这个理由。

## 五、明确不做的,我没碰

- 没有合并、没有自己批准自己
- 没有为第四张卡找任何替代验绿,没有改 base 或分支
- 没有改任何代码(本单代码改动 0)
- 没有恢复 `AppV2.tsx` 或恢复默认值 `'p-1'`
- 没有把四张合成一张

## 复现命令

```bash
git branch --show-current                                   # packet/mira/knowledge-panel
git merge-base --is-ancestor track/infra-foundation HEAD    # 0 = 分支已含 base
git log --oneline packet/mira/knowledge-panel..track/infra-foundation   # 区间内容,不是数字

# 每张卡的验绿点名自己的实现单(不许互借):
#  acc-disp-RWEf5ATPvICLkGtF__fEY -> ev-1ZDgqPoVkpeEB7T7O7hgY  pass
#  acc-disp-ZqvAuQ1GANGLx8Y8FLz2T -> ev-qHnyFs2tY8EtTvJcEiDXH  pass
#  acc-disp-FzldOwJuxCfxc2TGq9dMw -> ev-ucN0gvx2vIB_MmDMiZEgz  pass
#  acc-disp-DFeH6Ut_aNfzIbuuR21V5 -> 只有 ev-mkwI3idecZKlZBGXOoC31 fail,故未开

# GlobalDashboard 可达性(派单的理由对这条不成立):
rg -n 'GlobalDashboard' ui/src/App.tsx ui/src/app-v2/panel/SupervisorPanel.tsx
```
