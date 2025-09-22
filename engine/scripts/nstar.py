import os,sys,uuid,time,random,json,math,statistics,datetime

# Receipt file lives under repo trace/
RECEIPTS_PATH = os.environ.get("NSTAR_RECEIPTS", os.path.join(os.path.dirname(__file__), "..", "trace", "receipts.jsonl"))
RECEIPTS_PATH = os.path.abspath(RECEIPTS_PATH)
os.makedirs(os.path.dirname(RECEIPTS_PATH), exist_ok=True)

POLICY={
  "explore_budget":0.15,
  "branches":3,
  "assessor_pool":3,
  "rotate_p":0.5,
  "plateau_window":50,
  "plateau_delta":0.01,
  "adapt_step":0.05,
  "min_branches":2,
  "max_branches":6,
  "novelty_window":100
}
STATE={"runs":0,"wins":[],"costs":[],"sources_seen":set(),"last_adapt":None}

PROMPTS={
  "cognition":"Do the task: {task}. Output a concrete diff/plan.",
  "metacog":"Rate confidence 0..1 and list 3 likely failure modes for: {output}",
  "verify":"Given output '{output}', does it satisfy '{task}'? Return PASS/FAIL and 1 sentence.",
  "meta":"Given recent receipts, propose changes to branches/explore_budget/thresholds when plateauing."
}

def now(): return datetime.datetime.utcnow().isoformat()+"Z"
def jl(path,rec):
  with open(path,"a", encoding="utf-8") as f:
    f.write(json.dumps(rec,ensure_ascii=False)+"\n")

def pseudo_llm(prompt,seed=None):
  random.seed((seed or 0)+hash(prompt)%10_000_019)
  t=prompt.lower()
  if "confidence" in t:
    return {"confidence":round(random.uniform(0.55,0.95),2),"risks":["spec gap","edge cases","tool misuse"]}
  if "pass/fail" in t or "return pass/fail" in t:
    return {"verdict":"PASS" if random.random()>0.25 else "FAIL","note":"heuristic verifier"}
  if "propose changes" in t:
    r={}
    r["branches"]=max(POLICY["min_branches"],min(POLICY["max_branches"],POLICY["branches"]+(1 if random.random()>0.5 else -1)))
    r["explore_budget"]=max(0.05,min(0.5,round(POLICY["explore_budget"]+random.choice([-1,1])*0.05,2)))
    return r
  return "PATCH: "+prompt.split(": ",1)[-1][:120]

def plan_variants(task,k):
  return [pseudo_llm(PROMPTS["cognition"].format(task=f"{task} [plan {i+1}]"),seed=i) for i in range(k)]

def metacog_eval(out):
  r=pseudo_llm(PROMPTS["metacog"].format(output=out)); return r["confidence"],r["risks"]

def cheap_probe(out): return len(out)

def verify(task,out):
  v=pseudo_llm("Return PASS/FAIL: "+PROMPTS["verify"].format(output=out,task=task)); return v["verdict"]=="PASS",v["note"]

def plateau(wins,window,delta):
  if len(wins)<window*2: return False
  a=wins[-window*2:-window]; b=wins[-window:]
  return (statistics.mean(b)-statistics.mean(a))<delta

def adapt_if_needed(receipts):
  trig = plateau(STATE["wins"],POLICY["plateau_window"],POLICY["plateau_delta"])
  if not trig: return {"changed":False}
  props=pseudo_llm(PROMPTS["meta"],seed=STATE["runs"])
  POLICY["branches"]=props.get("branches",POLICY["branches"])
  POLICY["explore_budget"]=props.get("explore_budget",POLICY["explore_budget"])
  STATE["last_adapt"]=now()
  return {"changed":True,"policy":{k:POLICY[k] for k in ("branches","explore_budget")}}

def execute(task):
  run_id=str(uuid.uuid4())[:8]; t0=time.time()
  k=POLICY["branches"]; cands=plan_variants(task,k)
  scored=[(cheap_probe(o),o) for o in cands]; scored.sort(reverse=True)
  budget=int(math.ceil(POLICY["explore_budget"]*len(scored))); try_set=scored[:max(1,budget)]
  best=None; best_score=-1; probes=[]
  for s,o in try_set:
    conf,risks=metacog_eval(o); probes.append({"out":o,"score":s,"conf":conf,"risks":risks})
    s2=s*conf
    if s2>best_score:
      best_score, best = s2, o
  ok,note=verify(task,best)
  dt=time.time()-t0; cost=round(0.001*sum(p["score"] for p in probes),4)
  STATE["runs"]+=1; STATE["wins"].append(1 if ok else 0); STATE["costs"].append(cost)
  rec={"run_id":run_id,"ts":now(),"task":task,"ok":ok,"note":note,"policy":{k:POLICY[k] for k in POLICY},"best":best,"probes":probes,"cost":cost,"latency_s":round(dt,3)}
  jl(RECEIPTS_PATH,rec)
  adapt=adapt_if_needed(rec)
  if adapt["changed"]:
    jl(RECEIPTS_PATH,{"run_id":run_id,"ts":now(),"adapt":adapt})
  return ok,best,rec,adapt

if __name__=="__main__":
  if len(sys.argv)<2:
    print("usage: python nstar.py \"<task>\"")
    sys.exit(1)
  ok,out,rec,adapt=execute(sys.argv[1])
  print(json.dumps({"ok":ok,"result":out,"policy":rec["policy"],"adapt":adapt},ensure_ascii=False))
