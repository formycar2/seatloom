---
id: a_seat_branch_must_not_be_verified_until_it_is_synced_to_track
source_kind: canonical_event
source_ref: disp-bBdpBqTT9g5-kKU28xvIt
session_scope: seatloom
title: 席位分支落后主干时不能开卡验收 —— 沙箱会显示一份假的旧现状
domain: coordination
modality: failure
status: active
curation: authored
confidence: inferred
owner: aegis
created_by: aegis
created_at: 2026-10-21
supersedes: null
evidence_refs:
  - packet/mira/knowledge-panel
  - 2c69adb
---

## 发生了什么

2026-10-21 张先生在沙箱里看到「所有 Seat 不分 project 平铺显示」,报告为缺陷。

三层都查过,全是对的:
- 数据层:`ql_delivery` 1 席、`seatloom` 7 席,绑定表正确
- 命令层:`cmd_list_seats` 强制按项目查,不给 project_id 直接报错
- 过滤层:`boundMirrorNames` 逻辑正确,在 track 上命中 2 处

**但 `packet/mira/knowledge-panel` 落后 track 19 个提交,
项目过滤的 `2c69adb` 不在它上面,`boundMirrorNames` 命中 0。**

**所以他看到的不是功能坏,是版本旧。** 而这份旧代码是我让他装的 ——
为了让他在沙箱里验净化单。**我拿了一份缺了 19 个提交的代码,
让他去看现在的产品。他会以为新功能是坏的,进而打回一件本来正确的活。**

## 判据(以后照着做)

1. **开卡前必查 `git rev-list --count <branch>..track/infra-foundation`**,
   非 0 就不开卡 —— 合进去等于把产品往回拖。
2. **装沙箱前必查同一项**,落后就明确告诉操作者「沙箱会缺这些提交里的功能,
   那不等于它们坏了」。(`prepare-sandbox-build.sh` 已内置这个检查。)
3. **让席位起分支时说明基线**,且验收前重新核一次 —— 分支起出之后主干会前进,
   起的时候是干净的,验的时候可能已经落后。

## 为什么会 overlooked

派单时只看了「这条分支上有没有那件事的提交」,**没看它缺不缺别的东西**。
一个分支可以是「有 A、缺 B」,而 B 恰好是产品当前的关键行为。

## 冲突处理的默认取向

同步时除该席位自己负责的范围外,**按主干赢**。别把主干的修复覆盖回旧版 ——
那正是这次差点发生的事(19 个提交里有项目过滤,一旦被覆盖回去,
切项目时的数据串台会静默复发)。
