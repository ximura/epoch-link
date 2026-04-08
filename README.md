# epoch-link 🛰️

**A high-efficiency, disconnected-first telemetry protocol designed for tactical edge devices and unstable SATCOM links.**

`epoch-link` is a specialized binary codec designed to synchronize high-frequency sensor data from edge devices (drones, UGVs, sensors) to a central command layer. It prioritizes **data density** and **state recovery** in high-latency, low-bandwidth, and intermittent network environments.

---

## 🏗️ The Problem
Traditional telemetry (JSON/REST, or standard Protobuf over MQTT) often fails in tactical environments for two primary reasons:
1.  **Overhead:** Repeating headers and identifiers in every packet wastes expensive satellite bandwidth.
2.  **State Dependency:** Many incremental sync methods require a persistent connection. If a "delta" packet is lost, subsequent data becomes uninterpretable until a full state is re-requested (the "NACK" storm).

## 🚀 The Solution: Self-Contained Epochs (SCE)
`epoch-link` organizes data into **Self-Contained Epochs**. Each transmission frame is an independent "source of truth."

* **Keyframing:** Each batch begins with a full state snapshot (The Anchor).
* **Temporal Compression:** Subsequent events in the batch are encoded as bit-packed increments relative to the Anchor.
* **Atomic Delivery:** If you receive a batch, you have the full history of that window—no previous context or "handshaking" required to decode the current state.

---

## 🛠️ Technical Design

### 1. Frame Anatomy
| Section | Content | Optimization |
| :--- | :--- | :--- |
| **Epoch Header** | SessionID, SeqNum, AnchorTimestamp | 16-bit mapping, 64-bit micro-precision anchor. |
| **The Anchor** | Full Sensor Snapshot | Raw binary representation of the initial state (Event 0). |
| **The Delta Stream** | Event [1..N] | Bit-packed offsets, presence masks, and ZigZag encoded diffs. |
| **Integrity** | CRC32 Checksum | Ensures packet validity over noisy links. |

### 2. Core Optimization Techniques
* **ZigZag/Varint Encoding:** Small signed changes (e.g., altitude +/- 1m) are compressed into single-byte representations.
* **Presence Bitmasks:** Instead of sending "null" or "0" for unchanged sensors, a 16-bit mask indicates exactly which fields are present in the diff.
* **Linear Time Prediction:** Time deltas are encoded as jitter deviations from an expected sampling frequency, reducing timing overhead to **< 4 bits per event**.

---

## 💻 Implementation Goals (Rust)

This repository serves as a reference implementation of the `epoch-link` codec in **Rust**, focusing on:

* **Zero-Copy Deserialization:** Utilizing `std::io::Cursor` and specialized traits to minimize allocations.
* **Memory Safety:** Leveraging Rust's ownership model to ensure safe buffer handling during bit-stream parsing.
* **Performance:** Designed to run on resource-constrained ARM-based edge hardware (e.g., NVIDIA Jetson, Raspberry Pi Compute Module).

---

## 📈 Projected Impact
| Metric | Standard Telemetry | `epoch-link` |
| :--- | :--- | :--- |
| **Payload Size** | 100% (Baseline) | **~25% - 30%** |
| **Reliability** | State-dependent | **Stateless Recovery** |
| **Protocol Overhead** | High (TCP/HTTP) | **Ultra-Low (UDP/QUIC/Raw)** |

---

## 🧬 Repository Structure
* `/spec`: Detailed bit-level protocol specification.
* `/src/codec`: The core encoding/decoding logic (The "Squeezer").
* `/src/models`: Telemetry schema definitions.
* `/examples`: Sample implementation of a drone-to-ground-station sync.

---

> **Note:** This project is currently in the architectural design phase, targeting high-performance DefTech and industrial IoT applications.
