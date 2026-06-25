//! Compute-job pricing adapter.
//!
//! Turns a heterogeneous, SLA-bound compute job into a HANZO price (wei, 18
//! decimals) via the Hamiltonian Market Maker — NOT a constant-product
//! (`x*y=k`) AMM. Compute is perishable, heterogeneous, and SLA-bound; an AMM
//! prices a fungible, storable pair and cannot express scarcity convexity,
//! deadline pressure, or per-resource isolation. This module is the
//! compute-domain ADAPTER over [`crate::HamiltonianDynamics`]: it maps
//! `(demand q, shadow price p)` into the Hamiltonian phase space, evolves the
//! energy-conserving (symplectic) dynamics to the energy-modulated equilibrium
//! price via [`HamiltonianDynamics::calculate_price`], then modulates by SLA
//! tightness (perishability) and quality tier, and clamps to the bounds from
//! `docs/AI_TOKEN_ECONOMICS.md`.
//!
//! All Hamiltonian mechanics live in [`crate::hamiltonian`]; nothing here
//! re-implements them. The pricing entry point [`price`] is fully
//! deterministic for a given `(job, market)` — it uses only the noise-free
//! integrator path (no `rng`), so the HMM equilibrium is reproducible and
//! testable.

use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};

use crate::hamiltonian::{AnharmonicPotential, HamiltonianDynamics};

/// One HANZO in wei (18 decimals), matching `hanzo-mining` token semantics.
pub const WEI_PER_HANZO: u128 = 1_000_000_000_000_000_000;

/// Price floor in wei. Mirrors `AI_TOKEN_ECONOMICS.md` `min_price = 0.00001`
/// HANZO/unit — a non-zero floor so a quote is never free.
pub const MIN_PRICE_WEI: u128 = WEI_PER_HANZO / 100_000; // 1e13 wei

/// Price ceiling in wei. Mirrors `AI_TOKEN_ECONOMICS.md` `max_price = 0.01`
/// HANZO/unit — an unconditional cap so a quote is never unbounded.
pub const MAX_PRICE_WEI: u128 = WEI_PER_HANZO / 100; // 1e16 wei

/// Price elasticity (`AI_TOKEN_ECONOMICS.md` §"Dynamic Pricing Updates").
const ELASTICITY: f64 = 0.5;

/// Heterogeneous compute resource kinds. Each kind prices on its own
/// [`MarketState`]; prices never cross between kinds (property: heterogeneity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    /// CPU cores / vCPU-seconds.
    Cpu,
    /// GPU accelerators / GPU-seconds.
    Gpu,
    /// RAM (GB).
    Memory,
    /// Persistent storage (GB).
    Storage,
    /// Network egress (GB).
    Bandwidth,
}

/// Quality / privacy tier. Tighter isolation earns a higher multiplier,
/// matching the privacy-tier table in `AI_TOKEN_ECONOMICS.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum QualityTier {
    /// Open, no attestation. 1.0x.
    #[default]
    Open,
    /// Encrypted at rest. 1.2x.
    AtRest,
    /// CPU TEE (SGX/SEV). 1.5x.
    CpuTee,
    /// GPU TEE (H100 CC). 1.8x.
    GpuTee,
    /// TEE-I/O (Blackwell). 2.0x.
    TeeIo,
}

impl QualityTier {
    /// Tier multiplier from the economics table.
    fn multiplier(self) -> f64 {
        match self {
            QualityTier::Open => 1.0,
            QualityTier::AtRest => 1.2,
            QualityTier::CpuTee => 1.5,
            QualityTier::GpuTee => 1.8,
            QualityTier::TeeIo => 2.0,
        }
    }
}

/// A heterogeneous, SLA-bound compute job to be priced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputeJob {
    /// Which resource pool this job draws from.
    pub kind: ResourceKind,
    /// Units demanded by this job (cores, GPU-seconds, GB, …). Must be > 0.
    pub units: f64,
    /// Seconds until the job's deadline / latency SLA. A tighter (smaller)
    /// deadline means the work is more perishable and prices higher. `None`
    /// means best-effort (no urgency premium).
    pub deadline_secs: Option<f64>,
    /// Quality / privacy tier required.
    pub tier: QualityTier,
}

