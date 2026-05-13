# hanzo-hmm — Hidden Markov Model + Hamiltonian MarketMaker

Pure Hidden Markov Model primitives plus a Hamiltonian MarketMaker built on
top of them, used in the Hanzo network to price heterogeneous compute.

> **Naming.** This crate is *not* an LLM engine. It is the pricing /
> routing layer. The actual model-serving engine lives in
> `~/work/hanzo/engine` (a `mistral.rs` fork exposed as `hanzo-engine`).

## Two layers, one crate

| Layer | Module | Purpose |
|-------|--------|---------|
| HMM core | `hmm_core` | Viterbi, forward-backward, Baum-Welch on `HiddenMarkovModel<S, O>` |
| MarketMaker | `lib::MarketMaker` | Hamiltonian price dynamics + BitDelta adapters + active-inference routing |

The HMM layer is standalone and reusable for any sequence modelling task
(state detection, sequence prediction, anomaly detection). The MarketMaker
layer composes HMM regime detection with Hamiltonian mechanics and
BitDelta-quantized per-tenant adapters to set prices and routing
decisions across compute classes.

## HMM core usage

```rust
use hanzo_hmm::HiddenMarkovModel;

let states       = vec!["Fair", "Loaded"];
let observations = vec![1, 2, 3, 4, 5, 6];
let initial      = vec![0.5, 0.5];
let transitions  = vec![
    vec![0.7, 0.3],
    vec![0.4, 0.6],
];
let emissions = vec![
    vec![1.0 / 6.0; 6],
    vec![0.1, 0.1, 0.1, 0.1, 0.1, 0.5],
];

let hmm = HiddenMarkovModel::new(
    states, observations, initial, transitions, emissions,
)?;

let observed = vec![6, 6, 6, 1, 2];
let path     = hmm.viterbi(&observed)?;          // most likely state sequence
let p        = hmm.forward(&observed)?;          // P(observations | model)
let _seq     = hmm.generate(100);                // sample observations
```

## MarketMaker usage

```rust
use hanzo_hmm::{MarketMaker, Config, RoutingRequest, UserPreferences, PerformanceRequirements};

let mm = MarketMaker::new(Config::default()).await?;

let decision = mm.route_request("tenant-42", &RoutingRequest {
    input: "...".into(),
    context: vec![],
    preferences: UserPreferences {
        max_latency_ms: Some(1_500),
        max_cost_per_token: Some(0.001),
        preferred_models: vec![],
        quality_threshold: 0.8,
    },
    requirements: PerformanceRequirements {
        min_tokens_per_second: Some(40.0),
        max_memory_gb: Some(48.0),
        requires_function_calling: false,
        requires_vision: false,
    },
    observations: vec![0.1, 0.2, 0.05, 0.4],
}).await?;
```

## Algorithms

- **Viterbi** — most likely state sequence given observations
- **Forward** — `P(observations | model)`
- **Backward** — backward probabilities per state
- **Baum-Welch** — parameter learning from observation sequences

```rust
let path  = hmm.viterbi(&observations)?;
let prob  = hmm.forward(&observations)?;
let beta  = hmm.backward(&observations)?;
hmm.baum_welch(&training_sequences, max_iterations, tolerance)?;
```

## License

MIT
