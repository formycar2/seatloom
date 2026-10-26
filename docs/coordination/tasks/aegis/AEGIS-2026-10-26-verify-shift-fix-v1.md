# 验绿:Mira 的 shift 字符修复(问号打不出来)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-verify-shift-fix-v1 |
| target_seat | seat-flux-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |
| 验的对象 | `disp-fCOD_8QOby_Bi20NfrsyR`(Mira 的实现单,提交 `e7e327e`) |
| 分支 | `packet/mira/knowledge-panel` |

## 你验什么(可重跑)

Mira 的实现:`ui/src/hooks/useGlobalShortcuts.ts` 加了 `isTypingContext()`
(输入法组合态 / INPUT / TEXTAREA / contenteditable 内,快捷键全部让位),
测试 `ui/src/hooks/useGlobalShortcuts.test.ts` 13 条。

**Aegis 已实测**:在她分支上 13/13 过、`pnpm typecheck` 干净。
但**Aegis 只跑了测试,没看测试是不是真的在测那个 bug** —— 这是你要做的。

1. **测试有没有真的覆盖 bug**:把 `isTypingContext` 的判断临时注掉,
   那 13 条里应该有**若干条变红**(红的是真在测这个 bug)。
   如果注掉了还全绿,说明测试是装饰,要报出来。
2. **四条行为**(用真实 DOM,不是只跑单测):
   - 焦点在 input 内按 Shift+/ → 出现 `?`,不弹帮助
   - 焦点不在输入区按 `?` 或 F1 → 仍弹帮助
   - Cmd+K / Cmd+` 仍能打开
   - IME 组合态不触发快捷键
3. **有没有回归**:`pnpm test` 全量仍绿
4. **有没有偷偷放宽**:检查 diff,不许出现"把快捷键整个删掉"这类省事改法

## 判据怎么写

verdict 必须点名 `disp-fCOD_8QOby_Bi20NfrsyR`(实现单),不是你自己这张派单。

## 明确不做

- 不改代码(发现问题写进 verdict,让 Mira 修)
- 不因为她自己说 13/13 就放过 —— 第 1 条就是要防这个
