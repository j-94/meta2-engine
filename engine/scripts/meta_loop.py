import ast
import os
import sys
import time
import uuid
import json
import math
import random
import datetime
from typing import Any, Dict

RECEIPTS = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "trace", "meta_receipts.jsonl")
)
os.makedirs(os.path.dirname(RECEIPTS), exist_ok=True)
CONFIG = os.environ.get("META_CFG", os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "policies", "meta_loop.json")))
STATE_PATH = os.environ.get(
    "META_STATE",
    os.path.abspath(
        os.path.join(
            os.path.dirname(__file__), "..", "trace", "meta_ucb_state.json"
        )
    ),
)

def now():
  return datetime.datetime.utcnow().isoformat()+"Z"

def load_cfg(path):
  try:
    with open(path, "r", encoding="utf-8") as f:
      return json.load(f)
  except Exception:
    # default
    return {
      "telemetry": ["pass", "cost", "time", "mdl"],
      "rubric": "2*pass - 0.1*time - 0.1*cost - 0.05*mdl",
      "beta_plans": [{"id":"direct"},{"id":"single_tool"},{"id":"two_step"}],
      "gamma_configs": [{"id":"small350","model":"small","chunk":350},{"id":"base600","model":"base","chunk":600}],
      "policy": {"selector":"UCB","hysteresis_eps":0.2,"min_trials_per_arm":3}
    }

class UCB:
  def __init__(self, n, state=None, key_prefix="arm"):
    self.n = n
    self.count = [0] * n
    self.value = [0.0] * n
    self.t = 0
    self.key_prefix = key_prefix
    if state:
      self.t = state.get("t", 0)
      for i in range(n):
        k = f"{key_prefix}_{i}"
        arm = state.get(k)
        if arm:
          self.count[i] = int(arm.get("c", 0))
          self.value[i] = float(arm.get("v", 0.0))

  def pick(self):
    self.t += 1
    # ensure each arm tried at least once
    for i in range(self.n):
      if self.count[i] == 0:
        return i
    ucb_vals = [
      self.value[i] + math.sqrt(2 * math.log(self.t) / self.count[i])
      for i in range(self.n)
    ]
    return max(range(self.n), key=lambda i: ucb_vals[i])

  def update(self, i, reward):
    self.count[i] += 1
    # incremental mean
    self.value[i] += (reward - self.value[i]) / self.count[i]

  def dump(self):
    d = {"t": self.t}
    for i in range(self.n):
      d[f"{self.key_prefix}_{i}"] = {
        "c": self.count[i],
        "v": round(self.value[i], 6),
      }
    return d


class SafeRubric:
  ALLOWED_BINOPS = (
    ast.Add,
    ast.Sub,
    ast.Mult,
    ast.Div,
    ast.Pow,
  )
  ALLOWED_UNARY = (ast.UAdd, ast.USub)

  def __init__(self, expr: str, variables: Dict[str, float]):
    self.expr = expr or "0"
    self.variables = set(variables)
    self._tree = ast.parse(self.expr, mode="eval")
    self._validate(self._tree.body)

  def _validate(self, node: ast.AST):
    if isinstance(node, ast.BinOp):
      if not isinstance(node.op, self.ALLOWED_BINOPS):
        raise ValueError(f"Unsupported operator in rubric: {type(node.op).__name__}")
      self._validate(node.left)
      self._validate(node.right)
    elif isinstance(node, ast.UnaryOp):
      if not isinstance(node.op, self.ALLOWED_UNARY):
        raise ValueError("Unsupported unary operator in rubric")
      self._validate(node.operand)
    elif isinstance(node, ast.Num):
      return
    elif isinstance(node, ast.Constant):
      if not isinstance(node.value, (int, float)):
        raise ValueError("Only numeric constants are allowed in rubric")
    elif isinstance(node, ast.Name):
      if node.id not in self.variables:
        raise ValueError(f"Unknown variable '{node.id}' in rubric")
    else:
      raise ValueError(f"Unsupported expression '{ast.dump(node)}' in rubric")

  def _eval(self, node: ast.AST, metrics: Dict[str, float]) -> float:
    if isinstance(node, ast.BinOp):
      left = self._eval(node.left, metrics)
      right = self._eval(node.right, metrics)
      if isinstance(node.op, ast.Add):
        return left + right
      if isinstance(node.op, ast.Sub):
        return left - right
      if isinstance(node.op, ast.Mult):
        return left * right
      if isinstance(node.op, ast.Div):
        return left / right
      if isinstance(node.op, ast.Pow):
        return left ** right
      raise ValueError("Unsupported operator")
    if isinstance(node, ast.UnaryOp):
      operand = self._eval(node.operand, metrics)
      if isinstance(node.op, ast.UAdd):
        return +operand
      if isinstance(node.op, ast.USub):
        return -operand
      raise ValueError("Unsupported unary operator")
    if isinstance(node, ast.Num):
      return float(node.n)
    if isinstance(node, ast.Constant):
      return float(node.value)
    if isinstance(node, ast.Name):
      return float(metrics.get(node.id, 0.0))
    raise ValueError("Unsupported expression node")

  def eval(self, metrics: Dict[str, float]) -> float:
    return float(self._eval(self._tree.body, metrics))

