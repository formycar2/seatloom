# 开卡:四件活(豁免门已核实打开)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-22-mira-open-cards-2-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-22 |
| 前提 | 同步完成、门已核实打开,只差卡 |

## 你上报的那条,Aegis 核实了

你在提交里写「门在源码开了、二进制还旧」。**这是对的,而且这个区分很重要。**

Aegis 的核实:
- track 源码里 `DELETION_EXEMPTIONS` 在 `dispatch.rs:1666`(结构体)与 `:1683`(常量)
- `cargo test` **89/89 过**,含
  `exempt_deletion_passes_while_the_rest_are_still_refused`
  与 `deletion_exemptions_are_explicit_audited_and_exact`

**所以那道门对 `AppV2.tsx` 已经开了。你可以开卡。**

(Aegis 中途一度用 `strings | grep` 判二进制里有没有清单,命中 0 就以为没合进来 ——
那是错的判据,release 二进制里符号被编译掉了。已改用跑测试确认。
这个坑 AGENTS.md 里早写着,我又踩了一次。)

## 你做得好的两点

- 同步后**落后 0**,且区间内没有任何文件落在 `ui/src` 或 `src-tauri`(按新判据干净)
- 合并带入 5 条测试失败,你修了前置条件而不是把断言改弱 —— 那是对的

## 开四张,分开开

每张都必须:`dispatch_id` 是自己的实现单;`flux_verdict_ref` **点名它自己**的验绿;
`how_to_verify` 写「做什么、看到什么」;`acceptance_plan` 有 steps + expected。

| 卡 | 实现单 | 验绿 | 要如实标注 |
|---|---|---|---|
| 三处编造数字(V5) | `disp-RWEf5ATPvICLkGtF__fEY` | pass | 沙箱可见(预算健康度"—"、同步时间"未知") |
| 沙箱标识条 | `disp-ZqvAuQ1GANGLx8Y8FLz2T` | pass | 沙箱窗口有橙条、主版没有 |
| 文档净化 | `disp-FzldOwJuxCfxc2TGq9dMw` | pass | **界面上看不到**(AppV2 从未挂载),要如实写「已修复但路径不可达」 |
| 重启回原位 | `disp-DFeH6Ut_aNfzIbuuR21V5` | **fail(旧)** | 你修到 246/246,但需 Flux 重验通过才开这张 |

## 验收

- 卡各自 dispatch_id 与 branch 对得上
- 每张 flux_verdict_ref 点名自己的实现单,不许互借
- 看不到的东西如实标注,不写成「已能用」
- 不许为求好过把验收条件写含糊

## 明确不做

- 不合成一张大卡
- 不合并(张先生点)
- 不改代码
