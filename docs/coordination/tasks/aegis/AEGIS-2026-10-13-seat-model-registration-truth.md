# 席位模型登记:把假值改成真值

## 问题

`seats.default_model` 对五个席位记的是 `claude-opus-5[1m]` —— Claude Code 时代的模型名,
**不是 pi 的 `provider/model` 格式**。而它们实际跑的是 `stepplan-anthropic/step-5-preview`。

也就是说:账本记着一个它们没在跑、而且根本不是合法 pi 格式的值。

## 为什么没炸

`scripts/seat-launch.sh` 的 `model_for()` 校验形状:没有 `/` 的值一律回退到
`$MODEL`,并在启动行里写明来源。所以席位起得来、跑的是对的,只是账本是假的。
10-11 那次五个席位全被这个旧值拦下,脚本注释里写着"第一版漏了这个校验,会把五个席位修坏"。

## 修法

按 `.seatloom/seat-sessions.tsv` 里**每个席位最后一次启动记录**取真值:

| 席位 | 修正前(假) | 修正后(真) |
|---|---|---|
| flux / lyra / mira / nimbus / onyx | `claude-opus-5[1m]` | `stepplan-anthropic/step-5-preview` |
| vela | `stepplan-wash-anthropic/step-5-preview` | 不变(本就是合法 pi 格式,且是刻意选的无限额) |
| aegis | `stepplan-anthropic/step-5-preview` | 不变(10-12 切换时已修) |

修正后重启 Onyx 验证:模型来源从「默认(席位表登记的不是 pi 格式,已忽略)」
变成「seats.default_model」—— 不再依赖回退,降级从此是看得见的。

## 还没定的(需要人判断,不是我能定的)

这五个席位现在跑的是**回退默认值**,不是按席位刻意选的值。脚本注释明确警告过:
写死一个值给所有席位,会在重启时静默把席位降级 —— 洗数据席位登记无限额
`stepplan-wash-anthropic`,存在的意义就是不必为省 token 降低质量。

所以真正该问的是:**质量验证席位(它的活是逐条判据验交付)该不该也用无限额?**
我现在只把账本改成了真话,没有代替任何人决定配额。
