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

A real, seeded bug or non-idiomatic pattern in the accumulated project built across Modules 01-08 (mirrors `borrow-native`'s own capstone shape: diagnose the bottleneck, fix it, defend the diagnosis in writing - against a real project the learner helped build, not a fixture manufactured just to be broken). See [`docs/workshop-design.md`](../../docs/workshop-design.md).

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** every module's `turmoil` test suite is green again on the fixed program, across each module's own published seed set.
- **Conceptual tier (Coachgremlin):** a written diagnosis, produced *before* the fix, that correctly names the root-cause concept (not just a symptom) - Coachgremlin confirms the diagnosis is correct and was actually written first, not reconstructed after the fact.

## Takeaway

A personal distributed-systems diagnostic playbook, compressing the whole arc's diagnostic habits (RPC-layer doubt, election reasoning, log-matching checks, persistence minimality, snapshot boundaries, layering discipline, ownership invariants) into one checklist, built *from* the defended diagnosis above, not a substitute for it. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's fix passes the deterministic tier across every module's seed set, and Coachgremlin confirms the written diagnosis correctly named the root-cause concept, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
