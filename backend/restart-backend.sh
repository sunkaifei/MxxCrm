#!/usr/bin/env bash
# =============================================================================
# 后端重启串行化脚本 —— 并行会话安全的 taskkill/build/start 一体化入口
#
# 背景：多会话各自 taskkill→cargo build→Start-Process 会互相踩：
#   1) 链接器覆盖 exe 时被对方拉起的进程锁住（os error 5）；
#   2) 对方守护循环在 kill 后几秒内拉起旧 exe，继续占锁；
#   3) 一方代码写到一半导致全仓编译失败，服务无人拉起。
#
# 本脚本用 backend/.restart.lock 目录锁把"杀-建-启"串行化：
#   - 先把 exe 改名为 .prev 让出路径：守护循环拉不起旧二进制，链接器也不再被锁
#     （Windows 允许改名运行中的 exe）；
#   - 编译失败自动回滚启动旧版本，服务不裸奔；
#   - 锁带时间戳，超过 STALE 秒视为死锁强制接管，不会永久卡死。
#
# 用法：bash backend/restart-backend.sh   （成功返回 0，8088 已监听）
# 约定：所有会话重启后端一律走本脚本，禁止自行 taskkill+build+start。
# =============================================================================
set -u
cd "$(dirname "$0")" || exit 1

EXE="target/debug/mxx-crm.exe"
LOCK=".restart.lock"
LOCK_TIMEOUT=900   # 等锁上限（秒），超时放弃以免堆积
STALE=1200         # 锁存活超过该秒数视为遗留死锁，强制接管

# ---------- 1. 抢互斥锁（mkdir 原子性） ----------
start_ts=$(date +%s)
while ! mkdir "$LOCK" 2>/dev/null; do
  now=$(date +%s)
  ts=$(cat "$LOCK/ts" 2>/dev/null || echo 0)
  if [ $((now - ts)) -gt $STALE ]; then
    echo "[restart] 发现超过 ${STALE}s 的遗留锁，强制接管"
    rm -rf "$LOCK"
    continue
  fi
  if [ $((now - start_ts)) -ge $LOCK_TIMEOUT ]; then
    echo "[restart] 等锁超时（${LOCK_TIMEOUT}s），放弃本次重启"; exit 1
  fi
  echo "[restart] 其他会话正在重启，等待锁..."
  sleep 3
done
echo $$ > "$LOCK/pid"
date +%s > "$LOCK/ts"
trap 'rm -rf "$LOCK"' EXIT

# ---------- 2. 让出 exe 路径（防守护循环拉旧版 + 防链接器被锁） ----------
if [ -f "$EXE" ]; then
  rm -f "$EXE.prev" 2>/dev/null
  if ! mv -f "$EXE" "$EXE.prev" 2>/dev/null; then
    echo "[restart] exe 改名失败，路径仍被占用，请手工检查"; exit 1
  fi
fi

# ---------- 3. 停掉在跑实例 ----------
taskkill //IM mxx-crm.exe //F >/dev/null 2>&1
sleep 1

# ---------- 4. 编译（失败则回滚旧版并拉起，保证服务在线） ----------
if cargo build > build_restart.log 2>&1; then
  rm -f "$EXE.prev"
  echo "[restart] 编译成功"
else
  echo "[restart] 编译失败（常见原因：并行会话代码未写完），已回滚启动旧版本"
  grep -A6 "^error" build_restart.log | head -30
  [ -f "$EXE.prev" ] && mv -f "$EXE.prev" "$EXE"
fi

# ---------- 5. 启动新进程（工作目录与日志路径按 backend/ 动态解析） ----------
EXE_WIN=$(cygpath -m "$PWD/target/debug/mxx-crm.exe")
DIR_WIN=$(cygpath -m "$PWD")
powershell -NoProfile -Command "Start-Process -FilePath '$EXE_WIN' -WorkingDirectory '$DIR_WIN' -WindowStyle Hidden -RedirectStandardOutput '$DIR_WIN/stdout.log' -RedirectStandardError '$DIR_WIN/stderr.log'"

# ---------- 6. 等端口就绪 ----------
for _ in $(seq 1 20); do
  sleep 2
  if netstat -ano 2>/dev/null | grep -q ":8088 .*LISTENING"; then
    echo "[restart] 后端已就绪（8088 监听中）"
    exit 0
  fi
done
echo "[restart] WARN: 40s 内 8088 未监听，请检查 backend/stderr.log"
exit 1
