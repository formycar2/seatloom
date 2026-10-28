# 独立验 Nimbus 的两个修复(合完了但没人独立验过)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-flux-verify-two-nimbus-fixes-v1 |
| target_seat | seat-flux-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |

## 为什么派这一单

两个 Nimbus 的修复已经合并进 `track/infra-foundation`,但**没有任何独立验证**:

- `disp-VWK5z1k7Jcu8-iQFrGVrb` —— tauri bundle(判据修正 + favicon 404)
- `disp-NMcTvureVxwPh_3bFAGxx` —— `~/networkorchestrator` 假路径

账本里只有 Nimbus 自己的 `SeatDispatchDelivered/Completed`,
**没有 Flux 的 verdict,也没有验收卡**。

**Aegis 在合并时自己跑了编译和测试,但那不是独立验证。**
合并前的 build 只证明"能编过",不证明"修对了、修的是那个问题"。

## 要验的两件事

### 1. tauri bundle(`disp-VWK5z1k7Jcu8-iQFrGVrb`)

Nimbus 的结论:**这个 bug 不存在**。`Contents/Resources/` 只有 `icon.icns` 是
Tauri v2 的正常行为 —— 前端 brotli 压缩后 `include_bytes!` 嵌进可执行文件。

**你要独立判断这个结论对不对**,可以自己解压二进制比对,别只读他的交付。

他还指出 Aegis 原来的判据坏在两处:
- `grep -rho isComposing <Resources>` —— 压缩内容 grep 不到,假阳假阴都有
- `isComposing` / `proposeDimensions` **来自 `@xterm/addon-fit`,不是 SeatLoom 的代码** ——
  用它们证明的是"xterm 打进去了",不是"我们的代码打进去了"

他给的替代判据:`scripts/verify-tauri-bundle.sh`(解压 blob + 逐字节比 +
报哪些串来自 xterm 哪些来自 SeatLoom)。他实测三个对照:
自己的 app+自己的 dist PASS;张先生的 app+主仓 dist PASS;
张先生的 app+自己的 dist FAIL(证明能分辨两份不同构建)。

**验这个**:跑那个脚本,判断它的对照设计是否真的能分辨(换一份 dist 会不会 FAIL)。

### 2. 假路径(`disp-NMcTvureVxwPh_3bFAGxx`)

Nimbus 定位:`ui/src/lib/dto-to-v1.ts:153` 原来是 `path: '~/' + d.name` ——
前端拿不到真路径就自己造了一个。来历:2026-10-16 张先生看到三条写死的假项目卡,
当时用"假卡路径是 `~/Documents/...`,而 mapper 合成 `~/<名字>`"当**识别水印**,
水印活干完了没拆。

修法:把账本的 `home_path` 一路接过来(DB 查询 → 行模型 → DTO → UI 类型 → mapper),
三处显示改成 `{project.path ?? '无 home_path'}`。

**你要验**:
- 修完之后,那一行显示的是不是账本里真的 `home_path`(不是合成)
- 没有 `home_path` 的项目,显示的是不是「无 home_path」而不是空/猜测
- 那个测试真的会红吗(他说把代码改回 `'~/' + d.name`,2 个测试 FAIL)

## 判据(可重跑,给实际输出)

1. `scripts/verify-tauri-bundle.sh` 的对照设计能不能分辨两份不同构建
   (拿一份不一样的 dist 跑,它必须 FAIL)
2. 假路径那一行现在显示什么、值来自哪个字段
3. 那两个测试在旧代码上会不会红

## 明确不做

- 不许读一遍交付说明就下 verdict —— 要自己跑
- 不许为了让判据过而改判据
- 不许把"我复现不了"当 verdict。张先生看到过 `~/networkorchestrator`,
  那就有来源;如果你复现不了,说出你试了什么
- 如果你的结论是"Nimbus 也错了",直接说,别客气 —— 他这轮推翻过 Aegis 的错判,
  你也可能推翻他的
