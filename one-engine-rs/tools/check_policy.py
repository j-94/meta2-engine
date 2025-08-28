#!/usr/bin/env python3
import json
import sys
from pathlib import Path

def check_policy():
    """Enforce policy guardrails"""
    
    # Load summary
    summary_path = Path("_notebook/summary.json")
    if not summary_path.exists():
        print("No summary found, skipping policy check")
        return True
    
    with open(summary_path) as f:
        summary = json.load(f)
    
    violations = []
    
    # Policy: Trust should be > 0.7
    avg_trust = summary.get("avg_trust", 0)
    if avg_trust < 0.7:
        violations.append(f"Low trust: {avg_trust:.2f} < 0.7")
    
    # Policy: Error rate should be < 0.2
    error_rate = summary.get("error_rate", 0)
    if error_rate > 0.2:
        violations.append(f"High error rate: {error_rate:.2f} > 0.2")
    
    # Policy: Must have recent activity
    recent = summary.get("recent_episodes", [])
    if len(recent) == 0:
        violations.append("No recent episodes")
    
    if violations:
        print("POLICY VIOLATIONS:")
        for v in violations:
            print(f"  - {v}")
        return False
    else:
        print("Policy check PASSED")
        return True

if __name__ == "__main__":
    success = check_policy()
    sys.exit(0 if success else 1)
