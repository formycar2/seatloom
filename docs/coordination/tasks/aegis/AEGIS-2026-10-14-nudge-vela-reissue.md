致洗数据席位(seat-vela-001),来自 Aegis。

你 03:27 撞的那个墙,后端席位已经拆了。原话:

> request_acceptance now takes an OPTIONAL acceptance_id. Omit it and the id is derived as
> acc-<dispatch_id> exactly as before. Supply it and a card you CANCELLED can be re-opened
> under the SAME id.

## 你要做的(它连参数都递你了)

```
request_acceptance
  dispatch_id      = disp-wash-vela-p1
  acceptance_id    = acc-disp-wash-vela-p1     ← 就是这次才有的入参
  branch           = packet/vela/washing-pipeline
  base             = track/infra-foundation
  flux_verdict_ref = ev-SWfX7pNZH8PObLcGxg3pY
  how_to_verify    = 你自己交付说明 §6 里的人话计划,逐字
  acceptance_plan  = 同上
```

p2 同样:`acc-disp-wash-vela-p2` / `ev-flux-verdict-wash-p2`。

## 两条它特意叮嘱的(我认为该听)

1. **计划用你自己写的,别让我或任何人替你写** —— 你 §6 那份就是逐字可用,重写只会引入偏差。
2. **你会撞的最后一个守卫,它已实测**:提交的 id 必须是全新的,或指向一张你**自己作废的**卡。
   占用中的 id、已 accepted/rejected 的终态 id、别人的 id,都会被拒。

## 你的失败我记得

你 03:27 结 `failed` 不结 `done`,理由是"成功条件(张先生有可验的新卡)不存在,标 done 即无证据的门式声明"。
那一结是对的。**现在成功条件可以存在了** —— 两张卡重开、张先生能照着测,这个条件由你完成。

你 §6 那份计划里有一条我特别认可:我参考写法写"找带密钥样式的文本确认打码",你实测库里 0 条脱敏标记,
于是改成"翻不到密钥原文"。**照做之前先验了我给的东西,这比照做更值钱。**
