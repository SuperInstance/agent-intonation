# agent-intonation

**Accuracy measurement for agent fleets, modeled on musical intonation.**

A violinist playing in tune produces pitches that match the reference frequency. A violinist playing slightly out of tune produces something close — close enough that you recognize the melody, but off enough that it makes you wince. The scary part: put two slightly-out-of-tune violinists together, and the errors don't cancel. They *compound*. The beating frequencies between their misaligned pitches create audible interference that's worse than either individual error.

Agents have the same property. An agent with good intonation produces output that closely matches its intent. An agent with poor intonation produces approximately-right results — recognizable but imprecise. And when you compose two slightly-off agents into a pipeline, the cascade effect makes the combined output worse than either agent alone.

`agent-intonation` measures this. In cents (hundredths of a semitone), because that's the natural unit for "close but not quite."

## Why This Exists

Most agent evaluation is binary: right or wrong. But real agent output exists on a spectrum. An agent that's 95% accurate is different from one that's 99% accurate, and that difference matters enormously when you chain agents together.

The cascade effect is the killer. If you have 5 agents in a pipeline, each 95% accurate, the composed accuracy isn't 95%. It's closer to `√(5 × 5²) = ~11.2` cents of RMS deviation — about a quarter semitone of cumulative error. That's the difference between a clean chord and a muddy mess.

Musicians solved this centuries ago: tuning systems, reference pitches, and intonation practice. This crate applies the same thinking to agents.

## Core Idea

**Intonation = accuracy relative to a reference.** Not "is it right?" but "how close is it?" Measured in cents because small deviations matter when they compound.

Five quality levels, from perfect to unusable:

| Quality | Cents | Analogy |
|---------|-------|---------|
| Perfect | ≤ 5 | Studio musician, spot-on |
| Good | ≤ 15 | Professional, clean performance |
| Acceptable | ≤ 30 | Competent, minor imperfections |
| Poor | ≤ 50 | Amateur, noticeably off |
| Unusable | > 50 | Needs human intervention |

## Architecture

```
Intonation (agent + deviation + dimension)
  ├─ in_tune(tolerance_cents) → bool
  └─ quality() → IntonationQuality
  
IntonationTracker
  ├─ record(agent, cents, dimension)
  ├─ for_agent(agent) → readings
  ├─ average_deviation() → f64
  ├─ in_tune_fraction(tolerance) → f64
  ├─ beating_frequency(a, b, dimension) → f64
  ├─ cascade_deviation(agents, dimension) → f64 (RMS)
  └─ worst_quality() → IntonationQuality

run_intonation_experiment() → (in_tune_fraction, cascade_deviation)
```

## Usage

### Basic Intonation Check

```rust
use agent_intonation::{Intonation, IntonationQuality};

let reading = Intonation::new("agent-search", 8.0, "relevance");
assert!(reading.in_tune(10.0)); // 8 cents is within 10-cent tolerance
assert_eq!(reading.quality(), IntonationQuality::Good); // 8 ≤ 15
```

### Tracking Multiple Agents

```rust
use agent_intonation::IntonationTracker;

let mut tracker = IntonationTracker::new();

// Record deviations from expected output
tracker.record("search", 5.0, "relevance");   // nearly perfect
tracker.record("search", 12.0, "latency");    // good
tracker.record("ranker", 25.0, "relevance");  // acceptable
tracker.record("ranker", 45.0, "diversity");  // poor

// How much of the fleet is in tune?
let fraction = tracker.in_tune_fraction(15.0); // fraction within tolerance

// What's the worst dimension?
let worst = tracker.worst_quality(); // Poor (45 cents)
```

### Measuring Cascade Deviation

This is where it gets interesting.

