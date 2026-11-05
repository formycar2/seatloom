# disp-Ra5YbN4t78bzdbYhYkQiV 交付说明 —— 前端写路径补两条非 seatloom 用例

**派单**:`disp-Ra5YbN4t78bzdbYhYkQiV`(Lyra 发)
**提交**:`9fc7bbf`
**门禁**:`vitest` 387 / 31 files / tsc 净 / eslint 145 problems = baseline 零新增 /
prettier 净 / build 2.00s / `cargo test` 23 passed

---

## 1. 先复现她的变异,没有只信报告

她把 `DispatchInboxView.tsx:878` 改成:

```ts
const projectId = 'seatloom';   // 原来是 activeProjectId ?? ''
```

**正是这个派单删掉的硬编码项目,在调用点重新引入了一遍。**

```
pnpm vitest run src/views/DispatchInboxView.test.tsx
  → 14 passed / 14
```

**存活,与她报的一致。**

### 为什么抓不到

enqueue 调用上唯一的断言期待字面量 `'seatloom'`,而硬编码版也传那个 ——
**因为那条测试的 `activeProjectId` 就是 `'seatloom'`**(`beforeEach`,line 101)。

**读路径有非 seatloom 的用例**(line 433,用 `ql_delivery`,断言 `listCalls` 含它)。
**写路径没有等价的。**

`SupervisorIntent.test.tsx` 同样:每个 `activeProjectId` fixture 都是 `'seatloom'`,
且**没有任何断言点名项目参数**。

## 2. 补了两条,一条一个调用点,没动既有测试

### ① `DispatchInboxView.test.tsx` — 写路径镜像

```ts
useDataStore.setState({ activeProjectId: 'ql_delivery', backendHydrated: true });
// ...提交新派单表单...
await vi.waitFor(() =>
  expect(state.enqueue).toHaveBeenCalledWith(
    'ql_delivery',
    expect.anything(), expect.anything(), expect.anything(), expect.anything(),
  ),
);
```

**用 `expect.anything()` 占后续参数** —— 所以它断言的是**项目**,不是整个调用形状。
以后加一个参数,不会把一条作用域检查变成噪音。

**这条测试的形状就是上面那条读路径用例的镜像** ——
`ql_delivery` 也是从那里借的,fixture 已经存在。

### ② `SupervisorIntent.test.tsx` — 两个调用都点名

```ts
useDataStore.setState({ activeProjectId: 'ql_delivery', ... });
// ...typeAndSubmit...
expect(state.planProjectCalls).toContain('ql_delivery');   // 规划关于哪个项目
fireEvent.click(screen.getByText('确认并建卡'));
expect(state.enqueueCalls[0][0]).toBe('ql_delivery');      // 写进哪个项目
```

`planProjectCalls` 是 mock 里**专门为这条断言记录的** —— 只断言 intent 文本的话,
任何项目(包括硬编码的那个)都能过。

## 3. 变异:四个,全红

### `DispatchInboxView`

| # | 变异 | 结果 |
|---|---|---|
| 1 | `const projectId = 'seatloom'`(**她跑的那条**) | **1 红** |
| 2 | 丢掉参数(传空) | **2 红** |

### `SupervisorIntent`

| # | 变异 | 结果 |
|---|---|---|
| 3 | 三处全硬编码 `'seatloom'` | **1 红** |
| 4 | 三处全传空 | **2 红** |

恢复后各文件回到原状,`387 passed`。

## 4. 一个我认为值得记的模式

这两条缺口是**同一个形状第四次出现**:

| 轮次 | 幸存的变异 | 原因 |
|---|---|---|
| M-4 | `chooseTab` 丢 `selectedObjectId` | 磁盘快照看不见"把原值写回去" |
| 徽章 | `> 0` 改 `>= 0` | 断言的前提本身没被断言 |
| 轮询 | `setInterval` 永不到期 | 一个行为没有断言 |
| **这一条** | 调用点硬编码项目 | **测试跑在一个让硬编码值和正确值相同的状态里** |

**最后一条是最隐蔽的一种**:测试没有写错,它只是跑在 `activeProjectId='seatloom'`
这个状态下 —— 而硬编码的值恰好也是 `'seatloom'`。

**一个只在一个状态下跑过的断言,和没有断言的区别,只是它看起来像有。**

## 5. Lyra 验过的部分,我没重做

她独立验了 Rust 侧四个 spawn 点,并自己跑了两个方向的变异,都红,然后
`cp backup` + `diff -q` 恢复到字节一致,baseline 回 23 passed。

**那是承重的,我不重跑。** 我这边 `cargo test` 23 passed 只是确认她的恢复干净。

## 6. 未做

- **没有重构既有测试**(她明确要求不要)。
- **没有 merge into track**。
- **gitlab 仍未推** —— 已知不可达。origin 已推到 `9fc7bbf`。
