//! # agent-intonation
//!
//! Measuring how accurately an agent's output matches its intent.
//!
//! A musician playing in tune produces pitches that match the intended
//! frequencies. An "out of tune" musician is close but not quite right —
//! the notes are recognizable but the performance lacks precision.
//!
//! Agents have the same property. An agent with good intonation produces
//! outputs that closely match its intent. An agent with poor intonation
//! produces approximately-right results that are just off enough to cause
//! problems when composed with other agents' outputs.
//!
//! In music, intonation problems compound: two slightly-out-of-tune players
//! create beating frequencies. In agent systems, intonation problems compound
//! too: two slightly-off agents create cascading errors. This crate measures
//! and tracks that.

/// Cent deviation from perfect intonation (100 cents = one semitone).
#[derive(Debug, Clone)]
pub struct Intonation {
    /// Agent identifier.
    pub agent: String,
    /// Deviation in cents (0 = perfect, negative = flat, positive = sharp).
    pub cents: f64,
    /// What dimension this measures.
    pub dimension: String,
}

impl Intonation {
    pub fn new(agent: &str, cents: f64, dimension: &str) -> Self {
        Self { agent: agent.to_string(), cents, dimension: dimension.to_string() }
    }

    /// Is this within acceptable tolerance? (±10 cents is "in tune")
    pub fn in_tune(&self, tolerance_cents: f64) -> bool {
        self.cents.abs() <= tolerance_cents
    }

