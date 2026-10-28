# disp-SI-Qrw0EXuoYLVdKpIoIB 交付说明 —— 四件开卡,开出 1 件,另 2 件无合格 verdict

**派单**:`disp-SI-Qrw0EXuoYLVdKpIoIB`(Aegis 发,优先级 6)
**提交**:无代码改动(仅本文件)

---

## 1. 结果

| 派单 | 派单指定的 verdict | 我核到的 | 卡 |
|---|---|---|---|
| `disp-session-key-ui` | `ev-verdict-disp-session-key-ui` | pass、**点名自己**、Flux 发的 | **上一轮已开**,现为 pending,未重开 |
| `disp-first-screen-default-tab` | `ev-verdict-first-screen` | ❌ 那条**不点名它**;但我另找到 `ev-FLUX-AUDIT-GATE-A-DEFAULT-TAB` = pass、**点名自己**、Flux 发的 | ✅ **已开** |
| `disp-blocked-on-whom-ui` | `ev-verdict-first-screen` | ❌ **4 条 verdict 提及它,0 条点名它** | ❌ 未开 |
| `disp-provenance-badge-honest` | `ev-verdict-integration-round-v2` | ❌ **2 条 verdict 提及它,0 条点名它**(pass 那条点名 `disp-integration-round-verify-v2`) | ❌ 未开 |

**开出 1 件,未开 2 件,1 件早已开过。**

---

## 2. 派单指定的 verdict 有三处不点名目标

派单很仔细,连"provenance 那件不要用 fail 那条"都写明了。但它用的查法是:

```sql
select count(*) from acceptances where flux_verdict_ref = <verdict id>;
-- 四件全是 0
```

**这个查法回答"这条 verdict 被几张卡引用",回答不了"这条 verdict 判的是哪件活"。**
后者要看 `payload.dispatch_id`。逐条对:

| 派单 | 派单指定的 verdict | 该 verdict 的 `payload.dispatch_id` | 点名目标? |
|---|---|---|---|
| `disp-session-key-ui` | `ev-verdict-disp-session-key-ui` | `disp-session-key-ui` | **YES** |
| `disp-first-screen-default-tab` | `ev-verdict-first-screen` | `disp-first-screen-verify` | NO |
| `disp-blocked-on-whom-ui` | `ev-verdict-first-screen` | `disp-first-screen-verify` | NO |
| `disp-provenance-badge-honest` | `ev-verdict-integration-round-v2` | `disp-integration-round-verify-v2` | NO |

**注意 `ev-verdict-first-screen` 被派单同时指派给两件** —— 它只能点名一个派单,
所以至少有一件必然不合规。这不是巧合,是同一个根因。

---

## 3. 我多找了一条派单没提的合格 verdict

派单给 `disp-first-screen-default-tab` 指定的是 `ev-verdict-first-screen`(不合格)。
我把它名下**所有** verdict 列了一遍,5 条:

```
ev-verdict-integration-round-v2    pass  dispatch_id=disp-integration-round-verify-v2  NO
ev-verdict-first-screen            pass  dispatch_id=disp-first-screen-verify          NO
ev-mira-audit-flux-VH6l1H02        pass  dispatch_id=disp-VH6l1HixlUVYuy4m0NEp-        NO
ev-FLUX-AUDIT-GATE-A-DEFAULT-TAB   pass  dispatch_id=disp-first-screen-default-tab    YES  ← 这条
ev-verdict-integration-round       fail  dispatch_id=disp-integration-round-verify    NO
```

`ev-FLUX-AUDIT-GATE-A-DEFAULT-TAB`(10-26,Flux,pass)点名它自己。我核过 actor 是
`seat:seat-flux-001`,而作者是我 —— 第三方成立。**用这条开的卡。**

## 4. `disp-session-key-ui` 我没重开

它已经有卡了(`acc-disp-session-key-ui`,pending,上一轮我开的)。
派单的查法(`count(*) ... = 0`)是在**它开卡之前**跑的,现在不是 0 了。

重开会造成什么取决于工具行为,但无论哪种都不是我该做的:那张卡已经在操作员面前,
撤了重开只会让他看到一张卡消失又出现。

## 5. 另两件为什么没开

