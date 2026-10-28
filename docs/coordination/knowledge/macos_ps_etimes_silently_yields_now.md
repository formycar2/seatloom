---
memory_id: macos_ps_etimes_silently_yields_now
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: failure
confidence: inferred
status: active
tier: warm
source_kind: file
source_ref: scripts/fleet-process-census.sh
l2_ref_kind: file
l2_file_path: scripts/fleet-process-census.sh
l2_line_start: 33
l2_line_end: 46
curation: authored
l0_summary: macOS 的 ps 没有 etimes,而且它把"关键字清单"打到 stdout;awk 把非数字静默当 0,于是每个进程的起始时间都算成"现在"。写舰队普查时差点因此把四个旧镜像席位全判成最新。
---

## 发生了什么

写 `scripts/fleet-process-census.sh` 时,用 `ps -o etimes= -p <pid>` 取进程已运行秒数,
再用 `date +%s` 反算起始时间。在 macOS 上这个关键字不存在 —— 但**错误信息走的是
stdout**:`ps` 打出 "keyword not found" 加一整屏合法关键字清单,
`2>/dev/null` 拦不住。管道后面的 `awk '{print n-$1}'` 把每一行都当成输入,
`$1` 不是数字时 awk **静默按 0 算**,于是输出 7 行 `NOW`。

后果:每个进程的起始时间都变成"现在",`date -r` 报 usage 错误,
而新旧镜像的判据(`进程起始时间 < 二进制 mtime`?)直接失效 ——
四个 10-10 起的旧镜像席位会被判成"当前镜像"。
**这是一个每次都成功、只是结果是垃圾的检查**,比报错危险得多。

## 为什么会改变未来动作

- macOS `ps` 没有 `etimes`。要起始时间就用 `ps -o lstart= -p <pid>`
  (形如 `Sat Oct 10 17:06:08 2026`)配 `date -j -f "%a %b %d %H:%M:%S %Y"` 转 epoch。
- **任何"算出来的时间"都要校验是纯数字**:`case "$e" in ''|*[!0-9]*) 未知;; esac`。
  校验不过就报"未知"并计入需处理,绝不让垃圾值参与比较。
- 这条是 AGENTS.md「每一项检查都必须能报错」的实例:
  `awk` 对非数字输入不报错,所以"管道能跑通"不等于"取到了值"。

## 证据

- `ps -o etimes= -p 57836` → `ps: etimes: keyword not found` + 关键字清单(在 stdout 上)
- 修复前 `bash scripts/fleet-process-census.sh` 的输出:`line 40: 1791740283\n...×7: syntax error`
- 修复后同一命令正确报出 mira/nimbus/flux/onyx 起于 10-10(旧镜像)、lyra/vela 起于 10-11 18:05(当前镜像)
