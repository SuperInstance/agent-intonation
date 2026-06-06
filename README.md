# agent-intonation

*A violinist who plays 5 cents sharp is fine alone. In a quartet, they're a problem.*

---

Measuring how accurately an agent's output matches its intent — in musical terms, whether the agent plays in tune.

In music, intonation is the accuracy of pitch relative to the intended note. A few cents off is acceptable. Fifty cents off and you're playing a different note. And when multiple musicians are all slightly out of tune, the beating frequencies between them create audible interference — the compounded error is worse than any individual error.

Agents have the same property. An agent that's 95% accurate works fine in isolation. But when 5 agents each at 95% accuracy compose their outputs, the cascade deviation compounds. This crate measures and tracks that.

Provides: Intonation readings with cent deviations, IntonationQuality (Perfect/Good/Acceptable/Poor/Unusable), IntonationTracker with per-agent analysis, beating frequency between agent pairs (interference), cascade deviation (RMS of compounded errors), and a comparative experiment.

The insight: intonation quality matters more as the fleet grows. Two agents at ±10 cents are fine. Ten agents at ±10 cents create cascading inaccuracy. Fleet design must account for this — either improve individual accuracy or reduce the depth of the composition chain.

10 tests: in/out of tune, quality levels, average deviation, beating frequency, cascade compounding, worst quality, experiments with low vs high deviation.

Part of [SuperInstance](https://github.com/SuperInstance/SuperInstance).

License: MIT
