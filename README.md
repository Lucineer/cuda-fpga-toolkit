# cuda-fpga-toolkit

FPGA toolkit — TLMM ternary encoding, COE/MIF generation, Hilbert curve tile mapping for inference silicon

Part of the Cocapn chip layer — hardware design automation and silicon engineering.

## What It Does

### Key Types

- `QuantStats` — core data structure
- `TlmmEncoder` — core data structure
- `CoeGenerator` — core data structure
- `HilbertMapper` — core data structure
- `FpgaResourceEstimate` — core data structure

## Quick Start

```bash
# Clone
git clone https://github.com/Lucineer/cuda-fpga-toolkit.git
cd cuda-fpga-toolkit

# Build
cargo build

# Run tests
cargo test
```

## Usage

```rust
use cuda_fpga_toolkit::*;

// See src/lib.rs for full API
// 10 unit tests included
```

### Available Implementations

- `Ternary` — see source for methods
- `QuantStats` — see source for methods
- `TlmmEncoder` — see source for methods
- `CoeGenerator` — see source for methods
- `HilbertMapper` — see source for methods
- `FpgaResourceEstimate` — see source for methods

## Testing

```bash
cargo test
```

10 unit tests covering core functionality.

## Architecture

This crate is part of the **Cocapn Fleet** — a git-native multi-agent ecosystem.

- **Category**: chip
- **Language**: Rust
- **Dependencies**: See `Cargo.toml`
- **Status**: Active development

## Related Crates

- [cuda-weight-stream](https://github.com/Lucineer/cuda-weight-stream)
- [cuda-thermal-sim](https://github.com/Lucineer/cuda-thermal-sim)
- [cuda-signal-integrity](https://github.com/Lucineer/cuda-signal-integrity)
- [cuda-floorplanner](https://github.com/Lucineer/cuda-floorplanner)
- [cuda-power-estimator](https://github.com/Lucineer/cuda-power-estimator)
- [cuda-clock-tree](https://github.com/Lucineer/cuda-clock-tree)
- [cuda-ir-drop](https://github.com/Lucineer/cuda-ir-drop)
- [cuda-electromigration](https://github.com/Lucineer/cuda-electromigration)
- [cuda-latchup](https://github.com/Lucineer/cuda-latchup)
- [cuda-esd](https://github.com/Lucineer/cuda-esd)
- [cuda-drc](https://github.com/Lucineer/cuda-drc)
- [cuda-pcie](https://github.com/Lucineer/cuda-pcie)
- [cuda-noc](https://github.com/Lucineer/cuda-noc)
- [cuda-packet-buffer](https://github.com/Lucineer/cuda-packet-buffer)
- [cuda-synth](https://github.com/Lucineer/cuda-synth)
- [cuda-verilog](https://github.com/Lucineer/cuda-verilog)
- [cuda-weight-compiler](https://github.com/Lucineer/cuda-weight-compiler)
- [cuda-frozen-intelligence](https://github.com/Lucineer/cuda-frozen-intelligence)

## Fleet Position

```
Casey (Captain)
├── JetsonClaw1 (Lucineer realm — hardware, low-level systems, fleet infrastructure)
├── Oracle1 (SuperInstance — lighthouse, architecture, consensus)
└── Babel (SuperInstance — multilingual scout)
```

## Contributing

This is a fleet vessel component. Fork it, improve it, push a bottle to `message-in-a-bottle/for-jetsonclaw1/`.

## License

MIT

---

*Built by JetsonClaw1 — part of the Cocapn fleet*
*See [cocapn-fleet-readme](https://github.com/Lucineer/cocapn-fleet-readme) for the full fleet roadmap*
