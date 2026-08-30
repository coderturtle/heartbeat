# Module 09: Synthesis Capstone

## The question this module answers

Given a real bug in the system you built, which concept is actually the root cause?

## Where it sits in the arc

Ninth and final module. Prerequisite: all of Modules 01-08 - this module doesn't introduce new distributed-systems content, it tests whether the whole arc actually composed into working judgment. See [`modules/README.md`](../README.md) for the full arc.

## Learning objectives (placeholder - finalized when content is authored)

- Diagnose a real, seeded bug spanning 3+ concepts from the arc (e.g. a persistence bug that only manifests as an apparent log-replication failure) without being told which module it belongs to.
- Distinguish a symptom from a root cause under time pressure, the same skill Module 07 first required at a smaller scale.
- Defend a diagnosis in writing before fixing it - not fix first and rationalize after.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

A real, seeded bug or non-idiomatic pattern in the accumulated `Checkout` service built across Modules 01-08 (mirrors `borrow-native`'s own capstone shape: diagnose the bottleneck, fix it, defend the diagnosis in writing - against a real product the learner helped build, not a fixture manufactured just to be broken). See [`docs/workshop-design.md`](../../docs/workshop-design.md).

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** every module's `turmoil` test suite is green again on the fixed program, across each module's own practice set and grading draw.
- **Conceptual tier (Coachgremlin):** a written diagnosis, produced *before* the fix, that correctly names the root-cause concept (not just a symptom) - Coachgremlin confirms the diagnosis is correct and was actually written first, not reconstructed after the fact.

**Grading must be designed against known bypass patterns, not just trusted, added 2026-08-29 across two doubt-driven-development cycles.** This factory has direct, evidenced history with checker-execution-bypass patterns - a shadow module planted in the working directory, or an exit-code short-circuit, defeating a checker that only inspects a process's return code. Coachgremlin's content-building pass must design this module's grading (how the seeded bug is verified fixed, how the written diagnosis is checked against the actual root cause) with that specific failure mode in mind from the start: a neutral, non-learner-writable execution directory and a real pass-*count* verification (not a bare exit code, which a zero-tests-collected run could satisfy vacuously), run against both the practice set and the grading draw specifically, not "the full seed sets" left ambiguous about which.

**Grading-draw path scoping, left open on purpose (`docs/workshop-design.md`, Phase A.8 finding):** the capstone's solution tree is the largest and most accumulated of any module, so hashing the whole thing for the grading-draw nonce would give a learner the largest, easiest-to-grind file surface in the workshop - backwards from what this module's stakes call for. Coachgremlin's content-authoring pass must deliberately scope which files feed that hash (e.g., restricted to what's actually newly graded here) rather than defaulting to the entire repository tree.

## Takeaway

A personal distributed-systems diagnostic playbook, compressing the whole arc's diagnostic habits (RPC-layer doubt, election reasoning, log-matching checks, persistence minimality, snapshot boundaries, layering discipline, ownership invariants) into one checklist, built *from* the defended diagnosis above, not a substitute for it. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's fix passes the deterministic tier across every module's seed set, and Coachgremlin confirms the written diagnosis correctly named the root-cause concept, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
