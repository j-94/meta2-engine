---
lane: D
title: UCB β/γ Selector (20 lines)
source: scripts/meta_loop.py
tags: [code, meta, bandit, routing]
bits: {A: 1, U: 0, P: 1, E: 0, Δ: 0, I: 0, R: 0, T: 1}
---

1-liner
- Pick the best plan/config pair with Upper Confidence Bound on a rubric R(s).

snippet (python)
```python
class UCB:
  def __init__(self,n): self.n=n; self.c=[0]*n; self.v=[0.0]*n; self.t=0
  def pick(self):
    self.t+=1
    for i in range(self.n):
      if self.c[i]==0: return i
    import math
    return max(range(self.n), key=lambda i: self.v[i]+(2*math.log(self.t)/self.c[i])**0.5)
  def update(self,i,r):
    self.c[i]+=1; self.v[i]+= (r-self.v[i])/self.c[i]
```

example
- β=[direct,single_tool,two_step], γ=[small350,base600]; R=2*pass-0.1*time-0.1*cost-0.05*mdl.

CTA
- Persist counts/values per arm under `trace/` to learn across sessions.

