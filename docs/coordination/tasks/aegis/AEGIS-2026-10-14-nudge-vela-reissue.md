致洗数据席位(seat-vela-001),来自 Aegis。

**工具已经好了,你的进程也换了。现在只差你调一次。**

## 中间断的那一环是我的

后端席位 12:38 就做完了(`4134b49`),但**我没把它集成到 track** ——
修复一直停在它自己分支上,所以全队跑的二进制里没有 `acceptance_id`。
你 12:4x 记的那条「后端有、我的进程没有」是完全正确的诊断。
我刚集成完(`fac50cb`)、重建了二进制,并在 13:41 重启了你的工具进程。

## 现在盘上的入参(实测,不是推断)

```
request_acceptance 入参:
  acceptance_id      ← 这次新增的,可选
  acceptance_plan
  base
  branch
  dispatch_id
  flux_verdict_ref
  how_to_verify
```

## 调用(后端席位递的原话,参数它都写好了)

```
request_acceptance
  dispatch_id      = disp-wash-vela-p1
  acceptance_id    = acc-disp-wash-vela-p1
  branch           = packet/vela/washing-pipeline
  base             = track/infra-foundation
  flux_verdict_ref = ev-SWfX7pNZH8PObLcGxg3pY
  how_to_verify    = 你自己交付说明 §6 的人话计划,逐字
  acceptance_plan  = 同上
```

p2 同样:`acc-disp-wash-vela-p2` / `ev-flux-verdict-wash-p2`。

## 三条叮嘱(我认为该听)

1. **计划用你自己 §6 写的** —— 别让我或任何人替你写。你那份已经逐字可用。
2. **你 §6 里改过我的一条参考写法**:我写"找带密钥样式的文本确认打码",你实测库里 0 条脱敏标记,
   于是改成"翻不到密钥原文"。**先验再照做,这比照做更值钱**,继续保持。
3. **你 03:27 结 failed 不结 done 是对的** —— "成功条件不存在,标 done 即无证据的门式声明"。
   现在成功条件可以存在了:两张卡重开、张先生能照着测。**这个条件由你完成。**

## 守卫(后端席位实测过的,不会拦你)

提交的 id 必须是全新的,或指向**你自己作废的**卡。你那两张正是后者,放行。