    /// Categorize the intonation quality.
    pub fn quality(&self) -> IntonationQuality {
        let abs = self.cents.abs();
        if abs <= 5.0 { IntonationQuality::Perfect }
        else if abs <= 15.0 { IntonationQuality::Good }
        else if abs <= 30.0 { IntonationQuality::Acceptable }
        else if abs <= 50.0 { IntonationQuality::Poor }
        else { IntonationQuality::Unusable }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntonationQuality {
    Perfect = 4,
    Good = 3,
    Acceptable = 2,
    Poor = 1,
    Unusable = 0,
}

/// Track intonation across multiple agents and dimensions.
#[derive(Debug, Clone)]
pub struct IntonationTracker {
    readings: Vec<Intonation>,
}

impl IntonationTracker {
    pub fn new() -> Self { Self { readings: Vec::new() } }

    /// Record a new intonation reading.
    pub fn record(&mut self, agent: &str, cents: f64, dimension: &str) {
        self.readings.push(Intonation::new(agent, cents, dimension));
    }

    /// Get all readings for a specific agent.
    pub fn for_agent(&self, agent: &str) -> Vec<&Intonation> {
        self.readings.iter().filter(|r| r.agent == agent).collect()
    }

    /// Average deviation across all readings.
    pub fn average_deviation(&self) -> f64 {
        if self.readings.is_empty() { return 0.0; }
        self.readings.iter().map(|r| r.cents).sum::<f64>() / self.readings.len() as f64
    }

    /// Fraction of readings that are in tune.
    pub fn in_tune_fraction(&self, tolerance: f64) -> f64 {
        if self.readings.is_empty() { return 1.0; }
        let in_tune = self.readings.iter().filter(|r| r.in_tune(tolerance)).count();
        in_tune as f64 / self.readings.len() as f64
    }

    /// Beating frequency between two agents.
    /// When two agents are slightly out of tune, their outputs create
    /// interference — the "beating" effect. Higher beating = worse.
    pub fn beating_frequency(&self, agent_a: &str, agent_b: &str, dimension: &str) -> f64 {
        let a: Vec<&Intonation> = self.readings.iter()
            .filter(|r| r.agent == agent_a && r.dimension == dimension).collect();
        let b: Vec<&Intonation> = self.readings.iter()
            .filter(|r| r.agent == agent_b && r.dimension == dimension).collect();
        // Beating = |cents_a - cents_b| (difference in deviation)
        if let (Some(ia), Some(ib)) = (a.last(), b.last()) {
            (ia.cents - ib.cents).abs()
        } else { 0.0 }
    }

    /// Compose multiple agents' deviations: the cascade effect.
    /// When N agents each have small deviations, the composed output
    /// can be much worse than any individual deviation.
    pub fn cascade_deviation(&self, agents: &[&str], dimension: &str) -> f64 {
        let deviations: Vec<f64> = agents.iter()
            .filter_map(|&agent| {
                self.readings.iter()
                    .filter(|r| r.agent == agent && r.dimension == dimension)
                    .last()
                    .map(|r| r.cents)
            }).collect();
        if deviations.is_empty() { return 0.0; }
        // Cascade: deviations compound (RMS-like)
        let sum_sq: f64 = deviations.iter().map(|d| d * d).sum();
        sum_sq.sqrt()
    }

    /// Worst intonation quality across all readings.
    pub fn worst_quality(&self) -> IntonationQuality {
        self.readings.iter()
            .map(|r| r.quality())
            .min()
            .unwrap_or(IntonationQuality::Perfect)
    }

    pub fn len(&self) -> usize { self.readings.len() }
    pub fn is_empty(&self) -> bool { self.readings.is_empty() }
}

impl Default for IntonationTracker {
    fn default() -> Self { Self::new() }
}

/// Experiment: intonation quality affects fleet output.
pub fn run_intonation_experiment(
    n_agents: usize,
    base_deviation_cents: f64,
    steps: usize,
) -> (f64, f64) {
    let mut tracker = IntonationTracker::new();
    let agents: Vec<String> = (0..n_agents).map(|i| format!("agent-{}", i)).collect();

    // Each step, each agent produces output with some deviation
    let mut rng_state = 42u64;
    for _ in 0..steps {
        for agent in &agents {
            // Deterministic PRNG for deviation
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let noise = ((rng_state >> 32) as f64 / u32::MAX as f64 - 0.5) * 2.0;
            let deviation = base_deviation_cents * noise;
            tracker.record(agent, deviation, "output_accuracy");
        }
    }

    let in_tune = tracker.in_tune_fraction(15.0);
    let cascade = tracker.cascade_deviation(
        &agents.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        "output_accuracy",
    );
    (in_tune, cascade)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_tune() {
        let i = Intonation::new("a", 3.0, "pitch");
        assert!(i.in_tune(10.0));
    }

    #[test]
    fn test_out_of_tune() {
        let i = Intonation::new("a", 50.0, "pitch");
        assert!(!i.in_tune(10.0));
    }

    #[test]
    fn test_quality_levels() {
        assert_eq!(Intonation::new("a", 3.0, "x").quality(), IntonationQuality::Perfect);
        assert_eq!(Intonation::new("a", 12.0, "x").quality(), IntonationQuality::Good);
        assert_eq!(Intonation::new("a", 25.0, "x").quality(), IntonationQuality::Acceptable);
        assert_eq!(Intonation::new("a", 40.0, "x").quality(), IntonationQuality::Poor);
        assert_eq!(Intonation::new("a", 80.0, "x").quality(), IntonationQuality::Unusable);
    }

    #[test]
    fn test_tracker_average_deviation() {
        let mut t = IntonationTracker::new();
        t.record("a", 10.0, "x");
        t.record("a", -10.0, "x");
        assert!((t.average_deviation()).abs() < 1e-10);
    }

    #[test]
    fn test_tracker_in_tune_fraction() {
        let mut t = IntonationTracker::new();
        t.record("a", 5.0, "x");   // in tune
        t.record("a", 5.0, "x");   // in tune
        t.record("a", 50.0, "x");  // out of tune
        assert!((t.in_tune_fraction(15.0) - 2.0/3.0).abs() < 1e-10);
    }

    #[test]
    fn test_beating_frequency() {
        let mut t = IntonationTracker::new();
        t.record("a", 10.0, "pitch");
        t.record("b", 15.0, "pitch");
        let beating = t.beating_frequency("a", "b", "pitch");
        assert!((beating - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_cascade_deviation() {
        let mut t = IntonationTracker::new();
        t.record("a", 10.0, "x");
        t.record("b", 10.0, "x");
        t.record("c", 10.0, "x");
        let cascade = t.cascade_deviation(&["a", "b", "c"], "x");
        // RMS of [10, 10, 10] = sqrt(300) ≈ 17.3
        assert!(cascade > 10.0); // compounded is worse than individual
    }

    #[test]
    fn test_worst_quality() {
        let mut t = IntonationTracker::new();
        t.record("a", 3.0, "x");
        t.record("b", 40.0, "x");
        assert_eq!(t.worst_quality(), IntonationQuality::Poor);
    }

    #[test]
    fn test_experiment_runs() {
        let (in_tune, cascade) = run_intonation_experiment(5, 10.0, 20);
        assert!(in_tune >= 0.0 && in_tune <= 1.0);
        assert!(cascade >= 0.0);
    }

    #[test]
    fn test_experiment_low_deviation_better() {
        let (good_tune, _) = run_intonation_experiment(5, 5.0, 20);
        let (bad_tune, _) = run_intonation_experiment(5, 50.0, 20);
        assert!(good_tune >= bad_tune);
    }
}
