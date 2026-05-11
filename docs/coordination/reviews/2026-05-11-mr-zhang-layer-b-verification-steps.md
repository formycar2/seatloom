# Mr. Zhang Layer B Verification Steps — B1 + 008 Combined (2026-05-11)

| Field | Value |
|---|---|
| template | T4 |
| subtype | verification_checklist |
| id | 2026-05-11-mr-zhang-layer-b-verification-steps |
| status | issued |
| author | aegis |
| date | 2026-05-11 |
| to | mr-zhang |
| verifies | B1 (bidirectional tmux) + 008 (project_id schema) |
| tags | layer-b, runtime-verification, step-by-step |

---

## 前置条件

1. 确保 PostgreSQL 正在运行：
   ```bash
   pg_ctl -D /opt/homebrew/var/postgresql@17 status
   ```
   如果未运行：`brew services start postgresql@17`

2. 确保 tmux 会话 `Onyx-data-seatloom` 存在：
   ```bash
   tmux ls | grep Onyx-data-seatloom
   ```
   如果不存在：
   ```bash
   tmux new-session -d -s Onyx-data-seatloom
   ```

3. 确保代码在最新 commit `83b1223` 或之后：
   ```bash
   cd ~/Documents/GitHub/seatloom
   git log --oneline -1
   ```
   应该看到 `83b1223 feat(ui): mount SessionsWorkspace as 会话 tab in V1 NavRail` 或更新的 commit。

---

## 第一部分：Mira Sessions Tab Mount (3 步)

### 步骤 1：启动 SeatLoom

```bash
cd ~/Documents/GitHub/seatloom
pnpm tauri dev
```

**预期结果**：主窗口打开，显示 V1 界面（左侧 NavRail + 项目切换器 + 8 个 tab）。

**验证点**：
- [ ] 主窗口成功打开
- [ ] 左侧 NavRail 可见
- [ ] 顶部项目切换器显示 "SeatLoom Project"

**如果失败**：截图 + 错误信息发给 Aegis。

---

### 步骤 2：检查 NavRail 会话 tab

在左侧 NavRail 中，从上到下查看所有 tab 图标。

**预期结果**：在 "剧本" (Playbook) tab 之后，应该看到一个新的 tab：
- 图标：Terminal 图标（类似 `>_` 或方框内有命令行符号）
- 标签：会话

**验证点**：
- [ ] 会话 tab 可见
- [ ] 图标是 Terminal 图标
- [ ] 标签文字是 "会话"

**如果失败**：截图发给 Aegis。

---

### 步骤 3：点击会话 tab，检查 SessionsWorkspace 渲染

点击左侧 NavRail 的 "会话" tab。

**预期结果**：
- 主内容区域切换到 SessionsWorkspace 界面
- 左侧边栏显示 "Attach to tmux" 区域
- 下拉框显示 "-- Select a tmux session --"
- 下方有 "Attach" 按钮（灰色禁用状态）
- 右侧主区域显示空状态提示："无附加会话。从左侧下拉框选择一个 tmux 会话并点击 Attach。"

**验证点**：
- [ ] SessionsWorkspace 界面渲染成功
- [ ] 左侧边栏 "Attach to tmux" 区域可见
- [ ] 下拉框可见
- [ ] Attach 按钮可见（禁用状态）
- [ ] 右侧空状态提示可见

**如果失败**：截图 + 浏览器 DevTools Console 错误信息发给 Aegis。

---

## 第二部分：B1 Bidirectional Tmux (6 步)

### 步骤 4：下拉框列出真实 tmux 会话

点击 "-- Select a tmux session --" 下拉框。

**预期结果**：
- 下拉框展开
- 列表中显示 `Onyx-data-seatloom`（以及其他正在运行的 tmux 会话，如果有）

**验证点**：
- [ ] 下拉框展开
- [ ] `Onyx-data-seatloom` 在列表中

**如果失败**：
1. 在终端运行 `tmux ls` 确认会话存在
2. 截图下拉框内容
3. 发给 Aegis

---

### 步骤 5：Attach 到 Onyx-data-seatloom

1. 在下拉框中选择 `Onyx-data-seatloom`
2. 点击 "Attach" 按钮

**预期结果**：
- 右侧主区域出现一个新的 tab，标签为 "tmux: Onyx-data-seatloom"
- Tab 上有绿色圆点（表示活跃）
- Tab 内容区域显示蓝色横幅："ℹ️ Bidirectional mode (v0.0.2). Keystrokes in this terminal are forwarded into the tmux pane. Attach-only: closing the tab leaves the underlying tmux session running."
- 横幅下方是黑色终端区域，显示 tmux pane 的当前内容

**验证点**：
- [ ] 新 tab 出现，标签正确
- [ ] 绿色圆点可见
- [ ] 蓝色横幅可见，文字正确
- [ ] 终端区域可见，显示 tmux pane 内容

**如果失败**：截图 + DevTools Console 错误信息发给 Aegis。

---

### 步骤 6：测试键盘输入转发（bidirectional write）

在 SeatLoom 终端区域内点击，确保焦点在终端上，然后输入：

```
echo "hello from seatloom"
```

按 Enter。

**预期结果**：
- 在 SeatLoom 终端中看到 `echo "hello from seatloom"` 命令
- 按 Enter 后，看到输出 `hello from seatloom`

**验证点**：
- [ ] 输入的字符实时出现在 SeatLoom 终端中
- [ ] 按 Enter 后命令执行，输出可见

**交叉验证**（可选，但推荐）：
在另一个终端窗口运行：
```bash
tmux attach -t Onyx-data-seatloom
```
应该看到相同的 `echo "hello from seatloom"` 命令和输出。按 `Ctrl-b d` 退出 tmux attach（不要杀死会话）。

