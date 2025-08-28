#!/usr/bin/env python3
import json
import os
from pathlib import Path
from datetime import datetime

def rollup_traces():
    """Convert trace ledger to episodes"""
    os.makedirs("_episodes", exist_ok=True)
    os.makedirs("_notebook", exist_ok=True)
    
    episodes = []
    
    # Read trace ledger
    ledger_path = Path("trace/ledger.jsonl")
    if ledger_path.exists():
        with open(ledger_path) as f:
            for line in f:
                if line.strip():
                    try:
                        trace = json.loads(line)
                        episode = {
                            "episode_id": trace.get("run_id", "unknown"),
                            "goal": trace.get("goal", "unknown"),
                            "started_at": trace.get("ts", ""),
                            "bits": trace.get("bits", {}),
                            "steps": trace.get("steps", []),
                            "deliverables": trace.get("deliverables", [])
                        }
                        episodes.append(episode)
                        
                        # Save individual episode
                        ep_file = f"_episodes/{episode['episode_id']}.json"
                        with open(ep_file, 'w') as ef:
                            json.dump(episode, ef, indent=2)
                            
                    except json.JSONDecodeError:
                        continue
    
    # Create summary notebook
    summary = {
        "generated_at": datetime.utcnow().isoformat(),
        "total_episodes": len(episodes),
        "avg_trust": sum(ep["bits"].get("t", 0) for ep in episodes) / max(len(episodes), 1),
        "error_rate": sum(1 for ep in episodes if ep["bits"].get("e", 0) > 0) / max(len(episodes), 1),
        "recent_episodes": episodes[-10:] if episodes else []
    }
    
    with open("_notebook/summary.json", 'w') as f:
        json.dump(summary, f, indent=2)
    
    print(f"Processed {len(episodes)} episodes")

if __name__ == "__main__":
    rollup_traces()