impl ComputeJob {
    /// Best-effort, open-tier job for `units` of `kind`.
    pub fn new(kind: ResourceKind, units: f64) -> Self {
        Self {
            kind,
            units,
            deadline_secs: None,
            tier: QualityTier::default(),
        }
    }

    /// Attach a deadline/latency SLA in seconds.
    pub fn with_deadline_secs(mut self, deadline_secs: f64) -> Self {
        self.deadline_secs = Some(deadline_secs);
        self
    }

    /// Attach a quality/privacy tier.
    pub fn with_tier(mut self, tier: QualityTier) -> Self {
        self.tier = tier;
        self
    }
}

/// Market state for a single resource kind at quote time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketState {
    /// Which resource pool this state describes.
    pub kind: ResourceKind,
    /// Available units right now (supply). Must be > 0 to price.
    pub supply: f64,
    /// Outstanding demand / queue depth right now.
    pub demand: f64,
    /// Base price per unit, in wei. The HMM modulates around this anchor.
    pub base_price_wei: u128,
}

impl MarketState {
    /// New market state.
    pub fn new(kind: ResourceKind, supply: f64, demand: f64, base_price_wei: u128) -> Self {
        Self {
            kind,
            supply,
            demand,
            base_price_wei,
        }
    }
}

/// A HANZO price in wei (18 decimals). Always in `[MIN_PRICE_WEI,
/// MAX_PRICE_WEI]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HanzoPrice(pub u128);

impl HanzoPrice {
    /// Price in wei.
    pub fn wei(self) -> u128 {
        self.0
    }

    /// Price in whole HANZO (lossy display only).
    pub fn as_hanzo(self) -> f64 {
        self.0 as f64 / WEI_PER_HANZO as f64
    }
}

/// Marker substring in the `anyhow` error a kind mismatch produces, so callers
/// (and tests) can distinguish it from numeric-input rejection without a bespoke
/// error enum — the crate's error idiom is `anyhow`.
pub const ERR_KIND_MISMATCH: &str = "resource kind mismatch";

// --- HMM integration constants (deterministic path) ---------------------------

/// Phase-space dimension: pricing is a 1-D conservative system (one resource,
/// one shadow price), so a single generalized coordinate suffices.
const PHASE_DIM: usize = 1;
/// Hamiltonian energy scale; matches the crate default ([`crate::Config`]).
const ENERGY_SCALE: f64 = 1.0;
/// Quadratic stiffness of the shadow-price potential well.
const POTENTIAL_K2: f64 = 1.0;
/// Quartic stiffness — the anharmonic term makes the restoring force
/// super-linear in displacement, so deep scarcity bites convexly. A pure
/// quadratic (or `x*y=k`) cannot express this perishability convexity.
const POTENTIAL_K4: f64 = 0.5;
/// Symplectic integration step.
const DT: f64 = 0.05;
/// Number of leapfrog steps to settle toward equilibrium. Friction
/// (set inside [`HamiltonianDynamics`]) dissipates transient momentum so the
/// readout reflects the energy-modulated equilibrium, not the initial kick.
const SETTLE_STEPS: usize = 200;

