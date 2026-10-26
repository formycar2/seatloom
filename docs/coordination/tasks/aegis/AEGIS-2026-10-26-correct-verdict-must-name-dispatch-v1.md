# 更正验绿:必须点名实现单(shift 修复)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-correct-verdict-must-name-dispatch-v1 |
| target_seat | seat-flux-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |
| 为什么 | 你 03:40 出的那条验绿是 `pass`,但**payload 里没有 dispatch_id** |
| 涉及 | 实现单 `disp-fCOD_8QOby_Bi20NfrsyR`(Mira 的 shift 字符修复,提交 `e7e327e`) |

## 规矩(张先生定过,不是我新加的)

验绿必须**点名它验的那个实现单**,不能只写"验的是我这单"。
否则下游无法证明这条 verdict 属于哪件活,等于可以互借。

## 你要做的

发一条**更正事件**(账本不可改,只能追加),内容:

- `dispatch_id` = `disp-fCOD_8QOby_Bi20NfrsyR`(点名 Mira 的实现单)
- `verdict` = pass
- 把你实际验过的内容重述一遍(特别是:
  你有没有把 `isTypingContext` 注掉、看测试是否真的变红?
  那条如果做了,更正里要写实际结果;如果没做,就写"未做",
  **别在更正里补一个你没做过检查**)

## 判据

- 新事件的 payload 里能读到 `dispatch_id = disp-fCOD_8QOby_Bi20NfrsyR`
- verdict 与你 03:40 那条一致(还是 pass,不许为了好看改)
- 如实写哪些检查真做了、哪些没做

## 明确不做

- 不改你原来的 verdict 结论
- 不补做检查然后当成当时做过(那是伪造)
- 不动代码
