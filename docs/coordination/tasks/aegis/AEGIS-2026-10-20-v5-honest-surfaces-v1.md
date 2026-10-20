# V5 修复:让界面不再显示编造的数字

| Field | Value |
|---|---|
| id | AEGIS-2026-10-20-v5-honest-surfaces-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-20 |
| baseline | track/infra-foundation |

## 为什么派这一单

LYRA-2026-10-19-top-level-value-design-v1 把「产品说的和做的是同一件事」列为五条价值之一,
并排在第一优先级:另四条价值都不值得信,除非这一条成立。张先生多次说过模块看着是坏的 ——
根因不是功能缺失,是**界面显示了不存在的数据**,于是他无法判断哪一页是真的。

## 已逐行定位确认的三处(主干,共 3 个文件)

### 一、写死的「预算健康度」

`ui/src/components/SupervisionDashboard.tsx:62`

```ts
const budgetHealthPercent = 72;
```

常量 `72`,与任何真实预算或健康数据无关,却作为「预算健康度」呈现。

### 二、把当前时刻当成「最后同步时间」

`ui/src/components/SupervisionDashboard.tsx:104`

```tsx
<div className="text-sm font-bold ...">{new Date().toLocaleTimeString()}</div>
```

这是**页面渲染时刻**,不是数据新鲜度。用户会以为那是「数据什么时候同步的」。

### 三、全局仪表盘读纯假数据

- `ui/src/app-v2/dashboard/GlobalDashboard.tsx:11` — `import { MOCK_GLOBAL_SUMMARY } from '../mock-data';`
- `ui/src/app-v2/dashboard/GlobalDashboard.tsx:44` — `const summary = MOCK_GLOBAL_SUMMARY;`
- 数据本体:`ui/src/app-v2/mock-data.ts:292`

该组件由 `ui/src/app-v2/panel/SupervisorPanel.tsx:521` 渲染,**完整可达**,不是死代码。
假汇总内含 `p-1/p-2/p-3` 这类从未存在过的项目。

## 验收标准(缺一不可)

1. 上述三处不再出现编造值。删掉还是换成真实数据由你判断哪种更诚实;**若后端此刻没有对应的真实数据,
   必须明确显示「此数据暂无」并说明原因,不许用任何占位数字冒充。**
2. `:104` 必须区分「数据最后更新时间」与「页面刷新时间」。拿不到真实同步时间就写「未知」,
   不得用当前时刻顶替。
3. 全队/全项目聚合数值必须带范围标注(数的是哪些项目、截止时间),
   **不得把一个项目的数字当成全队的**。
4. 不得新增任何假数据文件;不得为了让测试通过把断言改成恒真。
5. 自行 `grep` 整个 `ui/src` 找出其它同类问题(写死百分比、假汇总、假时钟),
   一并列进交付说明;**不要求全部修改,但要求全部列出。**
6. 完成后构建必须通过。

## 明确不做

- 不改后端、不动数据库
- 不改其它席位负责的文件
- 不自己开验收卡、不合并(合并只能由张先生点)
- 不借这一单顺手改导航结构(那是另一项待他拍板的事)

## 交付

`docs/coordination/tasks/mira/MIRA-2026-10-20-honest-surfaces-v1.md`
写清:改了哪些文件、每处从假值改成了什么、构建怎么验的、grep 还发现了什么。
