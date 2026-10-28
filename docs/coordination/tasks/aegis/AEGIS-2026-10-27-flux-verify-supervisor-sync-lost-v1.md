# 独立验 supervisor 订阅失败处理(`disp-cLore9rXage5HKVn6Hsbd`)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-flux-verify-supervisor-sync-lost-v1 |
| target_seat | seat-flux-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 为什么派

这个派单**改了产品代码**(`ui/src/App.tsx` + 新组件),不只是加测试。
Mira 自己验了 8 条测试 + 5 个变异,但**席位的自证不算验证**。

## 她的修法(你要独立判断对不对)

Mira 的核心判断:`supervisorDetached` 是个 boolean,`false` 的含义是
"面板在主窗口里"—— 那是个**肯定句**。而订阅断掉后真实情况是"我不知道了"。
状态冻结在 `false` 时,界面在用自信的语气说一件它已经不知道的事。

所以她加的**不是**"把状态改成某个值",而是一条独立状态 `supervisorSyncLost` ——
一个是答案,一个是答案的可信度。界面上多一条黄条。

她还修了一个 Aegis 没要求的漏洞:`listen()` 还在 pending 时组件卸载,
unlisten 存进没人读的闭包 → **监听器泄漏**,页面存活期间一直挂着。
解法是加 `cancelled` 标志。

## 要验的

1. **她的核心判断成立吗?** —— 状态冻结在 `false` 真的会造成她说的那两个后果吗
   (⌘K 再开一份面板 / 面板回不来)?去看 ⌘K 和内嵌面板渲染怎么读这个状态
2. **`supervisorSyncLost` 这个设计有没有问题?** ——
   比如:启动查询失败(它有自己的 `.catch`)和订阅失败,两条路径的表现一致吗?
   如果启动查询也失败,界面显示什么?
3. **泄漏那个修法对吗?** `cancelled` 标志会不会漏掉别的路径
   (比如两个订阅只有一个失败、或者卸载后又重新挂载)
4. **测试真的守得住吗?** 她的 5 个变异(M1 一律 `.catch(() => {})`、
   M2 只给一边加、M3 去掉 `cancelled`、M4 `console.error` 不上界面、
   M5 strip 无条件显示)是不是都真能杀死。Aegis 已经亲手复现了 M1(RED 4),
   你验其余的,或者验出 Aegis 复现错了

## 判据

- 给出你跑的检查/变异和实际输出
- 明确说 verdict:pass 还是 fail,以及为什么

## 明确不做

- 不许只读她的交付说明就下 verdict
- 不许为了让 verdict 变绿而改判据
- 如果她的设计有更好的替代,说出来 —— 但**不要顺手改**,
  那是另一个派单
