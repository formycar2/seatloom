# 沙箱窗口内常驻一条版本标识条

| Field | Value |
|---|---|
| id | AEGIS-2026-10-21-sandbox-build-banner-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-21 |
| baseline | track/infra-foundation |
| 张先生裁定 | 2026-10-21:「能否给所有沙箱启动的页面有个显著的区别,加上每个实例版本区别」 |

## 为什么需要第二层

图标那一层已由 Aegis 做好(橙框 + 右上角橙块),它解决的是 **Dock 和 Cmd-Tab**
认得出。但**打开之后,两个窗口内容一模一样** —— 图标在 Dock 里,窗口里看不见。

所以还需要一层:**窗口内一条常驻的标识条**,写清:

- 这是沙箱还是主版
- 装的是哪个分支、哪个提交
- 何时构建

**判据:张先生不用离开窗口、不用切 Dock,就知道自己看的是哪一份。**

## 一个必须承认的约束

`localStorage` 四个键里三个没有命名空间(见
`docs/coordination/knowledge/ui_cannot_launch_app_yet_and_two_holes_must_not_stack.md`),
沙箱版与主版同时开会互抢。**标识条用的信息不能依赖前端自己存的东西**,
否则两个窗口会显示同一条。请用下述后端注入的方式。

## 怎么知道"我是沙箱"

不改数据库(那是被权限封顶的事),用**环境变量 + 构建期注入**:

- 沙箱包由 Aegis 的 `scripts/prepare-sandbox-build.sh` 安装和启动,
  启动时会带 `SEATLOOM_SANDBOX=1`
- 主版从不带这个变量

所以判据是:**进程有没有 `SEATLOOM_SANDBOX`**。有 → 沙箱;没有 → 主版。

Tauri 侧读法:在 `src-tauri/src/commands/` 加一条命令(不新增能力,
**只读一个已存在的环境变量**,不启动任何进程),把这三个值返回给前端:

```rust
pub struct RuntimeIdentityDto {
    pub is_sandbox: bool,      // env SEATLOOM_SANDBOX 是否为 "1"
    pub branch: Option<String>, // env SEATLOOM_SANDBOX_BRANCH
    pub commit: Option<String>, // env SEATLOOM_SANDBOX_COMMIT
    pub built_at: Option<String>, // 构建时间,来自构建期注入
}
```

`branch`/`commit`/`built_at` 由安装脚本注入(Aegis 会在
`prepare-sandbox-build.sh` 里补上 `SEATLOOM_SANDBOX_BRANCH` 等变量),
**没有就显示"未记录",不要编**。

## 前端做什么

1. 启动时调这条命令,**失败要说失败**(不许因为读不到就假装是主版)
2. 是沙箱时,在窗口顶部(或底部,你定,但必须**常驻**)渲染一条标识条:
   - 底色与主版区分(建议橙色系,与图标一致)
   - 文字:`沙箱 · 分支 packet/mira/knowledge-panel · 提交 abc1234 · 构建于 10-21 13:20`
   - 缺哪项就写对应项"未记录"
3. **是主版时不渲染这条** —— 主版要干净,不能为了对称给主版加个"我是主版"的条子
4. 这条不能被用户操作关掉。它是身份,不是提示

## 验收

- 同时开主版和沙箱:**沙箱有那条,主版没有**
- 条上的分支/提交与安装时的参数一致
- 后端命令读不到时,界面显示"未记录"或明确报错,**不显示猜测值**
- 关闭后端命令(比如临时改坏)时,前端要能说出失败,不许静默当成主版
- 类型检查、测试、构建通过;测试不许恒真

## 明确不做

- 不动 `launch-seatloom.sh`(那是 Aegis 的脚本,我会自己在
  `prepare-sandbox-build.sh` 里注入环境变量)
- 不给主版加对称标识条
- 不改数据库、不启动任何进程、不开新权限
- 不自己开验收卡、不合并

## 交付

`docs/coordination/tasks/mira/MIRA-2026-10-21-sandbox-build-banner-v1.md`
