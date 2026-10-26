# 修:全局快捷键吞掉 shift 字符输入(问号打不出来)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-fix-shift-shortcut-eats-input-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |
| 严重度 | **高** —— 张先生实测无法输入 `?` 等 shift 字符 |
| 来源 | 张先生报的 bug |

## 根因(Aegis 已定位,你直接改)

`ui/src/hooks/useGlobalShortcuts.ts` 第 9-13 行:

```js
if ((event.key === '?' && event.shiftKey) || event.key === 'F1') {
  event.preventDefault();   // ← 吞掉了输入
  onShowHelp();
  return;
}
```

**macOS 上 Shift+/ 的 `event.key` 就是 `'?'`。** 于是每打一个问号就触发帮助弹窗,
`preventDefault()` 让那个字符进不了输入框。

**同一条链路还会吞别的**:任何走 `preventDefault()` 的全局快捷键,
在输入框里按下都会被抢走(用户打字时按 Cmd+K 也会跳命令栏而不是把字符留给输入)。

## 修法:快捷键要先看焦点在不在输入框

标准做法,三选一(Aegis 不指定,你按代码现状选最干净的):

1. 在处理函数开头判断焦点目标是不是可编辑元素:
   ```js
   const t = event.target as HTMLElement | null;
   if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
   ```
2. 或用 `event.isComposing` 一并排除输入法组合态(中文输入时尤其要)
3. Cmd/Ctrl 组合键可保留(它们是修饰键,不产生字符),但**裸 Shift 字符必须放过**

**判据**:`?` 这类单字符快捷键,焦点在输入框/编辑区时**必须**让字符进去,
不该弹帮助。

## 验收(可重跑)

1. 焦点在任意输入框时按 Shift+/ → 输入框里出现 `?`,**不**弹帮助
2. 焦点**不在**输入框时按 `?` 或 F1 → 仍然弹帮助(这个行为别弄丢)
3. Cmd/Ctrl+K 命令栏、Cmd/Ctrl+` 终端,两个都还能用
4. 中文输入法组合态(如打拼音时)不触发任何快捷键
5. `pnpm test` 与 `pnpm typecheck` 仍全绿(新测试覆盖上面 1-4)

## 明确不做

- 不删帮助功能,只是别在打字时抢输入
- 不改其它快捷键的键位
- 不加新的快捷键
