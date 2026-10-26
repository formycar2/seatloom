# 开卡:shift 字符修复(`disp-Ksm1oC3fHNRnH2zumBN_p`)— 已开,并更正我自己一个错判

| Field | Value |
|---|---|
| dispatch | `disp-Ksm1oC3fHNRnH2zumBN_p` |
| packet | `AEGIS-2026-10-26-mira-open-shift-card-v1` |
| 分支 | `packet/mira/knowledge-panel` |
| 日期 | 2026-10-26 |
| 结论 | **卡已开**(`acc-disp-fCOD_8QOby_Bi20NfrsyR`,pending operator decision)。**并且:我上一份交付说明里有一句是错的,已在此更正** |

## 一、开卡前核了什么(不凭派单描述信)

| 核什么 | 怎么核 | 结果 |
|---|---|---|
| 验绿存在且 pass | `query_ledger` ReviewVerdictIssued + `ref_type=dispatch, ref_id=disp-fCOD_8QOby_Bi20NfrsyR` | `ev-FLUX-SHIFT-VERDICT-CORRECTED-01`,`verdict: pass`,`payload.dispatch_id` 点名我的实现单 ✓ |
| 验绿点的确实是**我的**实现单 | 同上,读 payload 而不是派单转述 | 点名 `disp-fCOD_8QOby_Bi20NfrsyR`,且 refs 里该 dispatch 是 `subject` ✓ |
| 分支含 base | `git merge-base --is-ancestor track/infra-foundation HEAD` → 0 | 合并后 ✓ |
| 区间内容(不看落后数字) | `git log --name-only` 区间 | 只有 3 个 Aegis 文档(`correct-verdict...`/`liveseats-scroll...`/`verify-shift-fix...`),**无 `ui/src` 或 `src-tauri` 改动**,合并安全 ✓ |

**验绿用的是「更正版」那条**(`ev-FLUX-SHIFT-VERDICT-CORRECTED-01`),不是它替换掉的
`ev-rKtx-DQARUF142u6lN3x5`。原因是 Flux 自己更正的:旧那条用了非契约键
`implementation_dispatch`,契约键是 `payload.dispatch_id`。派单点名用更正版,我用它。

## 二、更正:我上一份交付说明里「Cmd+K 在输入框内也不触发」是错的

Flux 在验绿里记了 **F1(P2)**:

> `useGlobalShortcuts` is guarded, but `ui/src/App.tsx:183-190` installs a
> window-level Cmd+K listener with no typing guard, which still fires and calls
> preventDefault with focus inside a real input.

我上一份交付说明写的是「Cmd/Ctrl+K 与 Cmd/Ctrl+\` 在输入框内也不触发」。
**前半句对 Cmd+K 不成立。** 我没有只信 Flux,自己实测了(把 `App.tsx:183` 的判定
条件逐字转写成探针,不动源码):

```
App.tsx 监听回调被调? true
defaultPrevented?      true
```

**确认:焦点在输入框时按 Cmd+K,App.tsx 那条监听仍然 `preventDefault()` 并切命令栏。**
我错在只枚举了 `useGlobalShortcuts` 一个文件就下了结论,没去核 `App.tsx:192`
那条并行的监听。派单里我写过「没动 App.tsx:192 的 Cmd+K(它只管 Cmd/Ctrl 组合,
不吃裸字符)」—— 「不吃裸字符」这句是对的(Cmd+K 不产生字符,所以问号那条 bug
确实只有 `useGlobalShortcuts` 一个成因,Flux 也这么确认),但我不该由此推出
「输入框内也不触发」。

**影响范围**:只影响我交付说明里那句话,不影响已开的卡 —— 卡里第 3 步我**一开始就
限定「焦点不在输入区」**,并在 preconditions 第 3 条写明了这处残留。所以卡没说错。

## 三、卡上写了什么

- `dispatch_id` = `disp-fCOD_8QOby_Bi20NfrsyR`(我的实现单)
- `flux_verdict_ref` = `ev-FLUX-SHIFT-VERDICT-CORRECTED-01`(点名它的那条 pass)
- `branch` = `packet/mira/knowledge-panel`,`base` = `track/infra-foundation`
- `how_to_verify` 四条按派单写(输入框内 Shift+/ 出 ? 不弹帮助 / 焦点在外 ? 或 F1
  仍弹帮助 / Cmd+K 与 Cmd+\` 照旧 / IME 组合态不触发),第 3 步带焦点限定
- `acceptance_plan` steps 四条 + expected 四条,`preconditions` 四条如实标注

**如实标注的**(派单明确要求,没省):

1. **真机窗口未验**。Flux 明确说了他没渲染完整 App、没驱动真 Tauri 窗口。
   四条行为是 hook 级单测 + jsdom 真实 DOM 覆盖。没写成「已真机验证」。
2. **「? 进入输入框」是代理断言**。jsdom 不模拟字符插入,断言的是
   `defaultPrevented === false` + 事件确实到达输入框,不是真实按键结果。
3. **Cmd+K 在输入框内仍会触发**(F1/P2,`App.tsx:183`)。不是本卡范围,
   是否要修请 Aegis 裁决。
4. **14 个既有 unhandled error** 与本卡无关(改动前后数量一致)。

## 四、我没做什么

- **没有顺手把 F1 一起修掉再开卡**。分支一动就越过验绿的 `e7e327e`,
  交上去的就是没验过的代码。F1 要修得先派单、修完、验绿,再开卡。
- 没有自己批准自己,没有合并
- 没有改代码,没有动 `App.tsx:183`
- 没有把「未真机验证」这句话省掉,没把判据写含糊

## 五、需要 Aegis 裁的

1. **F1:输入框内的 Cmd+K 要不要修?** Flux 定为 P2(不吃字符,只是抢焦点),
   并说「whether Cmd+K should work while typing is a product ruling for Aegis」。
   我上一单也问过同一个问题,还没得到裁决。**在我拿到裁决前,我不会自己动它** ——
   它决定的是产品行为,不是我的交付方式。
2. 若裁「要修」:请派单,我修 `App.tsx:183` 那条(与 `useGlobalShortcuts` 用同一个
   判据),再请 Flux 验,然后开卡。不建议并进现在这张。

## 复现命令

```bash
# 验绿(点名我的实现单、pass、契约键)
#   query_ledger ReviewVerdictIssued ref_type=dispatch ref_id=disp-fCOD_8QOby_Bi20NfrsyR
#   -> ev-FLUX-SHIFT-VERDICT-CORRECTED-01  payload.dispatch_id=disp-fCOD_8QOby_Bi20NfrsyR  verdict=pass

# 分支含 base、区间内无界面/Rust 改动
git merge-base --is-ancestor track/infra-foundation HEAD   # 0
git log --name-only --format= packet/mira/knowledge-panel..track/infra-foundation

# F1 仍存活(逐字转写 App.tsx:183 的判定条件,不动源码):
cd ui
#   input.focus(); dispatch KeyboardEvent('keydown',{key:'k',metaKey:true})
#   -> 回调被调 true,defaultPrevented true
rg -n "addEventListener\('keydown'" src --glob '!*.test.*'
#   useGlobalShortcuts.ts:82(有守卫)/ App.tsx:192(无守卫,Cmd+K)
#   SupervisorCommandBar.tsx:80(isOpen 门控)/ SupervisorPanel.tsx:251(Escape,不 preventDefault)
```
