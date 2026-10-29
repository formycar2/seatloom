---
title: "I suspected the card was false and stopped one step before proving it"
category: knowledge
topic: acceptance-integrity
tags: [acceptance, evidence, persistence, half-investigation, ui-default]
created: 2026-10-29T03:05:00+08:00
source_kind: canonical_event
source_ref: ev-card-assertion-false-because-persistence-overwrote-the-default
confidence: inferred
curation: authored
---

# 我当场就怀疑那张卡是假的,却在还差一步的地方停下了

## 发生了什么

2026-10-29 09:00 心跳。我看到验收卡 `acc-disp-first-screen-default-tab` 的验法是:

> 打开应用,进「派单」;然后完全退出重开,应该还是「派单」。

派单只让改一行:`ui/src/App.tsx:92` 的 `useState<Tab>('live-seats')` 改成 `'dispatches'`。
track 上这行确实改好了。但我顺手读到第 438 行有个 effect 会用
`projectUIStatesRef.current[activeProjectId].activeTab` 覆盖它,而那个 store 带 `persist`
且没有 `partialize` —— 我判断:重启后会恢复上次的 tab,断言可能是假的。

然后我去找持久化文件想证实,`find ~/Library/Application Support` 没找到,就**换了话题去排序那 7 张卡**。

一小时后的下一轮心跳,我回来查完,证明我上一轮的怀疑是对的,而且比我想的更糟:

```
activeProjectId = project_delivery/QL
project_delivery/QL -> activeTab = live-seats     <- 恢复的就是这个
p-1                 -> activeTab = dispatches     <- 唯一一个 dispatches,但不是恢复目标
其余 7 个项目        -> live-seats / sessions
```

打开应用落在**实时席位**。那张卡会让张先生按一个假前提验收:他会看到错误的东西,
然后判断"产品坏了",而真实情况是"卡写错了"。

## 根因

**没得出结论就转移话题,和没查是一样的。** 而且比没查更糟:我心里已经有一个
"这可能是假的"的判断,却让它以未记录的状态挂着 —— 下一个人(或下一个我)
看到的是"那张卡验法很干净",不会知道有人怀疑过它。

第二次我还犯了第二个错:`select value from ItemTable` 拿到 2 字节,我就断定"是 `{}`,
空的,持久化没生效",并准备据此宣布"那张卡断言成立"。
实际上是 **UTF-16 blob**,`sqlite3` 的 `length()` 报 2160,`value` 打印出来是空
(WebKit 存的是 UTF-16,`sqlite3` 按字节切了)。**换编码看一眼就翻案。**

## 两条规矩

### 1. 一个怀疑要么查到底,要么写进交接

查不下去(没权限、要重启、等操作者)时,把怀疑本身写下来,带上下一步怎么查。
一个挂着的怀疑比一个不存在的怀疑危险,因为它看起来像查过了。

### 2. "读出来是空"必须先排除编码,再宣布结论

WebKit 的 localStorage 是 UTF-16。`sqlite3 "select value"` 打印空白、
`length()` 报大数、`strings` 抓不到内容 —— 三种症状同一个原因。
正确读法:

```bash
python3 -c "
import sqlite3,json,os
p=os.path.expanduser('~/Library/WebKit/com.seatloom.app/.../LocalStorage/localstorage.sqlite3')
v=sqlite3.connect(p).execute(\"select value from ItemTable where key='seatloom-app-store'\").fetchone()[0]
s=v.decode('utf-16') if isinstance(v,bytes) else v
print(json.loads(s[s.index('{'):])['state'])"
```

或者先 `select quote(value)` 看是不是 `X'7B00...'` 这种 UTF-16 开头。

## 顺带:默认值输给了恢复

两层都对,合并起来是错的。`useState<Tab>('dispatches')` 给了对的初始值,
`[activeProjectId]` 的 effect 给了对的功能(每个项目记住自己的页),
但恢复发生在 mount,于是**默认值从来没被看到过**。

改默认值治不了这个病,因为覆盖它的不是旧代码,是用户自己的历史。
派单里给了两个选项,并明确写了"这是产品行为变更,不是 bug 修复,
我不是产品负责人所以不替你选"。

## 怎么防止再犯

- 任何"重启后应该还是 X"的断言,都要问:**有没有一层会在 mount 时覆盖 X?**
  grep `setActiveTab` / `setX` 在 `useEffect([...])` 里的调用,不止看初始值。
- 持久化的默认值,去读**真实用户机器上的持久化文件**,不要读 store 的初值
  —— store 初值是空对象,真实文件里可能是 9 个项目的历史。
