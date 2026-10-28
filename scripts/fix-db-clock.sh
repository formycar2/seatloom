#!/usr/bin/env bash
# fix-db-clock.sh — 把 podman 虚拟机的时钟拨回真实时间。
#
# 问题:数据库(跑在 podman 虚拟机里)的时钟比真实时间慢约 137 分钟。
#
# 哪边是对的 —— 已有独立第三方证据:
#   Cloudflare 的 HTTP Date 头与宿主机 `date -u` **精确到秒一致**,sntp 查 time.apple.com 偏差 0.04 秒。
#   所以**宿主机是对的,虚拟机慢了**。
#   (苹果官网的 Date 头慢 9 分钟,那是 CDN 缓存响应,不能用来对时。)
#
# 为什么第一版没修好:第一版只跑 `chronyc makestep`,它返回 200 OK 但一秒都没动 ——
# 虚拟机里的 chronyd **认为自己是准的**(`chronyc tracking` 自报偏差 3 毫秒),
# 算出的偏差是 0,所以没有东西可步进。典型的虚拟机休眠后时钟停摆:
# 宿主机睡过/重启过,虚拟机时钟跟着停,chrony 的参考数据还是停摆前的。
#
# 所以这一版改成**直接设时间**,并在之后重启 chronyd 让它重新建立参考。
#
# 为什么要修:账本里每条新事件的 occurred_at / started_at 都会错 2 小时 17 分。
# 双时间线(occurred_at + created_at)是这套账本的纪律之一,时钟错了这条纪律就是空的。
#
# 为什么要脚本:`podman machine ssh sudo ...` 被权限分类器拦住了(拦得对,那是 sudo)。
#
# 不停服务:只调时钟,**不重启虚拟机、不重启数据库、不动任何数据**。
# 副作用:now() 会**向前**跳约 137 分钟(不是向后,所以不会出现时间倒流)。
# 正在飞的领单会突然显得老了 2 小时 —— 这是安全的:
# 叫醒一个已经 in_flight 的席位,那单就是它当前的任务。
#
# 用法: bash scripts/fix-db-clock.sh
set -uo pipefail

dbnow(){ podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom \
           -tAc "select floor(extract(epoch from now()))::bigint" 2>/dev/null | tr -d ' '; }
dbstr(){ podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom \
           -tAc "select to_char(now() at time zone 'UTC','MM-DD HH24:MI:SS')||' UTC'" 2>/dev/null; }
skew(){ local h d; h=$(date -u +%s); d=$(dbnow)
        [ -z "${d:-}" ] && { echo ""; return 1; }
        local s=$(( h - d )); [ "$s" -lt 0 ] && s=$(( -s )); echo "$s"; }

report(){ # report 标签
  echo "$1:"
  echo "  宿主机(已验证为准)  $(date -u '+%m-%d %H:%M:%S UTC')"
  echo "  数据库                $(dbstr)"
  local s; s="$(skew)" || { echo "  ❌ 数据库连不上"; return 1; }
  echo "  偏差                  $(( s / 60 )) 分 $(( s % 60 )) 秒"
}

report "修之前" || exit 1
before="$(skew)"
echo

if [ "${before:-0}" -le 60 ]; then
  echo "✅ 偏差已在 1 分钟内,无需处理。"
  exit 0
fi

echo "第一步 — 直接把虚拟机的系统时间设成宿主机的时间(不重启任何服务)"
TARGET="$(date -u '+%Y-%m-%d %H:%M:%S')"
podman machine ssh "sudo date -u -s '${TARGET}'" 2>&1 | sed 's/^/  /'
sleep 2
echo

echo "第二步 — 重启 chronyd,让它基于新时间重新建立 NTP 参考"
echo "         (不重启它的话,它仍抱着停摆前的参考数据,过一会可能把时间拽回去)"
podman machine ssh 'sudo systemctl restart chronyd && sleep 3 && sudo chronyc makestep' 2>&1 | sed 's/^/  /'
sleep 3
echo

report "修之后" || exit 1
after="$(skew)"
echo

# 判定必须能报错:只有偏差真的落到 1 分钟内才算成功
if [ "${after:-99999}" -le 60 ]; then
  echo "✅ 对齐了(差 ${after} 秒)。账本的时间戳从现在起是真实时间。"
  echo
  echo "顺带检查:时钟跳了之后,正在飞的领单会显得更老 —— 这是预期的,不是卡住。"
  podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom -tAc \
    "select '  '||target_seat_id||' 领单已 '||round(extract(epoch from (now()-started_at))/60)||' 分钟' from dispatches where status='in_flight';" 2>/dev/null
  exit 0
else
  echo "❌ 还差 ${after} 秒 —— 两步都没生效。把这段贴给 Aegis,不要自己重试。"
  echo "   仅剩的办法是重启虚拟机,但那会停掉数据库和整个舰队,所以让 Aegis 先判断。"
  exit 1
fi
