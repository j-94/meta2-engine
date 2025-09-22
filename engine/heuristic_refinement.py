#!/usr/bin/env python3
"""Synthetic crowd-heuristic refinement benchmark for Meta².

This CLI builds a small testbed where a crowd-sourced heuristic labels
examples with systematic bias. It provides utilities to generate the
dataset, assess the baseline heuristic, and score human/agent-provided
refinements against a held-out split.

Usage examples:

  # Create a dataset under ./benchmarks/
  meta2-engine heuristics generate --out benchmarks/refinement.jsonl

  # Inspect baseline metrics
  meta2-engine heuristics score --dataset benchmarks/refinement.jsonl

  # Score a custom refinement defined in my_rule.py:refined
  meta2-engine heuristics score \
      --dataset benchmarks/refinement.jsonl \
      --refiner-script my_rule.py \
      --function refined

  # Quickly play with a boolean expression over features
  meta2-engine heuristics score --dataset benchmarks/refinement.jsonl \
      --expression '(color == "red" and weight > 5) or (size == "L" and weight > 7)'

Outputs are printed as JSON so results can be piped into dashboards/tests.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import math
import random
import statistics
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Dict, Iterable, List, Optional


@dataclass
class Example:
    idx: int
    split: str
    features: Dict[str, Any]
    crowd_label: bool
    oracle_label: bool

    @property
    def crowd_error(self) -> bool:
        return self.crowd_label != self.oracle_label


Split = Dict[str, List[Example]]


def _oracle(example: Dict[str, Any]) -> bool:
    """Ground-truth decision surface used for synthetic generation."""

    color = example["color"]
    size = example["size"]
    weight = example["weight"]

    return (color == "red" and weight > 5.0) or (size == "L" and weight > 7.0)


def _crowd(example: Dict[str, Any], *, rng: random.Random, noise: float) -> bool:
    """Crowd heuristic with a simple bias (only keying on color)."""

    raw = example["color"] == "red"
    if rng.random() < noise:
        return not raw
    return raw


def generate_dataset(count: int, seed: int, noise: float) -> List[Example]:
    rng = random.Random(seed)
    rows: List[Example] = []

    palette = ["red", "blue", "green"]
    sizes = ["S", "M", "L"]

    for idx in range(count):
        color = rng.choice(palette)
        size = rng.choice(sizes)
        weight = round(rng.uniform(1.0, 10.0), 2)
        features = {"color": color, "size": size, "weight": weight}
        oracle_label = _oracle(features)
        crowd_label = _crowd(features, rng=rng, noise=noise)

        split = "train"
        if idx >= math.floor(count * 0.6):
            split = "val"
        if idx >= math.floor(count * 0.8):
            split = "test"

        rows.append(
            Example(
                idx=idx,
                split=split,
                features=features,
                crowd_label=crowd_label,
                oracle_label=oracle_label,
            )
        )
    return rows


def write_jsonl(path: Path, examples: Iterable[Example]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        for ex in examples:
            fh.write(
                json.dumps(
                    {
                        "id": f"ex-{ex.idx:04d}",
                        "split": ex.split,
                        "features": ex.features,
                        "crowd_label": ex.crowd_label,
                        "oracle_label": ex.oracle_label,
                    }
                )
            )
            fh.write("\n")


def load_jsonl(path: Path) -> List[Example]:
    if not path.exists():
        raise FileNotFoundError(path)
    rows: List[Example] = []
    with path.open("r", encoding="utf-8") as fh:
        for idx, line in enumerate(fh):
            record = json.loads(line)
            rows.append(
                Example(
                    idx=idx,
                    split=record["split"],
                    features=record["features"],
                    crowd_label=bool(record["crowd_label"]),
                    oracle_label=bool(record["oracle_label"]),
                )
            )
    return rows


def accuracy(truth: Iterable[bool], preds: Iterable[bool]) -> float:
    truth_list = list(truth)
    pred_list = list(preds)
    if not truth_list:
        return 0.0
    correct = sum(t == p for t, p in zip(truth_list, pred_list))
    return correct / len(truth_list)


def confusion(truth: Iterable[bool], preds: Iterable[bool]) -> Dict[str, int]:
    truth_list = list(truth)
    pred_list = list(preds)
    tp = sum(t and p for t, p in zip(truth_list, pred_list))
    tn = sum((not t) and (not p) for t, p in zip(truth_list, pred_list))
    fp = sum((not t) and p for t, p in zip(truth_list, pred_list))
    fn = sum(t and (not p) for t, p in zip(truth_list, pred_list))
    return {"tp": tp, "tn": tn, "fp": fp, "fn": fn}


def f1(conf: Dict[str, int]) -> float:
    tp = conf["tp"]
    fp = conf["fp"]
    fn = conf["fn"]
    denom = 2 * tp + fp + fn
    if denom == 0:
        return 0.0
    return (2 * tp) / denom


def improvement_report(examples: List[Example], refined: Iterable[bool]) -> Dict[str, Any]:
    test = [ex for ex in examples if ex.split == "test"]
    truth = [ex.oracle_label for ex in test]
    crowd_preds = [ex.crowd_label for ex in test]
    refined_preds = list(refined)

    crowd_acc = accuracy(truth, crowd_preds)
    refined_acc = accuracy(truth, refined_preds)
    delta = refined_acc - crowd_acc

    crowd_conf = confusion(truth, crowd_preds)
    refined_conf = confusion(truth, refined_preds)

    crowd_f1 = f1(crowd_conf)
    refined_f1 = f1(refined_conf)

    subpop = [ex for ex in test if ex.crowd_error]
    sub_truth = [ex.oracle_label for ex in subpop]
    refined_sub = [pred for ex, pred in zip(test, refined_preds) if ex.crowd_error]
    sub_acc = accuracy(sub_truth, refined_sub) if subpop else None

    return {
        "count_test": len(test),
        "crowd_accuracy": round(crowd_acc, 4),
        "refined_accuracy": round(refined_acc, 4),
        "accuracy_delta": round(delta, 4),
        "crowd_f1": round(crowd_f1, 4),
        "refined_f1": round(refined_f1, 4),
        "f1_delta": round(refined_f1 - crowd_f1, 4),
        "crowd_confusion": crowd_conf,
        "refined_confusion": refined_conf,
        "subpopulation_error_count": len(subpop),
        "refined_accuracy_on_crowd_mistakes": None if sub_acc is None else round(sub_acc, 4),
    }


def load_callable(script_path: Path, func_name: str) -> Callable[[Dict[str, Any], bool], bool]:
    spec = importlib.util.spec_from_file_location("refiner_module", script_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"Cannot import {script_path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)  # type: ignore[arg-type]
    func = getattr(module, func_name, None)
    if func is None:
        raise AttributeError(f"{script_path} has no callable '{func_name}'")
    return func


def build_expression_callable(expr: str) -> Callable[[Dict[str, Any], bool], bool]:
    compiled = compile(expr, "<expression>", "eval")

    allowed_builtins = {"abs": abs, "min": min, "max": max, "round": round}

    def _call(features: Dict[str, Any], crowd_label: bool) -> bool:
        local_scope = {**features, "features": features, "crowd": crowd_label}
        return bool(eval(compiled, {"__builtins__": allowed_builtins}, local_scope))

    return _call


def baseline_refiner(features: Dict[str, Any], crowd_label: bool) -> bool:  # noqa: ARG001
    """Heuristic that approximates the oracle using interpretable rules."""

    return _oracle(features)


def evaluate_refiner(
    examples: List[Example],
    func: Callable[[Dict[str, Any], bool], bool],
) -> Dict[str, Any]:
    test = [ex for ex in examples if ex.split == "test"]
    refined_preds = [bool(func(ex.features, ex.crowd_label)) for ex in test]
    report = improvement_report(examples, refined_preds)

    train = [ex for ex in examples if ex.split == "train"]
    train_crowd_errors = sum(ex.crowd_error for ex in train)
    report["train_size"] = len(train)
    report["train_crowd_errors"] = train_crowd_errors
    if train:
        report["train_error_rate"] = round(train_crowd_errors / len(train), 4)

    return report


def cmd_generate(args: argparse.Namespace) -> None:
    dataset = generate_dataset(count=args.count, seed=args.seed, noise=args.noise)
    path = Path(args.out)
    write_jsonl(path, dataset)

    summary = {
        "path": str(path),
        "count": len(dataset),
        "splits": {
            split: sum(ex.split == split for ex in dataset) for split in {"train", "val", "test"}
        },
        "crowd_error_rate_train": round(
            statistics.mean(ex.crowd_error for ex in dataset if ex.split == "train"), 4
        ),
        "crowd_error_rate_test": round(
            statistics.mean(ex.crowd_error for ex in dataset if ex.split == "test"), 4
        ),
    }
    print(json.dumps(summary, indent=2))


def cmd_score(args: argparse.Namespace) -> None:
    dataset = load_jsonl(Path(args.dataset))

    if args.refiner_script:
        func = load_callable(Path(args.refiner_script), args.function)
    elif args.expression:
        func = build_expression_callable(args.expression)
    elif args.refiner == "baseline":
        func = baseline_refiner
    else:
        raise ValueError("Provide --refiner-script, --expression, or use --refiner baseline")

    report = evaluate_refiner(dataset, func)
    print(json.dumps(report, indent=2))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    gen = sub.add_parser("generate", help="Create a synthetic crowd-heuristic dataset")
    gen.add_argument("--out", type=str, required=True, help="Destination JSONL path")
    gen.add_argument("--count", type=int, default=200, help="Number of examples (default: 200)")
    gen.add_argument("--seed", type=int, default=13, help="Random seed (default: 13)")
    gen.add_argument(
        "--noise",
        type=float,
        default=0.1,
        help="Flip probability applied to the crowd heuristic (default: 0.1)",
    )
    gen.set_defaults(func=cmd_generate)

    score = sub.add_parser("score", help="Evaluate a refinement against the held-out test set")
    score.add_argument("--dataset", required=True, help="Path to generated dataset JSONL")
    score.add_argument(
        "--refiner",
        choices=["baseline"],
        default="baseline",
        help="Use built-in refiners (default: baseline oracle)"
    )
    score.add_argument(
        "--refiner-script",
        help="Path to a Python file exposing a callable that implements the refined heuristic",
    )
    score.add_argument(
        "--function",
        default="refined",
        help="Function name inside --refiner-script (default: refined)",
    )
    score.add_argument(
        "--expression",
        help="Boolean expression over features (e.g., 'color==\"red\" and weight>5')",
    )
    score.set_defaults(func=cmd_score)

    return parser


def main(argv: Optional[List[str]] = None) -> None:
    parser = build_parser()
    args = parser.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