def jl(path, rec):
  with open(path, "a", encoding="utf-8") as f:
    f.write(json.dumps(rec, ensure_ascii=False)+"\n")

def execute(plan_id, cfg, task):
  t0=time.time()
  # Placeholder: synthesize artifact text depending on plan/cfg
  artifact = f"PLAN[{plan_id}] CFG[{cfg['id']}] TASK[{task}]"
  # Simulate pass probability increases with bigger chunk/topk and two_step plan
  base_p = 0.55 + (0.05 if plan_id == "single_tool" else 0.0) + (0.1 if plan_id == "two_step" else 0.0)
  base_p += 0.05 if cfg.get("chunk", 0) >= 600 else 0.0
  ok = random.random() < min(0.95, base_p)
  # Telemetry
  latency = 0.05 + (cfg.get("chunk", 300) / 3000.0)
  cost = 0.1 + 0.05 * (1 if cfg.get("model") == "base" else 0)
  mdl = len(artifact) / 1000.0
  s = {
    "pass": 1 if ok else 0,
    "time": round(latency, 3),
    "cost": round(cost, 3),
    "mdl": round(mdl, 3),
  }
  dt = time.time() - t0
  return artifact, s, dt


def eval_rubric(rubric, metrics):
  if isinstance(rubric, SafeRubric):
    try:
      return rubric.eval(metrics)
    except Exception:
      pass
  # default fallback
  return (
    2.0 * metrics.get("pass", 0)
    - 0.1 * metrics.get("time", 0)
    - 0.1 * metrics.get("cost", 0)
    - 0.05 * metrics.get("mdl", 0)
  )

def load_state():
  try:
    with open(STATE_PATH, 'r', encoding='utf-8') as f:
      return json.load(f)
  except Exception:
    return {}

def save_state(state):
  try:
    os.makedirs(os.path.dirname(STATE_PATH), exist_ok=True)
    with open(STATE_PATH, 'w', encoding='utf-8') as f:
      json.dump(state, f, ensure_ascii=False, indent=2)
  except Exception:
    pass


def _update_stat_entry(entry: Dict[str, Any], value: float, key: str):
  prev = entry.get(key, 0.0)
  entry[key] = prev + (value - prev) / entry["count"]


