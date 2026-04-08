# Epoch-Link Protocol Specification (v1.0)

This document defines the binary wire format for the epoch-link protocol. All multi-byte integers are encoded in Big-Endian (Network Byte Order) unless otherwise specified.

---

## 1. Frame Layout
A single epoch-link frame is a Self-Contained Epoch (SCE). It is designed to be atomic; the receiver requires no state from previous frames to decode the current one.
```
+----------------+----------------+----------------+----------------+
|  Static Header |  Keyframe (S0) |  Delta Stream  |    Checksum    |
|   (16 Bytes)   |  (Schema Dep.)  | (Variable Bit) |   (4 Bytes)    |
+----------------+----------------+----------------+----------------+
```
---

## 2. Static Header (16 Bytes)
The header provides the global context and identifies the structure of the data payload.

| Offset | Field      | Size | Description |
| :---   | :---       | :--- | :--- |
| 0      | Magic      | 1B   | Constant 0xED (Epoch Data). |
| 1      | Ver/Flags  | 1B   | High 4b: Version (0x1) / Low 4b: Flags. |
| 2      | SessionID  | 2B   | 16-bit mapped ID for the source device. |
| 4      | EpochSeq   | 4B   | Monotonic batch counter. |
| 8      | AnchorTime | 8B   | Unix Microseconds (UTC) baseline for this epoch. |
| 14     | SchemaID   | 1B   | Identifies the Keyframe layout and Delta logic. |
| 15     | EventCount | 1B   | Number of Delta events in this frame (0-255). |

### 2.1 Header Flags (4 bits)
| Bit | Name | Function | Description |
| :--- | :--- | :--- | :--- |
| 0 | ACK | Acknowledgment | If 1, receiver must confirm receipt of this EpochSeq. |
| 1 | COMP | Compression | If 1, indicates use of advanced compression algorithms. |
| 2 | ENC | Encryption | If 1, Keyframe and Delta Stream are encrypted. |
| 3 | RES | Reserved | Must be 0. Reserved for future protocol expansion. |

**Note on Header Byte 1:**
Byte 1 is shared between the **Version** (High Nibble) and **Flags** (Low Nibble). 
Example: `0x15` represents Version 1 (`0x1`) with Encryption and Acknowledgment flags set (`0x4 | 0x1 = 0x5`).

---

## 3. Keyframe (S0)
The Keyframe is a raw binary snapshot of the state at AnchorTime. The size and field order are determined by the **SchemaID**.

- **Schema 0x01 (Tactical):** High-rate flight data (Coords, Velocity, Heading).
- **Schema 0x02 (Health):** Low-rate diagnostics (Battery, Temp, Signal Strength).
- **Schema 0x03 (Payload):** Mission-specific data (Gimbal angles, Detection IDs).

*Note: Because the Header is 16 bytes, if your Keyframe fields (like f64 coordinates) are aligned to 8-byte boundaries within the struct, the CPU can perform direct memory mapping.*

---

## 4. Delta Stream
The Delta Stream contains EventCount events. This section is **bit-packed** and does not respect byte boundaries.

### 4.1 Event Header
1. **Presence Mask (16 bits):** Each bit corresponds to a field in the Schema. 1 = Present; 0 = No change from Keyframe.
2. **Time Offset (Varint):** Microseconds elapsed since AnchorTime.

### 4.2 Field Deltas
Values for fields marked 1 in the mask follow immediately:
- **Numeric Deltas:** (CurrentValue - KeyframeValue).
- **ZigZag Encoding:** Maps signed deltas to unsigned integers to optimize Varint storage.
- **Varint Packing:** Written as Variable-Length Quantities (VLQ).

---

## 5. Encoding Techniques

### 5.1 ZigZag Encoding
Formula: `(n << 1) ^ (n >> 31)`
- Maps small negative/positive changes to small unsigned integers.
- `0 -> 0, -1 -> 1, 1 -> 2, -2 -> 3`

### 5.2 Varint (VLQ) Logic
Each byte uses the MSB as a continuation bit.
- 0xxxxxxx: Last byte.
- 1xxxxxxx: More bytes follow.

---

## 6. Integrity (4 Bytes)
The frame ends with a **CRC-32 (Castagnoli)** checksum.
- **Coverage:** Full frame (Header + Keyframe + Delta Stream).
- **Polynomial:** 0x1EDC6F41.

---

## 7. Constraints
- **MTU Alignment:** Keep total frame under 1200 bytes.
- **Precision:** Diffs are relative to Keyframe, resetting drift every Epoch.
- **Padding:** Pad the final byte of the Delta Stream with 0s to align to the Checksum.

---