/// Price a heterogeneous, SLA-bound compute job via the Hamiltonian Market
/// Maker. Deterministic for a given `(job, market)`.
///
/// HMM mapping (NOT `x*y=k`):
/// 1. `imbalance = (demand - supply) / supply` — the economics-doc demand
///    pressure; it seeds the generalized coordinate `q` and momentum `p`.
/// 2. The Hamiltonian `H(q,p) = p²/2 + V(q)` with anharmonic `V` is evolved by
///    the existing symplectic integrator until friction settles it.
/// 3. [`HamiltonianDynamics::calculate_price`] reads the energy-modulated
///    equilibrium: `base · (1+a‖q‖)(1+b⟨p⟩)(1+c·tanh(E/scale))`, where
///    `E = T + V` is the total energy — scarcity raises `E`, raising price.
/// 4. SLA tightness (perishability) and tier multipliers scale the result,
///    which is then clamped to `[MIN_PRICE_WEI, MAX_PRICE_WEI]`.
pub fn price(job: &ComputeJob, market: &MarketState) -> Result<HanzoPrice> {
    ensure!(
        job.kind == market.kind,
        "{ERR_KIND_MISMATCH}: job is {:?} but market is {:?}",
        job.kind,
        market.kind
    );
    validate_finite_positive("job.units", job.units)?;
    validate_finite_positive("market.supply", market.supply)?;
    validate_finite_nonneg("market.demand", market.demand)?;
    if let Some(d) = job.deadline_secs {
        validate_finite_positive("job.deadline_secs", d)?;
    }
    if market.base_price_wei == 0 {
        bail!("market.base_price_wei must be > 0");
    }

    // (1) Effective demand for this quote includes the job's own draw on the
    //     pool, so a larger job sees a deeper imbalance (its marginal scarcity).
    let effective_demand = market.demand + job.units;
    let imbalance = (effective_demand - market.supply) / market.supply;

    // (2) Seed the HMM phase space. The generalized coordinate is the *shadow-
    //     price displacement*, a normalized quantity in (-1, 1): `tanh(imbalance)`.
    //     Raw imbalance is unbounded (demand can dwarf supply), but the
    //     shadow-price coordinate saturates — a maximally-scarce pool is pinned
    //     at the edge of the potential well, and the symplectic integrator stays
    //     in its stable basin (an un-normalized quartic seed diverges to NaN).
    //     Position = displacement; momentum = price pressure in the same
    //     direction, so kinetic and potential energy both grow with |imbalance|.
    let displacement = imbalance.tanh();
    let mut dynamics = HamiltonianDynamics::new(ENERGY_SCALE, PHASE_DIM);
    dynamics.set_potential(Box::new(AnharmonicPotential::new(
        POTENTIAL_K2,
        POTENTIAL_K4,
    )));
    {
        // Public phase-space fields; deterministic seeding (no rng).
        let mut ps = dynamics.get_phase_space().context("read HMM phase space")?;
        ps.positions[0] = displacement;
        ps.momenta[0] = ELASTICITY * displacement;
        dynamics
            .set_phase_space(ps)
            .context("seed HMM phase space")?;
    }

    // (3) Evolve the conservative system to its energy-modulated equilibrium.
    for _ in 0..SETTLE_STEPS {
        dynamics.evolve(DT).context("evolve HMM dynamics")?;
    }

    // The base price the HMM modulates is the per-unit anchor in whole HANZO.
    let base_hanzo = market.base_price_wei as f64 / WEI_PER_HANZO as f64;
    let hmm_unit_hanzo = dynamics
        .calculate_price(base_hanzo)
        .context("read HMM equilibrium price")?;

    // (4) Compose the HMM energy premium with the scarcity direction so the
    //     per-unit price is monotone in imbalance, then apply perishability and
    //     tier. `calculate_price` modulates *up* from base via total energy
    //     E = T + V (scarcity raises E); the bounded `scarcity` factor restores
    //     a discount for slack supply (imbalance < 0) around the anchor.
    let scarcity = (1.0 + (ELASTICITY * imbalance).tanh()).max(f64::MIN_POSITIVE);
    let unit_hanzo = hmm_unit_hanzo * scarcity;

    // Perishability: tighter deadline ⇒ higher price; best-effort pays none.
    let urgency = sla_multiplier(job.deadline_secs);
    // Quality / privacy tier multiplier.
    let tier = job.tier.multiplier();

    // Total per-unit price, scaled by units demanded, then clamped to bounds.
    let total_hanzo = unit_hanzo * urgency * tier * job.units;
    Ok(HanzoPrice(clamp_to_wei(total_hanzo)))
}

