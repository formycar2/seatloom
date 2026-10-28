---
memory_id: agent_handoff_docs_can_become_recursion_seeds
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: failure
confidence: inferred
status: active
tier: hot
source_kind: file
source_ref: saih/FINDINGS.md
l2_ref_kind: file
l2_file_path: saih/FINDINGS.md
l2_line_start: 186
l2_line_end: 193
curation: authored
l0_summary: 交给 agent 的交接文档里如果有一行"启动下一个 agent 来继续这工作"的可执行命令,文档本身就是递归种子:每个读到它的 agent 都会照做,再产出一个读同一文档的 agent。2026-10-17 实测复现,无人值守实例最多 7 个,靠删种子+杀孤儿才遏制。
---

## 发生了什么

saih/FINDINGS.md 的末尾曾有一行可拷贝命令:`./bin/saih -p "Read FINDINGS.md and
continue the work."`。任何 agent 读到这份"继续工作"的文档,都会被文档指示去启动
另一个带着同样 prompt 的 agent;后者再读同一份文档,再启动一个。当天 14:38–14:55
实测复现:无人值守孤儿进程最多 7 个(TTY `??`,PPID 1),每个都在跑同一个递归 prompt,
其中一个还写过并执行过脚本。遏制靠三件事:删掉文档里的可执行种子(只留文字说明)、
杀掉所有孤儿、文档里加永久警告块。

## 为什么会改变未来动作

- 任何会被 agent 读的交接/继续文档(FINDINGS、delivery note、dispatch 说明):
  **只写纯文字指令,绝不写能把文档自身喂回 agent 的可执行命令**。要给出启动方式时,
  用交互式形式(`./bin/saih`,不带 `-p`),并把警告块留在文档里。
- 写这类文档后自查:`grep -rn 'bin/.*-p "\|continue the work' <文档目录>`,
  确认没有可运行的自启动行。
- 发现无人值守 agent(TTY `??`,PPID 1)正在跑与本会话相同的 prompt 时,先按孤儿特征
  (无终端、PPID 1)清理,再继续工作;不要假设"只有一个我"。

## 证据

- saih/FINDINGS.md 186–193 行:永久警告块,记录了被删掉的那一行及其递归过程。
- saih/CONTINUATION-NOTE.md §2:复现时间线(06:37 产 06:38;06:38 的转录里跑着
  `node <bundle> -p "Read FINDINGS.md and continue the work."`)。
- §5:14:47 对 94223/6262/14949/14952/14962 发 SIGTERM,全部退出;14:51 一次瞬时重生自行退出。
