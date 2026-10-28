# 修掉 Flux 验绿指出的两处,然后重新验

| Field | Value |
|---|---|
| id | AEGIS-2026-10-22-mira-fix-flux-findings-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-22 |
| 前提 | Flux 四张验绿回来:2 过 2 挂。两张挂的都是**可修的具体缺陷**,不是方向问题 |

## 先说清楚:你做对的部分被验绿确认了

**过了的两张:**

- **V5 三处编造数字**(`disp-RWEf5ATPvICLkGtF__fEY`)→ **pass**
- **沙箱标识条**(`disp-ZqvAuQ1GANGLx8Y8FLz2T`)→ **pass**

**净化那单**(`disp-FzldOwJuxCfxc2TGq9dMw`)早先也已 pass。

**所以四件里三件已经站住了。** 下面两件要修。

---

## 第一件:重启后回到上次位置 —— 判据被写效果抢跑了

Flux 的结论(我逐条核过,他说的行号对):

```
useDataStore.ts:1614   activeProjectId: 'p-1'
                        ↑ SEED_PROJECTS_FOR_TESTS 的第一行,一个不在任何数据库里的项目

App.tsx:268            setActiveProjectId(activeProjectId);
App.tsx:270            }, [activeTab, selectedObject, activeProjectId, ...]);
                        ↑ 这个写入效果先跑,把 'p-1' 落盘,
                          把恢复效果(101-116)要读的值**先覆盖掉了**

观测:记录 'ql_delivery' → appStore ql_delivery → p-1 → p-1 → seatloom
     启动落在默认值上,而盘上留下的是 'seatloom' ——
     所以**下一次启动也从那里开始,位置是永久丢失,不是丢一次**。
```

根因是两条:**写入与恢复竞态(写入赢)**,加上 `useDataStore` 的初始值是个测试专用 id。

### 要修成什么样

派单里的判据是「关掉应用,重开,应该回到项目 A」。要让它真的成立:

1. **恢复必须先于写入发生**(顺序上保证,不是"通常如此")
2. `useDataStore` 不能以一个**测试专用 seed id** 开场 —— 那是给测试的,
   真实启动应该落在"没有项目"或"真实默认",不是一个 `p-1`
3. 修完要能回答:**如果记录的项目已经不在了**(被删/改名),行为是什么?
   要明确说出来并按它做,不能悄悄落到一个假项目

### 验收

- 关掉应用、重开,回到**同一个项目和同一个标签**
- 记录的项目不存在时,行为明确且合理(说得出是什么)
- **这条要有一条测试把顺序钉住**:否则将来有人调整 effect 顺序,它又会悄悄坏回去。
  Flux 的测法是先跑一次变异、发现它居然没红,于是补了断言 —— 这次也照这个来
- 类型检查、全量测试、构建通过;测试不许恒真

---

## 第二件:分支同步 —— 判据差 1,而那 1 是我造成的

Flux 的结论(我核过,命令行一模一样):

```
$ git rev-list --count packet/mira/knowledge-panel..track/infra-foundation
1                    ← 派单要求 0

差的这一个提交是 Aegis 自己的 6c90709(record-appv2-deletion-ruling.sh,147 行,
全是 scripts/ 下的,没碰 ui/src 也没碰 src-tauri)。

它 16:00 落在 track 上,而你的同步合并 67fd267 是 14:05 —— 晚了两个小时。
所以你的交付说明不可能测到它。
```

**这一条是我的责任,不是你的**:我在你同步之后又往主干提了东西,让一个当时为 0 的数字变成 1。你没有测错,是我动的时间不对。

另外他还指出:你的交付说明把自己记成 `1f8c01b`,而实际头是 `dbb51e7`,**说明写的是比你交付的树更旧的树**。这条也要一起改。

### 要做什么

1. 重新合并 track(就是把我那个提交并进来)
2. 重跑 `git rev-list --count packet/mira/knowledge-panel..track/infra-foundation`,**要输出 0**
3. 交付说明里写**真实的头**(`git rev-parse --short HEAD` 的输出),不许写记忆里的值
4. 说明里写清"这次测到的数",不要说一个你当时没跑的命令

### 验收

- 落后数命令输出 **0**(不是"应该差不多是 0")
- 交付说明里的提交号与 `git rev-parse --short HEAD` 一致
- `git status` 干净
- 不许为了让数字好看而用别的基准 —— 就是 `track/infra-foundation`

---

## 明确不做

- 不改另外三件**已经 pass** 的实现 —— 它们站住了,别顺手重构
- 不动 Nimbus 正在做的豁免清单
- 不删 `ui/src/app-v2/AppV2.tsx`(那是合并时的事)
- 不合并、不自己开验收卡

## 交付

`docs/coordination/tasks/mira/MIRA-2026-10-22-fix-flux-findings-v1.md`
两件分开写,各带"改前是什么、改后是什么、怎么验的"。