/// SLA tightness premium. Tighter (smaller) `deadline_secs` ⇒ larger
/// multiplier. Best-effort (`None`) ⇒ 1.0. The premium is bounded in
/// `[1.0, 1.0 + URGENCY_MAX]` and strictly decreasing in the deadline horizon.
fn sla_multiplier(deadline_secs: Option<f64>) -> f64 {
    /// Largest urgency premium (at deadline → 0): +100%.
    const URGENCY_MAX: f64 = 1.0;
    /// Horizon (seconds) at which urgency has decayed to half of `URGENCY_MAX`.
    const HALF_LIFE_SECS: f64 = 60.0;
    match deadline_secs {
        None => 1.0,
        Some(d) => {
            // 1 + URGENCY_MAX * HALF_LIFE / (HALF_LIFE + d): strictly
            // decreasing in d, → 1+URGENCY_MAX as d→0, → 1 as d→∞.
            1.0 + URGENCY_MAX * HALF_LIFE_SECS / (HALF_LIFE_SECS + d)
        }
    }
}

/// Clamp a whole-HANZO price to the wei bounds. Saturating conversion avoids
/// overflow/underflow; the result is always in `[MIN_PRICE_WEI, MAX_PRICE_WEI]`.
fn clamp_to_wei(hanzo: f64) -> u128 {
    if !hanzo.is_finite() || hanzo <= 0.0 {
        return MIN_PRICE_WEI;
    }
    let wei_f = hanzo * WEI_PER_HANZO as f64;
    let wei = if wei_f >= MAX_PRICE_WEI as f64 {
        MAX_PRICE_WEI
    } else {
        wei_f as u128
    };
    wei.clamp(MIN_PRICE_WEI, MAX_PRICE_WEI)
}

fn validate_finite_positive(name: &str, v: f64) -> Result<()> {
    ensure!(
        v.is_finite() && v > 0.0,
        "{name} must be finite and > 0, got {v}"
    );
    Ok(())
}

