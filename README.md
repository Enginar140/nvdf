# VDF — Vehicle Description Format

[![CI](https://github.com/Enginar140/nvdf/actions/workflows/ci.yml/badge.svg)](https://github.com/Enginar140/nvdf/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

> **Status: design stage (v0.1 in progress). Nothing is stable yet.**

VDF is an open, simulator-independent format for describing **vehicles and robots** —
fixed-wing UAVs, multirotors, VTOLs, underwater and surface vessels, ground robots and
manipulators — **together with the physics that existing formats leave out**:
aerodynamic coefficient tables, propellers (CT/CQ vs. advance ratio), ESC–motor–battery
power trains, hydrodynamic coefficients and realistic sensor noise (Allan variance).

| | URDF | SDF | MJCF | JSBSim | **VDF** |
|---|---|---|---|---|---|
| Robots / joints | ✓ | ✓ | ✓ | – | ✓ |
| Sensors with noise models | – | partial | partial | – | ✓ |
| Aerodynamic tables | – | plugin XML | – | ✓ | ✓ |
| Propeller / motor / battery | – | plugin XML | – | ✓ | ✓ |
| Explicit units | – | – | – | partial | ✓ |
| Typed schema + validation | – | partial | partial | – | ✓ |

## Design principles
- **Schema first.** Rust types are the source of truth; a JSON Schema is generated from them,
  so any language and any editor can validate and autocomplete documents.
- **Human-writable.** YAML syntax (`.nvdf`), explicit units (`"4.2 kg"`, `"30 deg"`),
  plain numbers are SI (angles in radians).
- **One frame convention.** FLU body / ENU world (ROS REP-103); aviation modules declare
  their own axes explicitly.
- **Composition without a macro language.** `include`, typed `params`, `${param}`
  substitution with arithmetic — no loops, no conditionals.
- **Lossless where possible, loud where not.** Converters to and from URDF, SDF, MJCF and
  JSBSim report every piece of information they cannot carry.

Full design (Turkish): [`docs/tasarim/00_vdf_tasarimi.md`](docs/tasarim/00_vdf_tasarimi.md).

## Example (planned syntax)
```yaml
nvdf: 0.1
kind: model
name: quad-x-450
bodies:
  frame:
    mass: "1.45 kg"
    inertia: {ixx: 0.0123, iyy: 0.0123, izz: 0.0224}
propulsion:
  rotors:
    - {name: m1, body: frame, spin: ccw,
       motor: catalog://motors/tmotor-mn3110-700kv, prop: catalog://props/apc-10x4.7}
```

## Repository layout
| Path | Contents |
|---|---|
| `crates/nvdf` | Library: parse, resolve, validate |
| `crates/nvdf-cli` | `nvdf` command-line tool |
| `docs/` | Design documents |

## Türkçe özet
VDF; sabit kanat İHA, multikopter, su altı aracı, kara robotu ve robot kolunu,
aerodinamik tabloları, pervane/motor/batarya zinciri ve gerçekçi sensör gürültüsüyle
birlikte tanımlayan, simülatörden bağımsız açık bir formattır. İlk kullanıcısı NSIM
simülatörüdür. Tasarım belgesi Türkçedir.

## License
Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual
licensed as above, without any additional terms or conditions.