**如果失败**：
1. 检查输入是否出现在 SeatLoom 终端
2. 检查是否出现在 tmux pane（用 `tmux capture-pane -t Onyx-data-seatloom -p` 查看）
3. 截图 + 发给 Aegis

---

### 步骤 7：测试 Ctrl-C 转发

在 SeatLoom 终端中输入：

```
sleep 100
```

按 Enter，然后立即按 `Ctrl-C`。

**预期结果**：
- `sleep 100` 命令开始执行
- 按 `Ctrl-C` 后，命令被中断
- 终端显示 `^C`，然后返回 shell 提示符

**验证点**：
- [ ] `sleep 100` 命令启动
- [ ] `Ctrl-C` 中断命令
- [ ] 返回 shell 提示符

**如果失败**：
1. 检查 `Ctrl-C` 是否被浏览器捕获（某些浏览器会拦截）
2. 尝试在 tmux pane 中手动 `Ctrl-C`，确认 tmux 会话本身正常
3. 截图 + 发给 Aegis

---

### 步骤 8：测试粘贴（paste-buffer path）

复制一段文本到剪贴板，例如：

```
echo "paste test from clipboard"
```

在 SeatLoom 终端中右键 → 粘贴（或 `Cmd-V` / `Ctrl-V`，取决于 xterm.js 配置）。

**预期结果**：
- 粘贴的文本出现在 SeatLoom 终端中
- 按 Enter 后命令执行

**验证点**：
- [ ] 粘贴的文本出现
- [ ] 命令执行成功

**如果失败**：
1. 检查粘贴是否被浏览器阻止（某些浏览器需要权限）
2. 尝试手动输入相同命令，确认输入路径正常
3. 截图 + 发给 Aegis

---

### 步骤 9：测试 R3 invariant — 关闭 tab 不杀死 tmux 会话

在 SeatLoom 中，点击 "tmux: Onyx-data-seatloom" tab 右侧的 `×` 按钮，关闭 tab。

**预期结果**：
- Tab 从 SeatLoom 中消失
- 右侧主区域回到空状态："无附加会话。从左侧下拉框选择一个 tmux 会话并点击 Attach。"

**验证点**：
- [ ] Tab 关闭
- [ ] 空状态提示重新出现

**交叉验证**（必须）：
在终端运行：
```bash
tmux ls | grep Onyx-data-seatloom
```

**预期结果**：
- `Onyx-data-seatloom` 仍然在列表中，状态为 `(attached)` 或 `(detached)`，但**不是** `(dead)`

**验证点**：
- [ ] `Onyx-data-seatloom` 仍然存在
- [ ] 状态不是 `(dead)`

**如果失败**：
1. 如果 `tmux ls` 中看不到 `Onyx-data-seatloom`，说明 SeatLoom 错误地杀死了会话（R3 违规）
2. 截图 + `tmux ls` 输出发给 Aegis

---

## 第三部分：008 Schema Migration (4 步)

### 步骤 10：检查 Inbox 视图渲染

在 NavRail 中点击 "收件箱" (Inbox) tab。

**预期结果**：
- Inbox 视图渲染成功
- 显示收件箱项目列表（可能为空，或显示种子数据）

**验证点**：
- [ ] Inbox 视图渲染
- [ ] 无 JavaScript 错误

**如果失败**：截图 + DevTools Console 错误信息发给 Aegis。

---

### 步骤 11：检查 WorkItems 视图渲染

在 NavRail 中点击 "工作项" (WorkItems) tab。

**预期结果**：
- WorkItems 视图渲染成功
- 显示工作项列表（可能为空，或显示种子数据）

**验证点**：
- [ ] WorkItems 视图渲染
- [ ] 无 JavaScript 错误

**如果失败**：截图 + DevTools Console 错误信息发给 Aegis。

---

### 步骤 12：检查 Sessions 视图渲染（再次）

在 NavRail 中点击 "会话" (Sessions) tab。

**预期结果**：
- SessionsWorkspace 再次渲染成功（与步骤 3 相同）

**验证点**：
- [ ] SessionsWorkspace 渲染
- [ ] 无 JavaScript 错误

**如果失败**：截图 + DevTools Console 错误信息发给 Aegis。

---

### 步骤 13：检查 Handoffs 视图渲染

在 NavRail 中点击 "交接" (Handoffs) tab（如果存在）。

**预期结果**：
- Handoffs 视图渲染成功
- 显示交接项列表（可能为空，或显示种子数据）

**验证点**：
- [ ] Handoffs 视图渲染
- [ ] 无 JavaScript 错误

**如果失败**：截图 + DevTools Console 错误信息发给 Aegis。

---

## 验证完成后

将所有验证点的结果（✓ PASS 或 ✗ FAIL）汇总，发给 Aegis。格式：

```
第一部分（Mira Sessions Tab Mount）：
- 步骤 1: PASS
- 步骤 2: PASS
- 步骤 3: PASS

第二部分（B1 Bidirectional Tmux）：
- 步骤 4: PASS
- 步骤 5: PASS
- 步骤 6: PASS
- 步骤 7: PASS
- 步骤 8: PASS
- 步骤 9: PASS

第三部分（008 Schema Migration）：
- 步骤 10: PASS
- 步骤 11: PASS
- 步骤 12: PASS
- 步骤 13: PASS
```

如果所有步骤 PASS，Aegis 将：
1. 将 B1 从 PROVISIONAL → UNCONDITIONAL
2. 将 008 从 PROVISIONAL → UNCONDITIONAL
3. 开启 SG-B stage gate 决策

如果任何步骤 FAIL，Aegis 将根据失败类型派发修复 packet。

---

*Prepared by Aegis · 2026-05-11 · Step-by-step runtime verification for Mr. Zhang*