fn validate_finite_nonneg(name: &str, v: f64) -> Result<()> {
    ensure!(
        v.is_finite() && v >= 0.0,
        "{name} must be finite and >= 0, got {v}"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Base anchor: 0.001 HANZO/unit, comfortably inside [min, max].
    const BASE: u128 = WEI_PER_HANZO / 1_000; // 1e15 wei

    fn market(kind: ResourceKind, supply: f64, demand: f64) -> MarketState {
        MarketState::new(kind, supply, demand, BASE)
    }

    /// (a) Monotonic in scarcity: holding demand fixed, lower supply ⇒
    /// strictly higher (or equal) price.
    #[test]
    fn property_a_monotonic_in_scarcity() {
        let job = ComputeJob::new(ResourceKind::Gpu, 1.0);
        let mut last = 0u128;
        // Supply shrinking from abundant to scarce.
        for supply in [1000.0, 500.0, 200.0, 100.0, 50.0, 20.0, 10.0] {
            let p = price(&job, &market(ResourceKind::Gpu, supply, 100.0))
                .unwrap()
                .wei();
            assert!(
                p >= last,
                "price must not fall as supply shrinks: supply={supply} price={p} prev={last}"
            );
            last = p;
        }
        // And strictly higher at the scarce end than the abundant end.
        let abundant = price(&job, &market(ResourceKind::Gpu, 1000.0, 100.0))
            .unwrap()
            .wei();
        let scarce = price(&job, &market(ResourceKind::Gpu, 10.0, 100.0))
            .unwrap()
            .wei();
        assert!(
            scarce > abundant,
            "scarce={scarce} must exceed abundant={abundant}"
        );
    }

    /// (b) Monotonic in demand: holding supply fixed, higher demand/queue ⇒
    /// higher price.
    #[test]
    fn property_b_monotonic_in_demand() {
        let job = ComputeJob::new(ResourceKind::Cpu, 1.0);
        let mut last = 0u128;
        for demand in [0.0, 50.0, 100.0, 200.0, 400.0, 800.0] {
            let p = price(&job, &market(ResourceKind::Cpu, 200.0, demand))
                .unwrap()
                .wei();
            assert!(
                p >= last,
                "price must not fall as demand rises: demand={demand} price={p} prev={last}"
            );
            last = p;
        }
        let low = price(&job, &market(ResourceKind::Cpu, 200.0, 0.0))
            .unwrap()
            .wei();
        let high = price(&job, &market(ResourceKind::Cpu, 200.0, 800.0))
            .unwrap()
            .wei();
        assert!(
            high > low,
            "high-demand={high} must exceed low-demand={low}"
        );
    }

    /// (c) SLA / perishability: a tighter deadline ⇒ higher price than a loose
    /// deadline for the same job and market.
    #[test]
    fn property_c_sla_perishability() {
        let m = market(ResourceKind::Gpu, 200.0, 100.0);
        let tight = ComputeJob::new(ResourceKind::Gpu, 1.0).with_deadline_secs(5.0);
        let loose = ComputeJob::new(ResourceKind::Gpu, 1.0).with_deadline_secs(3600.0);
        let best_effort = ComputeJob::new(ResourceKind::Gpu, 1.0);

        let p_tight = price(&tight, &m).unwrap().wei();
        let p_loose = price(&loose, &m).unwrap().wei();
        let p_best = price(&best_effort, &m).unwrap().wei();

        assert!(
            p_tight > p_loose,
            "tight deadline {p_tight} must exceed loose deadline {p_loose}"
        );
        assert!(
            p_loose > p_best,
            "loose deadline {p_loose} must exceed best-effort {p_best}"
        );
        // Strictly monotone across a sweep of deadlines (tighter ⇒ dearer).
        let mut prev = u128::MAX;
        for d in [1.0, 10.0, 60.0, 300.0, 3600.0, 86400.0] {
            let p = price(
                &ComputeJob::new(ResourceKind::Gpu, 1.0).with_deadline_secs(d),
                &m,
            )
            .unwrap()
            .wei();
            assert!(
                p < prev,
                "deadline {d}s price {p} must be below tighter {prev}"
            );
            prev = p;
        }
    }

    /// (d) Bounded: price is always within `[MIN_PRICE_WEI, MAX_PRICE_WEI]`,
    /// never zero, never unbounded — even under extreme inputs.
    #[test]
    fn property_d_bounded() {
        // Extreme scarcity + tight SLA + top tier + huge job: must cap, not blow up.
        let hot = ComputeJob::new(ResourceKind::Gpu, 1_000_000.0)
            .with_deadline_secs(0.001)
            .with_tier(QualityTier::TeeIo);
        let starved = market(ResourceKind::Gpu, 0.000_001, 1_000_000.0);
        let p_hot = price(&hot, &starved).unwrap().wei();
        assert!(p_hot <= MAX_PRICE_WEI, "must clamp at max: {p_hot}");
        assert!(p_hot >= MIN_PRICE_WEI, "must stay above min: {p_hot}");
        assert_eq!(
            p_hot, MAX_PRICE_WEI,
            "saturating extreme must hit the ceiling"
        );

        // Extreme slack + best-effort + open tier + tiny job: must floor, not zero.
        let cold = ComputeJob::new(ResourceKind::Storage, 0.000_001);
        let glut = market(ResourceKind::Storage, 1_000_000.0, 0.0);
        let p_cold = price(&cold, &glut).unwrap().wei();
        assert!(p_cold >= MIN_PRICE_WEI, "must clamp at min: {p_cold}");
        assert!(p_cold <= MAX_PRICE_WEI, "must stay below max: {p_cold}");
        assert!(p_cold > 0, "price is never zero");

        // A spread of ordinary inputs always lands inside the band.
        for &kind in &[
            ResourceKind::Cpu,
            ResourceKind::Gpu,
            ResourceKind::Memory,
            ResourceKind::Storage,
            ResourceKind::Bandwidth,
        ] {
            for supply in [1.0, 50.0, 500.0] {
                for demand in [0.0, 100.0, 1000.0] {
                    let p = price(&ComputeJob::new(kind, 4.0), &market(kind, supply, demand))
                        .unwrap()
                        .wei();
                    assert!(
                        (MIN_PRICE_WEI..=MAX_PRICE_WEI).contains(&p),
                        "out of band: kind={kind:?} supply={supply} demand={demand} price={p}"
                    );
                }
            }
        }
    }

    /// (e) Heterogeneity: different resource kinds price independently. Moving
    /// a CPU pool's demand must not change a GPU pool's price, and a kind
    /// mismatch between job and market is rejected.
    #[test]
    fn property_e_heterogeneity() {
        let gpu_job = ComputeJob::new(ResourceKind::Gpu, 1.0);
        let gpu_market = market(ResourceKind::Gpu, 200.0, 100.0);

        // Price the GPU pool while a *separate* CPU pool swings wildly.
        let gpu_price_before = price(&gpu_job, &gpu_market).unwrap().wei();
        let _cpu_calm = price(
            &ComputeJob::new(ResourceKind::Cpu, 1.0),
            &market(ResourceKind::Cpu, 1000.0, 0.0),
        )
        .unwrap();
        let _cpu_hot = price(
            &ComputeJob::new(ResourceKind::Cpu, 1.0),
            &market(ResourceKind::Cpu, 1.0, 100_000.0),
        )
        .unwrap();
        let gpu_price_after = price(&gpu_job, &gpu_market).unwrap().wei();

        assert_eq!(
            gpu_price_before, gpu_price_after,
            "GPU price must not move because a CPU pool moved"
        );

        // A job and a market for different kinds must be rejected (no shared
        // state, no accidental cross-kind pricing).
        let mismatch = price(&gpu_job, &market(ResourceKind::Cpu, 200.0, 100.0));
        let err = mismatch.expect_err("kind mismatch must be rejected");
        assert!(
            err.to_string().contains(ERR_KIND_MISMATCH),
            "expected kind-mismatch error, got: {err}"
        );
    }

    /// (f) Deterministic: identical `(job, market)` yields byte-identical
    /// prices across repeated calls (noise-free HMM path, no rng).
    #[test]
    fn property_f_deterministic() {
        let job = ComputeJob::new(ResourceKind::Gpu, 3.0)
            .with_deadline_secs(42.0)
            .with_tier(QualityTier::GpuTee);
        let m = market(ResourceKind::Gpu, 137.0, 211.0);

        let first = price(&job, &m).unwrap();
        for _ in 0..1000 {
            assert_eq!(
                price(&job, &m).unwrap(),
                first,
                "pricing must be deterministic for a fixed (job, market)"
            );
        }
    }

    /// Tier ordering: higher privacy tier ⇒ higher price, all else equal.
    #[test]
    fn tier_monotonic() {
        let m = market(ResourceKind::Gpu, 200.0, 100.0);
        let mut prev = 0u128;
        for tier in [
            QualityTier::Open,
            QualityTier::AtRest,
            QualityTier::CpuTee,
            QualityTier::GpuTee,
            QualityTier::TeeIo,
        ] {
            let p = price(&ComputeJob::new(ResourceKind::Gpu, 1.0).with_tier(tier), &m)
                .unwrap()
                .wei();
            assert!(
                p > prev,
                "tier {tier:?} price {p} must exceed lower tier {prev}"
            );
            prev = p;
        }
    }

    /// Bad input is rejected explicitly (no panic, no silent zero).
    #[test]
    fn rejects_bad_input() {
        let m = market(ResourceKind::Cpu, 100.0, 10.0);
        assert!(price(&ComputeJob::new(ResourceKind::Cpu, 0.0), &m).is_err());
        assert!(price(&ComputeJob::new(ResourceKind::Cpu, f64::NAN), &m).is_err());
        assert!(price(
            &ComputeJob::new(ResourceKind::Cpu, 1.0),
            &MarketState::new(ResourceKind::Cpu, 0.0, 10.0, BASE)
        )
        .is_err());
        assert!(price(
            &ComputeJob::new(ResourceKind::Cpu, 1.0),
            &MarketState::new(ResourceKind::Cpu, 100.0, 10.0, 0)
        )
        .is_err());
        assert!(price(
            &ComputeJob::new(ResourceKind::Cpu, 1.0).with_deadline_secs(-1.0),
            &m
        )
        .is_err());
    }
}
