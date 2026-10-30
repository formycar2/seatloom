# disp-0-raPGqd6aJpe9hHGnxMm 交付说明 —— 断言零张卡时没有徽章

**派单**:`disp-0-raPGqd6aJpe9hHGnxMm`(Lyra 发)
**提交**:`e0a1a2a`
**门禁**:380 tests / 31 files,tsc 净,eslint 135 errors = baseline,prettier 净,build 1.85s

---

## 1. 先复现,没有只信报告

```
NavRail.tsx:144  badge: pendingAcceptanceCount > 0 ? ... : undefined
                → 改成 >= 0
结果:15 passed
```

**存活,与 Lyra 报的一致。**

后果:`NavRail.tsx:232` 是 `{item.badge !== undefined && (...)}`,
于是一张卡都没有时,「派单」那一项上挂一个**永远的 0**。

## 2. 为什么这比噪音严重

`App.restartRestore.test.tsx` 自己的注释写着:

> Empty by default so the nav badge is absent unless a test says otherwise —
> a badge that is always there cannot prove anything.

**整套测试工具建立在"零张时徽章缺席"这个假设上,而这个假设本身没有任何断言。**

一个依赖未断言前提的套件,是可以被悄悄解除武装的 ——
而这次差点就是:那个 `> 0` 一旦变成 `>= 0`,我所有关于徽章的测试
(有卡时显示 2、活动项目不存在时仍显示 2)会**继续全绿**,
因为它们在"徽章存在"的前提下工作。

**Flux 说得准确:一个永远都在的徽章,不能传达任何信息。**

## 3. 补的断言

```ts
CANNED.cmd_list_acceptances = [];
mountApp();
await waitFor(() => expect(useDataStore.getState().projects.length).toBeGreaterThan(0));
await waitFor(() => expect(screen.getByTitle('派单')).toBeTruthy());
expect(screen.getByTitle('派单').textContent).not.toContain('0');
```

### 为什么按按钮自身文本,不按文档

第一版我写 `expect(screen.queryByText('0')).toBeNull()`,
它红了 —— **但不是因为正确的理由**:

```
→ Found multiple elements with the text: 0
```

屏幕上有很多个 0(inbox 徽章、各处计数器)。
**断言必须是关于"派单"这一项**,才能在 `> 0` 被改坏时因为正确的理由失败。
改成读那个按钮自己的 `textContent`。

## 4. 变异:两个都红

| # | 变异 | 结果 |
|---|---|---|
| 1 | `> 0` → `>= 0`(Lyra 跑的那条) | **1 红** —— `expected '0' not to contain '0'` |
| 2 | `badge: pendingAcceptanceCount`(完全去掉条件) | **1 红** |

恢复后 `16 passed (16)`,工作区干净。

**没有改 badge 语义** —— 派单明确说 `> 0` 是对的,要补的是钉住它的断言。

## 5. 上一单的两条,Lyra 已确认修好

她说:"你把名单改成从 `LIVE_TAB_BACKING` 推导(provenance.ts:114),
所以漂移在构造上不可能了。我往 BACKING 里单加一个标签 → 测试红。
**这是比「加一条双向断言」更彻底的修法 —— 你把两份名单合成了一份。**"

这条她复验过,我不再动。

## 6. 未做

- **推送失败** —— gitlab SSH 22 端口持续超时(重试 3 次),origin 一直无权限。
  提交在本地完整(`e0a1a2a`),工作树干净。
- **没有 merge into track**(派单 `do_not`)。
