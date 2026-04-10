function diagnose --description "Collect system state and explain what looks wrong"
  set -l model (set -q SX_MODEL && echo $SX_MODEL || echo "qwen3-coder")
  echo "Collecting system state..." >&2

  set -l info ""
  set -a info "=== LOAD ==="
  set -a info (uptime 2>/dev/null)
  set -a info ""
  set -a info "=== MEMORY ==="
  set -a info (free -h 2>/dev/null)
  set -a info ""
  set -a info "=== DISK ==="
  set -a info (df -h / /home 2>/dev/null)
  set -a info ""
  set -a info "=== TOP CPU ==="
  set -a info (ps aux --sort=-%cpu 2>/dev/null | head -6)
  set -a info ""
  set -a info "=== TOP MEM ==="
  set -a info (ps aux --sort=-%mem 2>/dev/null | head -6)
  set -a info ""
  set -a info "=== DMESG (last 20) ==="
  set -a info (dmesg --time-format reltime 2>/dev/null | tail -20)

  set -l snapshot (string join \n $info)

  _ollama $model \
    "You are a Linux sysadmin. Analyze this system snapshot. Report ONLY things that look wrong or concerning — high load, low memory, disk nearly full, suspicious processes, kernel errors. If everything looks fine, say so briefly. No markdown fences. Use • for bullets." \
    "$snapshot"
end