### 5.1 `disp-blocked-on-whom-ui`(依赖栏显示被指物是否存在)

提及它的 4 条 verdict,**0 条点名它**:

```
ev-verdict-integration-round-v2  → disp-integration-round-verify-v2
ev-verdict-first-screen          → disp-first-screen-verify
ev-pnBnPX1pdaRysj8UYW077         → disp-ySsxkvHxKurEgKpt7Q_Hm
ev-verdict-integration-round     → disp-integration-round-verify (fail)
```

### 5.2 `disp-provenance-badge-honest`(徽章依赖读取)

提及它的 2 条,**0 条点名它**:

```
ev-verdict-integration-round-v2  → disp-integration-round-verify-v2   (pass,派单指定的)
ev-verdict-integration-round     → disp-integration-round-verify      (fail,派单叫我别用)
```

**这两件的活确实被 Flux 验过** —— `FLUX-2026-10-28-verify-integration-round-v2-v1.md`
第 76-90 行审了 provenance 徽标那个缺陷(十条对抗断言、结论 pass),
`disp-first-screen-verify` 的 payload 也把"依赖栏被指物是否存在"列进了攻击面。

**但 verdict 的 `dispatch_id` 字段指向的是验单自己,不是被审的实现派单。**
这是账本字段与实际验证范围不一致,不是没验过。

---

## 6. 我在守卫里看到这个形状正在扩散

跑 Aegis 的 `check-verdict-refs.sh` 时:

```
good=5 bad=3
  BAD  acc-disp-b5VbRor1fHhP3nN4YZI_l   verdict names disp-ySsxkvHxKurEgKpt7Q_Hm
  BAD  acc-disp-ibABeNk-nkEDAkI6Qj2sG   verdict names disp-ySsxkvHxKurEgKpt7Q_Hm
  BAD  acc-disp-reemit-stalled-announcements
       verdict_ref=ev-verdict-integration-round-v2
       the verdict does not name THIS dispatch - it names disp-integration-round-verify-v2
```

**`acc-disp-reemit-stalled-announcements` 犯的是我上一轮一模一样的错** ——
用了派单指定的 `ev-verdict-integration-round-v2`,而它点名的是
`disp-integration-round-verify-v2`。那是 Nimbus 的卡。

另两张是**同一条 verdict 被两张卡引用**,而它只能点名一个派单。

**这不是我在纠正别人,是在说这个形状有传染性**:当一个派单说"用这条 verdict"、
而这条 verdict 其实不合格时,照着做的人就会开出坏卡。我上一轮就是这么错的。

**该修的不是各个席位,是那个查法。** 派单用的
`select count(*) from acceptances where flux_verdict_ref=<id>` 给出 0 只说明
"没被卡引用",不说明"可以引用"。Aegis 落地的 `check-verdict-refs.sh` 是对的判据,
但它是在卡开出来之后才跑。

---

## 7. 我给 Aegis 的三个建议

1. **开卡派单里不要指定 verdict id,指定 dispatch id 就好。** 让开卡的席位自己
   去找"点名这个 dispatch 的那条 pass verdict"—— 找不到就报告,而不是拿到一条
   就用的。指定 id 反而把错误具体化了。
2. **`check-verdict-refs.sh` 应该在开卡前跑,不是只进心跳。** 它已经能判 BAD,
   现在的顺序是"开出来 → 心跳发现 → 撤 → 重开",而我这一轮就是这么绕的。
3. **那 3 张 BAD 卡需要各自的席位处理** —— 尤其是
   `acc-disp-reemit-stalled-announcements`,它引用了一条我上一轮刚刚撤掉的
   同一条 verdict。

---

## 8. 门禁

零代码改动。未跑门禁 —— 没有代码变动,跑出来是与上一次完全相同的数字。

---

## 9. 顺带:一次 push 失败

上一轮往 `origin`(github.com/formycar2/seatloom)推被拒:

```
Please make sure you have the correct access rights
and the repository exists.
```

`gitlab` 推成功了。`ENGINEERING_STANDARDS` 第 49 行要求"push to both remotes",
但 GitHub 镜像我推不上去。**这一条我报告,不自己绕** —— 换远端、改配置、
或者绕过规范都属于我该先问的那类动作。
