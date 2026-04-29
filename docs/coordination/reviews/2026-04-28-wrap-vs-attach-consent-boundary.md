# Wrap vs Attach：为什么“可治理审批”只在 Wrap 才能保证（中文说明）

| 项目 | 内容 |
|------|------|
| 文档 | Wrap vs Attach 审批边界说明 |
| 状态 | Draft — Pending Lyra Review |
| 作者 | Aegis |
| 日期 | 2026-04-28 |
| 目的 | 解释 CLI 场景下 wrap/attach 的本质区别，以及为什么“权限/确认”治理只能在 wrap 可靠实现 |

---

## 0. 一句话结论

- **Wrap**：SeatLoom 处在 I/O 与执行路径的“控制点”上（父进程/PTY owner），因此可以实现**可强制、可审计、可复现**的审批治理。
- **Attach**：SeatLoom 是“后来者/旁观者”，最多能观察与记录，**无法保证拦截与审批**。

这不是“看不看得到屏幕”的区别，而是**SeatLoom 是否处在动作执行链路的必经之路**。

---

## 1. 你看到的现象：在 tmux 里都一样，为什么本质不同？

如果 cc/codex/opencode/gemini 都只是 tmux 里跑的 CLI，看上去是一样的。但底层有两种完全不同的连接方式：

### 1.1 Wrap（SeatLoom 启动并托管）

```
SeatLoom 创建 PTY
  → fork/exec 启动 agent 进程
  → SeatLoom 读写该 PTY 的 stdin/stdout
```

SeatLoom 是父进程/PTY 的拥有者（owner）。这意味着：

- SeatLoom 可以**暂停/恢复 stdin**（卡住输入通道）
- SeatLoom 可以**记录每一次输入输出**
- SeatLoom 可以在运行前注入环境（PATH/shim/MCP 配置等），把“危险动作”导向可治理层

### 1.2 Attach（附着到外部已运行进程）

```
tmux 已创建 PTY 并启动 agent
  → agent 绑定在既定 PTY 与环境上运行
  → SeatLoom 事后尝试附着/观察/复用输出
```

SeatLoom不是该 PTY 的唯一控制者，通常也无法成为“动作必经之路”。因此：

- 你无法保证“每一次确认都必须先经过 SeatLoom”
- 你无法保证“危险命令不会被直接执行”
- 你很难在不重启进程的情况下改变它的 PATH/代理执行路径

---

## 2. “申请权限/做确认”其实有两类，Wrap 只对其中一类有机会做可靠治理

### 2.1 A 类：自然语言确认（文本层）

例子：Agent 在终端输出：

- “要不要继续？”
- “我准备执行 rm -rf，你确认吗？”

本质：这只是文本提示。**能否强制治理取决于谁控制 stdin**。

- **Wrap 能做的**：SeatLoom 可检测到确认点后**暂停 stdin**，弹出 SeatLoom 的确认 UI；用户允许后，再把 `y/Enter` 送回 PTY。
- **Attach 很难保证**：如果 stdin 仍由原终端/用户直接控制，SeatLoom 看到了文本也没法阻止用户已经按下 `y`。

> 重要：如果“检测确认点”靠正则解析自然语言，将非常不可靠。要可靠，需要 agent/harness 输出结构化标记（例如 `__SEATLOOM_APPROVAL_REQUEST__{...}`）。Wrap 有机会强制这种协议；Attach 很难 retrofit。

### 2.2 B 类：真实权限/能力边界（执行层）

例子：

- 写文件、删除文件
- 执行 shell 命令
- 网络访问（下载依赖、调用外部 API）
- sudo / 权限提升

本质：这是“动作是否经过你能拦截的执行层”。

- **Wrap 才有可能**把动作导向可拦截层：例如通过 shim（`seatloom exec`）、受控 PATH wrapper、MCP 工具代理等。
- **Attach 基本做不到**：进程已启动，环境已固定，SeatLoom很难在不重启的情况下把执行路径改成“必须先过 policy gate”。

---

## 3. 关键澄清：Wrap 也不是“魔法”，必须把审批点放在工程可控层

即使是 Wrap，SeatLoom 也不能“凭肉眼看输出”就可靠治理一切。要把审批做成可靠产品能力，必须工程化把审批点放在可控层：

- **可控层 1：工具调用层**（MCP tools / SeatLoom tool runner）
- **可控层 2：命令执行层**（shim/代理执行：`seatloom exec`，或受控 PATH）
- **可控层 3：文件系统层**（至少做路径白名单 + diff 预览；更强是沙箱/权限隔离）

不要把治理建立在“Agent 会用自然语言问你要不要继续”这种不可控假设上。

---

## 4. 产品合同应该如何写：Guaranteed vs Best-effort

为避免实现阶段误解，PRD 必须把能力边界写死：

### 4.1 Guaranteed（仅 Wrap 模式）

- 每个高风险动作都能触发 SeatLoom 的 policy gate
- 统一确认 UI（Allow once / session / project / deny）
- Ledger 审计：谁在何时因何放行了什么动作
- 可复现：同类动作遵循同一策略，不依赖运行时“心情/文本表达”

前提：危险动作必须走 SeatLoom 的可控执行层（shim/MCP/代理）。

### 4.2 Best-effort（Attach 模式）

- 可以观察、记录、回放、生成 Playbook
- **不保证拦截与审批**
- 当用户需要治理时，SeatLoom 应提示迁移到 Wrap（或 MCP Host）路径

---

## 5. 对当前设计的直接影响（必须落到交互与架构）

1. UI 中明确标注 Session 的启动方式：`wrapped` / `attached`
2. Consent/Policy Gate 只在 `wrapped` session 上提供“保证语义”
3. 对 `attached` session：提供提示与迁移引导（例如“一键重启为 wrap，并从最近 checkpoint 构建 LaunchPack”）
4. Ledger 记录所有审批：`consent.requested` / `consent.granted` / `consent.denied`（带 scope）

---

## 6. 结论

Wrap 与 Attach 的本质区别是 SeatLoom 是否处在“控制点”上：能否成为动作执行链路的必经之路。  
因此，“可治理审批”只能在 Wrap 模式做到可靠与可审计；Attach 模式只能 best-effort 记录与提示。

---

*本文件建议由 Lyra 纳入 PRD v0.5：作为 Consent Layer 的 guaranteed/best-effort 合同条款，并在 interaction-spec/ux-spec 中补足 wrapped/attached 的 UI 与流程。*