def update_stats(stats: Dict[str, Any], plan_id: str, cfg_id: str, telemetry: Dict[str, float], score: float, tele_keys):
  beta_stats = stats.setdefault("beta", {})
  gamma_stats = stats.setdefault("gamma", {})
  pair_stats = stats.setdefault("pairs", {})

  def ensure(entry_dict, key):
    entry = entry_dict.get(key)
    if not entry:
      entry = {"count": 0, "avg_score": 0.0, "pass_rate": 0.0}
      entry_dict[key] = entry
    return entry

  def bump(entry):
    entry["count"] += 1
    entry["avg_score"] += (score - entry["avg_score"]) / entry["count"]
    if "pass" in tele_keys:
      _update_stat_entry(entry, telemetry.get("pass", 0.0), "pass_rate")
    for key in tele_keys:
      if key == "pass":
        continue
      if key in telemetry:
        _update_stat_entry(entry, telemetry[key], f"avg_{key}")
    entry["last_score"] = score
    entry["last"] = {
      "telemetry": telemetry,
      "ts": now(),
    }

  bump(ensure(beta_stats, plan_id))
  bump(ensure(gamma_stats, cfg_id))
  pair_key = f"{plan_id}::{cfg_id}"
  bump(ensure(pair_stats, pair_key))


def summarize_feedback(stats: Dict[str, Any], plan_id: str, cfg_id: str):
  def top_n(section):
    items = []
    for key, info in section.items():
      items.append(
        {
          "id": key,
          "count": info.get("count", 0),
          "avg_score": round(info.get("avg_score", 0.0), 3),
          "pass_rate": round(info.get("pass_rate", 0.0), 3),
        }
      )
    return sorted(items, key=lambda x: x["avg_score"], reverse=True)

  beta_fb = top_n(stats.get("beta", {}))
  gamma_fb = top_n(stats.get("gamma", {}))
  pair_fb = top_n(stats.get("pairs", {}))
  selected_pair = f"{plan_id}::{cfg_id}"
  selected_stats = stats.get("pairs", {}).get(selected_pair, {})
  return {
    "beta": beta_fb,
    "gamma": gamma_fb,
    "pairs": pair_fb,
    "selected": {
      "pair": selected_pair,
      "snapshot": selected_stats,
    },
  }

def run_once(task, cfg):
  beta = [b["id"] for b in cfg["beta_plans"]]
  gamma = [g for g in cfg["gamma_configs"]]
  telemetry_keys = cfg.get("telemetry", []) or ["pass", "cost", "time", "mdl"]
  try:
    rubric = SafeRubric(cfg.get("rubric", ""), {k: 0.0 for k in telemetry_keys})
  except ValueError:
    rubric = None

  st = load_state()
  beta_ucb = UCB(len(beta), st.get("beta"), key_prefix="b")
  gamma_ucb = UCB(len(gamma), st.get("gamma"), key_prefix="g")

  # Try one combined pick
  j = beta_ucb.pick()
  k = gamma_ucb.pick()
  plan = beta[j]
  gcfg = gamma[k]

  artifact, s, dt = execute(plan, gcfg, task)
  score = eval_rubric(rubric, s)
  beta_ucb.update(j, score)
  gamma_ucb.update(k, score)

  stats = st.get("stats", {})
  update_stats(stats, plan, gcfg["id"], s, score, telemetry_keys)
  feedback = summarize_feedback(stats, plan, gcfg["id"])

  st = {
    "beta": beta_ucb.dump(),
    "gamma": gamma_ucb.dump(),
    "beta_ids": beta,
    "gamma_ids": [g["id"] for g in gamma],
    "rubric": cfg.get("rubric"),
    "ts": now(),
    "stats": stats,
  }
  save_state(st)

  return {
    "plan": plan,
    "config": gcfg,
    "artifact": artifact,
    "telemetry": s,
    "score": round(score, 3),
    "latency_s": round(dt, 3),
    "beta_state": st["beta"],
    "gamma_state": st["gamma"],
    "feedback": feedback,
  }

def main():
  if len(sys.argv)<2:
    print("usage: python scripts/meta_loop.py '<task>'")
    sys.exit(1)
  task = sys.argv[1]
  cfg = load_cfg(CONFIG)
  out = run_once(task, cfg)
  rec = {"run_id": str(uuid.uuid4())[:8], "ts": now(), "task": task, **out}
  jl(RECEIPTS, rec)
  print(json.dumps(rec, ensure_ascii=False))

if __name__ == "__main__":
  main()
