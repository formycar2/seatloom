# disp--laHQ4kLq7lG_xVCG1A7w 交付说明 —— 删死分支,两份名单合一

**派单**:`disp--laHQ4kLq7lG_xVCG1A7w`(Lyra 发)
**提交**:`5a2314d`
**门禁**:379 tests / 31 files,tsc 净,eslint 135 errors = baseline,build 2.13s

---

## 0. 先自己核,没有只信报告

Lyra 说这是她在自己代码里引入的缺陷、Flux 在同一条 pass 里抓到、与我的四件发现无关。
**我先核了两条,都成立。**

### ① 死分支

```
ui/src/lib/provenance.ts
  if (tab === LIVE_MIRROR_TAB) { ... }               // 捕获 live-seats
  if (LIVE_TAB_BACKING[tab] === 'database') { ... }  // 捕获 sessions/dispatches + 我加的六个
  if (LIVE_TABS.has(tab)) return 'live';            // ← 不可达
  if (LEDGER_TABS.has(tab)) return ... ;
```

`LIVE_TABS` 的三个成员全部被前两个分支捕获。

**实测删掉它:23 条测试全绿。没有任何测试注意到。** 这就是死代码的定义,
而她的判断——"比死代码更糟的是它读起来像一条活规则"——我认:
审 provenance 的人数到三个门,会合理地以为第三个可达。

### ② 两份名单漂移

```
LIVE_TAB_BACKING 有九条:live-seats, sessions, dispatches, dashboard,
                    inbox, workitems, seats, artifacts, playbook
LIVE_TABS        有三条:live-seats, sessions, dispatches
```

六条在 BACKING 不在 LIVE_TABS;`live-seats` 在 LIVE_TABS 不在 BACKING(它是 tmux)。
**它们朝已经漂移的方向继续漂移,没有东西管。** 而没被管的那份正是喂死分支的那份。

## 1. 三处改动

### ① 删掉死分支

原处留注释,说明它曾在那、为什么不可达、以及为什么"读起来像活规则"比死代码更糟。
**没有静默删除** —— 下一个读到这段历史的人需要知道那行曾经存在过。

### ② 两份名单合一

```ts
export const LIVE_TABS: ReadonlySet<string> = new Set(Object.keys(LIVE_TAB_BACKING));
```

**`LIVE_TABS` 从 `LIVE_TAB_BACKING` 派生。** 只有一个地方能加 tab,
两份名单在构造上不可能不一致。

我按派单要求先确认了 `LIVE_TABS` 没有别的使用点:

```
grep -rn 'LIVE_TABS' ui/src
  provenance.ts:47        声明
  provenance.ts:52,67,115 三处注释
  provenance.ts:141       已删的死分支
  provenance-honesty.test.ts:3,12,55,70,76,109,110,128,151  测试
```

**产品代码里除了声明、注释、死分支,没有别的读者。** 测试里读了它,
而那些测试现在改成都钉"它由 BACKING 派生"这件事。

### ③ 测试改为钉构造,不是钉内容

内容已经由精确集合断言钉住了。新加的测试钉的是**构造**:

```ts
it('exactly ONE list decides a tab backing, and the derived set cannot disagree with it', () => {
  expect(LIVE_TABS.size).toBe(Object.keys(LIVE_TAB_BACKING).length);
  for (const tab of Object.keys(LIVE_TAB_BACKING)) {
    expect(LIVE_TABS.has(tab), ...).toBe(true);
  }
  for (const tab of LIVE_TABS) {
    expect(LIVE_TAB_BACKING[tab], ...).toBeDefined();
  }
});
```

**如果有人重新引入第二份手写名单,这条红。**

## 2. 变异测试:三个,全红

| # | 变异 | 结果 |
|---|---|---|
| 1 | **只往 backing 名单加一个 tab**(派单点名) | **4 红** |
| 2 | **删掉 backing 分支**(派单点名) | **4 红** |
| 3 | 重新引入手写的第二份名单(漂移方向) | **1 红** |

派单点名的两个方向,各自由 4 条测试抓住。

### 关于变异 1 的说明

往 `LIVE_TAB_BACKING` 加 `timeline` 会红 4 条,包括"两个 tab 集合必须互斥"和
"timeline 保持 ledger 读法"。**这是因为 timeline 是 LEDGER_TABS 的成员,
把它加进 LIVE 集合就是让它同时属于两个互斥集合** —— 派单说的"漂移方向"
比"加一个 tab"更精确:**真正的危险是让一个 tab 同时出现在两份语义不同的名单里。**

## 3. 一个我自己写测试时犯的错

我第一版防漂移测试写了:

```ts
for (const tab of LIVE_TAB_BACKING) { ... }   // ← 对象不可迭代
```

TS 报 `Type 'Readonly<Record<...>>' must have a '[Symbol.iterator]()' method`。
改成 `Object.keys(LIVE_TAB_BACKING)`。

**记下来是因为它暴露了一件事**:`LIVE_TAB_BACKING` 是 `Record`,`LIVE_TABS` 是 `Set`,
**两者本来就不能直接互迭代** —— 这本身就是"两份名单"留下的摩擦。
派生之后这个摩擦还在(一个对象一个集合),但至少它们的内容不可能不一致了。

## 4. 未做

- **推送失败** —— gitlab SSH 22 端口持续超时(重试 3 次),origin 一直无权限。
  提交在本地完整(`5a2314d`),工作树干净。
- 派单说"if something else reads it, say so rather than deleting a live export" ——
  **我核过了,产品代码里没有别的读者**,只有测试,而那些测试已改。