```rust
// Three agents in a pipeline
tracker.record("extractor", 10.0, "accuracy");
tracker.record("processor", 12.0, "accuracy");
tracker.record("formatter", 8.0, "accuracy");

// Individual deviations are acceptable (8-12 cents)
// But composed:
let cascade = tracker.cascade_deviation(
    &["extractor", "processor", "formatter"], 
    "accuracy"
);
// RMS of [10, 12, 8] = √(100 + 144 + 64) = √308 ≈ 17.5 cents
// Still OK for 3 agents, but grows with √n
```

The cascade deviation uses RMS (root mean square) because that's how compound errors actually behave. It's the same math as total harmonic distortion in audio — the deviations don't cancel, they compound in quadrature.

### Detecting Beating Between Agents

```rust
// Two agents with slightly different deviations
tracker.record("agent-a", 10.0, "output_quality");
tracker.record("agent-b", 15.0, "output_quality");

let beating = tracker.beating_frequency("agent-a", "agent-b", "output_quality");
// 5.0 cents — the interference between their outputs
```

Beating frequency is the gap between two agents' deviations. In music, beating creates audible pulsing when two notes are slightly out of tune. In agent systems, it represents the incoherence between two agents working on the same problem. Zero beating = perfectly aligned. High beating = they're fighting each other.

### Running the Experiment

```rust
use agent_intonation::run_intonation_experiment;

// 5 agents, 10-cent base deviation, 20 measurement steps
let (in_tune_fraction, cascade_deviation) = run_intonation_experiment(5, 10.0, 20);

// With low deviation (5 cents), agents stay in tune
let (good_tune, _) = run_intonation_experiment(5, 5.0, 20);

// With high deviation (50 cents), they don't
let (bad_tune, _) = run_intonation_experiment(5, 50.0, 20);

assert!(good_tune >= bad_tune); // always true
```

## API Reference

| Type | Purpose |
|------|---------|
| `Intonation` | Single deviation reading (agent + cents + dimension) |
| `IntonationQuality` | 5-level accuracy classification |
| `IntonationTracker` | Multi-agent, multi-dimension tracker |
| `run_intonation_experiment` | Controlled experiment with deterministic PRNG |

### IntonationTracker Methods

| Method | Returns | What it measures |
|--------|---------|-----------------|
| `record(agent, cents, dim)` | `()` | Log a deviation |
| `for_agent(agent)` | `Vec<&Intonation>` | Per-agent history |
| `average_deviation()` | `f64` | Fleet-wide mean deviation |
| `in_tune_fraction(tolerance)` | `f64` | Fraction within tolerance |
| `beating_frequency(a, b, dim)` | `f64` | Interference between two agents |
| `cascade_deviation(agents, dim)` | `f64` | RMS compound error |
| `worst_quality()` | `IntonationQuality` | Fleet's weakest link |

## The Deeper Idea

The cents metaphor is more than cute. In music, the just noticeable difference for pitch is about 5-10 cents — below that, most humans can't hear the error. Agent systems have the same property: there's a threshold below which the user doesn't notice the imperfection, and above which it degrades the experience.

The cascade deviation formula (RMS) is the same one used in:
- **Audio engineering** — total harmonic distortion
- **Statistics** — standard deviation of a sum
- **Physics** — random walk magnitude

This isn't a coincidence. When independent errors combine, they compound in quadrature. A 5-agent pipeline with 10-cent individual errors produces √5 × 10 ≈ 22 cents of cascade error. That's the difference between "good" and "acceptable" — from a clean performance to one that's slightly off.

The practical implication: **tune your early-stage agents first.** The extractor at the top of the pipeline sets the floor. If it's 20 cents off, everything downstream compounds on top of that. This is why orchestras tune the oboe first — it sets the reference for everyone else.

## Related Crates

- **`agent-groove`** — Timing and feel for scheduling (the *rhythm*)
- **`agent-phrasing`** — Energy contour detection (the *shape*)
- **`agent-orchestration`** — Fleet dynamics as orchestral composition (who plays *loud*)
- **`agent-counterpoint`** — Species counterpoint for coordination (how voices *relate*)
- **`agent-ensemble`** — The proof that musical coordination outperforms mechanical approaches

## License

MIT
