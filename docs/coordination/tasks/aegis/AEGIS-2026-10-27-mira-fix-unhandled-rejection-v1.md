# 两个订阅没有 `.catch()` —— 你自己上报的,现在派回去修

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-mira-fix-unhandled-rejection-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 来源

你在 `disp-B4Y5oAohMQwtXm5Vva1fw` 的交付说明第六节上报:

> `App.tsx:160/166` 两个订阅没有 `.catch()`。我是在测试侧补的 mock,
> 产品代码里的承诺仍然悬空:如果 Tauri 的 `listen()` 真的失败
> (不是测试环境),那是个 unhandled rejection,而不是界面上一句话。

**你明确说了"要改也该单独派单"。现在派。**

## 现状(Aegis 已核对)

`ui/src/App.tsx` 里,同一个 effect 内三种写法不一致:

```ts
.supervisorWindowStatus()          // 有 .catch(() => {})
  .then((s) => setSupervisorDetached(s.detached))
  .catch(() => {});

onSupervisorDetached(() => {       // 没有 .catch()
  ...
}).then((fn) => { offDetached = fn; });

onSupervisorReembedded(() => {     // 没有 .catch()
  ...
}).then((fn) => { offReembed = fn; });
```

## 要你做的

给这两个订阅补上失败处理。**但要想清楚"失败了界面上该发生什么",**
不要一律 `.catch(() => {})` 吞掉 —— 上面那个 `supervisorWindowStatus()`
就是这么吞的,而"吞掉"和"让用户知道"是两种不同的决定。

分情况说清:

- 订阅失败时,supervisor 分离状态**应该显示成什么**?
- 这个 effect 已经有个 cleanup(`return () => { if (offDetached) ... }`),
  失败时 cleanup 还要成立吗?
- 如果 `listen()` 失败是因为 Tauri 不在(非 Tauri 环境),
  和"Tauri 在但事件总线坏了",应该区别对待吗?

## 判据

- 两个订阅都有失败处理,而且**处理方式有理由**(不是照抄上面那个 `{}`)
- 写测试证明失败路径被走到(注入一个 reject 的 listen,断言界面/状态的反应)
- 说明为什么选这个反应,不选"吞掉"

## 明确不做

- 不许一律 `.catch(() => {})` 然后说"修好了" —— 那只是把 unhandled rejection
  变成静默失败,问题从"会报错"变成"不报错但状态是假的"
- 不许顺手改别的订阅/effect
- 不许为了让测试过而改测试的注入方式
