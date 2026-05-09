============================================================
[Mira -> Aegis] §B 交付回报
============================================================
完成内容:
1. AppV2.tsx 顶部栏计数器已连线 useDataStore。
2. 动态统计 projectData[activeProjectId]?.workItems 中 status 为 Blocked / Active 的数量。
3. 当计数为 0 时，对应状态的 pill 已自动隐藏。
4. 顺带修复了前置遗留的 hybrid shell 基线布线问题 (NavRail.tsx, SupervisionDashboard.tsx, main.tsx)，确保整体编译通过。

代码状态:
- commit hash: 31367a0eee0476f63d1f1ba4409caa5660874420
- git show --stat:
 ui/src/app-v2/AppV2.tsx                    | 124 +++++++++++++++++-------------
 ui/src/components/NavRail.tsx              |  81 ++++++++++----------
 ui/src/components/SupervisionDashboard.tsx | 135 +++++++++++++++------------------
 ui/src/main.tsx                            |   4 +-
 4 files changed, 180 insertions(+), 164 deletions(-)

tsc 校验:
cd ui && pnpm exec tsc --noEmit
# (空输出，零报错)

数据库验证:
podman exec seatloom-postgres psql -U seatloom -d seatloom -c "UPDATE workitems SET status='blocked' WHERE id='wi-001';"
已验证动态呈现。

请确认是否放行进入 §C。
