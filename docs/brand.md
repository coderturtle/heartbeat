# Brand / Style Layer: Heartbeat

> The only place this workshop's personality lives. `README.md` and, once built, `site/`'s layout and `astro.config.mjs` all read from this file — they don't redefine voice, banned language, or visual identity independently. Adapted from `borrow-native/docs/brand.md`, itself adapted from `terminal-velocity/docs/brand.md` and blog-factory-lab's `templates/brand-style-layer-template.md`.

## Site identity

**Name:** Heartbeat
**Tagline:** Let a simulated, adversarial network — not an opinion — be the first gate.
**Parent brand:** Hekton
**Slug:** `heartbeat`

The name is Raft's own term for the RPC a leader sends to prove it's still alive — the workshop's whole premise is that "still alive" is a much harder claim to verify in a distributed system than it sounds, and this workshop's deterministic gate exists to actually check it rather than assume it.

## Tone and voice

**Core voice:** A competent peer, not an instructor. Specific, dryly funny, anti-hype by default. Treats the reader as someone who already ships Rust daily (via `borrow-native` or equivalent) — the only thing assumed unfamiliar is distributed systems itself.

**Tone rules:**
- Prefer plain verbs and concrete nouns; show the mechanism (a specific `turmoil` seed, a specific failed invariant), don't just assert the result.
- Any claim about what the teaching method achieves ("the deterministic gate makes grading more trustworthy," "Coachgremlin catches what a passing test can't") must be marked as a hypothesis or stated intent unless there's actual evidence behind it — this workshop's own design docs got caught stating one such claim as settled fact by its own Review Panel (Skeptical Critic persona); the fix is a permanent voice rule, not a one-time edit.
- First person for build-log entries. System/instructional language for module content and workshop structure.
- Admit uncertainty directly rather than smoothing over it — this subject has more genuine open risk (a test can pass by luck) than `borrow-native`'s did, and the voice should reflect that honestly rather than oversell the gate's authority.
- Never imply a passing deterministic gate means "safe to run in production" or "definitely correct" — name what it actually checked.

## Hard rules

- **No em dash characters.** Use period, colon, semicolon, comma, parenthesis, or a plain hyphen instead. (Applies to all published workshop content — README, module READMEs, build-log entries, the site. Design/planning docs under `docs/` are working documents and are exempt.)
- No AI-slop openers ("In today's fast-paced world...", "It's important to note...").
- No unqualified efficacy superlatives ("game-changing," "revolutionary," "10x," "unlock your potential," "bulletproof," "split-brain-proof").
- No engagement bait, fake scarcity, or "one weird trick" framing.
- **Distributed-systems-specific:** never claim a passing `turmoil` test set proves an implementation is correct, free of races, or production-ready. Name what was actually checked (which fault, how many seeds) instead of a blanket confidence claim. This workshop's whole bet is that most distributed-systems bugs hide exactly where an overconfident "tests pass" claim would paper over them.
- **Rust-specific (inherited from `borrow-native`):** never imply `unsafe`, `.clone()`-to-silence-the-compiler, or `unsafe impl Send`/`Sync` is a reasonable default fix in any published example.

## Banned phrases

Reused from the wider Hekton house style, plus workshop-specific additions:

- delve, tapestry, unlock, seamless, game-changing, revolutionize, transform your workflow, supercharge, effortlessly, cutting-edge, thought leader
- "in today's fast-paced world," "it's important to note," "at scale" (unless the content proves the scale)
- Workshop-specific: "master the art of," "in this comprehensive guide," "unlock your potential," "10x your skills," "split-brain-proof," "bulletproof consensus" (both imply a certainty this workshop's own honesty rules forbid), "fight the network" (frames the simulated adversary as something to defeat rather than understand, the same anti-pattern `borrow-native` named for "fight the borrow checker")

## Visual identity

Inherit `terminal-velocity`/`borrow-native`'s Astro starter tokens rather than invent a new palette, once the site is built: `--accent`, `ink`/`paper` Tailwind tokens, the `.post-body` typography rhythm, "no section dividers, whitespace only."

| Element | Direction |
|---|---|
| Overall mood | Clean technical workshop notebook. Not a marketing landing page. |
| Colour approach | Dark-on-light default; restrained palette; dark mode optional later |
| Typography | Crisp, generous whitespace, readable code blocks (Rust syntax highlighting matters here, same as `borrow-native`) |
| Imagery | Artifact-led: `turmoil` seed output, a partition diagram, a terminal session — not stock photos or decorative AI art |
| Decoration | No neon AI aesthetic, no hero banners, no gradient-mesh backgrounds |

## Gremlin and factory language rules

- Coachgremlin and the Workshop Gremlin are real, documented agents with concrete responsibilities (`<hekton-machinery>/gremlins/`) — reference them plainly when explaining how the workshop works, don't decorate every heading with gremlin language, and don't assume a learner already knows what a "Gremlin" is without a one-line explanation the first time the term appears in learner-facing copy.
- A module README is a production artifact: plain. A build-log entry can be funny where the actual events were funny.

## Anti-goals

- Not an AI-hype funnel or a marketing page for Hekton.
- Not a certification mill — no claim that completing this workshop credentials anything.
- Not a place to publish unverified efficacy claims — every claim about what the teaching method or the deterministic gate achieves gets the hypothesis/stated-intent treatment above until there's real evidence.
- Not overrun with gremlin language to the point of reading childish.
- Not a replacement for MIT 6.5840 — it exists alongside that course's own labs and lecture notes, not instead of them; never imply 6.5840 is inadequate, only that this workshop adds something it doesn't (Rust, agent-native delivery, a conceptual grading tier, keepable takeaways).

## Application map

| Artifact | Reads |
|---|---|
| `README.md` | Title + tagline |
| `site/` (once built) | Tone, hard rules, banned phrases, visual identity |
| Module READMEs | Tone, hard rules, banned phrases, distributed-systems-specific overclaiming rule |
| Build-log entries | Tone and voice rules (first person, tension, no hype) |

## [TBD]: items for later

- [ ] Exact accent colour token (once site is built)
- [ ] Favicon / wordmark treatment
- [ ] Dark mode colour tokens
- [ ] Rust-syntax-highlighting theme choice for code blocks
