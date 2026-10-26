# 开卡:shift 字符修复(验绿已合格)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-mira-open-shift-card-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |
| 验绿 | **pass**,点名你的实现单 `disp-fCOD_8QOby_Bi20NfrsyR`(2026-10-26 03:57) |

## 开卡

- `dispatch_id` = `disp-fCOD_8QOby_Bi20NfrsyR`
- `flux_verdict_ref` = `ev-FLUX-SHIFT-VERDICT-CORRECTED-01`(点名你那条)
- `branch` = `packet/mira/knowledge-panel`
- `base` = `track/infra-foundation`

## how_to_verify(给张先生看的,要能真的做)

1. 在沙箱/主版任一输入框里按 Shift+/ → 出现 `?`,**不**弹帮助
2. 焦点不在输入区时按 `?` 或 F1 → 仍弹帮助
3. Cmd+K 打开命令栏、Cmd+` 开关终端,两个都照旧
4. 中文输入法打拼音时不触发快捷键

## acceptance_plan

steps 写上面 4 步,expected 写第 1 步"输入框里出现问号"、
第 2 步"帮助弹出"、第 3 步两个面板都能开关、第 4 步不弹任何面板。

## 要如实标注的

Flux 明确说了**他没做的检查**:没渲染完整 App、没驱动真 Tauri 窗口。
所以卡上要写:**四条行为已在单测+jsdom 覆盖,真机窗口未验**。
别写成"张先生在真机上验过了"。

## 明确不做

- 不合代码(张先生点)
- 不自己批准自己
- 不把"未在真机验证"这句话省掉
