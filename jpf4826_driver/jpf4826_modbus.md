# JPF4826 4-Wire PWM DC Fan Temperature Control Speed Controller

## RS485 Communication Protocol

### Protocol Standard

**Modbus-RTU** (slave device, passive response only)

### Serial Port Parameters

| Parameter | Value |
| --------- | ----- |
| Baud Rate | 9600  |
| Data Bits | 8     |
| Parity    | None  |
| Stop Bits | 1     |

---

## Modbus Register Map

| Address    | Description                | Data Type | R/W | Function Code |
| ---------- | -------------------------- | --------- | --- | ------------- |
| **0x0000** | Current Temperature        | INT16     | R   | 0x03          |
| **0x0001** | Fan Status                 | BITMAP    | R   | 0x03/0x02     |
| **0x0002** | Modbus Address             | UINT16    | R/W | 0x03/0x06     |
| **0x0003** | Manual Speed Control       | UINT16    | R/W | 0x03/0x06     |
| **0x0004** | Start/Full Temp (Combined) | UINT16    | R/W | 0x03/0x06     |
| **0x0005** | Work Mode                  | UINT16    | R/W | 0x03/0x06     |
| **0x0006** | Fan Quantity               | UINT16    | R/W | 0x03/0x06     |
| **0x0007** | Fan1 Speed (RPM)           | UINT16    | R   | 0x03          |
| **0x0008** | Fan2 Speed (RPM)           | UINT16    | R   | 0x03          |
| **0x0009** | Fan3 Speed (RPM)           | UINT16    | R   | 0x03          |
| **0x000A** | Fan4 Speed (RPM)           | UINT16    | R   | 0x03          |
| **0x000B** | PWM Frequency Select       | UINT16    | R/W | 0x03/0x06     |
| **0x000C** | Start Temperature          | INT16     | R/W | 0x03/0x06     |
| **0x000D** | Full Speed Temperature     | INT16     | R/W | 0x03/0x06     |
| **0x000E** | Fan Fault Code             | BITMAP    | R   | 0x03          |
| **0x0020** | Reset Controller           | UINT16    | W   | 0x06          |

### Register Details

#### 0x0000 - Current Temperature

- **Offset:** +40
- **Actual Temperature (°C) = Register Value - 40**
- Example: Register = 0x0047 (71 decimal) → Temperature = 71 - 40 = 31°C

#### 0x0001 - Fan Status (Bitmap)

- **Bit 0:** Fan 1 status (0=stopped, 1=running)
- **Bit 1:** Fan 2 status
- **Bit 2:** Fan 3 status
- **Bit 3:** Fan 4 status

#### 0x0002 - Modbus Address

- **Range:** 0x0001 to 0x00FE
- **Broadcast Address:** 0xFFFF supported

#### 0x0003 - Manual Speed Control / Mode

- **Write Range:** 0x0000 to 0x0064 (0-100% manual speed), 0xFFFF to exit manual mode
- **Read Behavior:** When in temperature mode, this register contains the current calculated speed value (not 0xFFFF)
- **Important:** The operating mode (Temperature vs Manual) cannot be determined by reading this register
- **Note:** Temperature control is disabled while in manual mode
- **Exit Manual Mode:** Write 0xFFFF or power cycle to restore temperature control

#### 0x0004 - Start/Full Temperature (Combined)

- **High Byte:** Start temperature (L)
- **Low Byte:** Full speed temperature (H)
- **Range:** 0x1415 to 0xA09F
- **Offset:** +40 for both bytes
- **Example:** 0x465A → Start=70-40=30°C, Full=90-40=50°C

#### 0x0005 - Work Mode

- **0x0000:** Fan shutdown mode (fan stops below L-3°C)
- **0x0001:** Minimum speed mode (fan maintains 20% below L-3°C)

#### 0x0006 - Fan Quantity

- **Range:** 0x0001 to 0x0004
- **0x0000:** Disables fault detection

#### 0x0007 to 0x000A - Fan Speed (RPM)

- **Calculation:** RPM = 60 × N / 2
  - N = pulses per second from fan tachometer
  - /2 assumes 2 pulses per revolution (standard for most fans)
- **Note:** Special fans may require custom RPM calculation

#### 0x000B - PWM Frequency Selection

- **Range:** 0x0000 to 0x0005 (default: 0x0005)
- **Values:**
  - 0x0000 = 500Hz
  - 0x0001 = 1kHz
  - 0x0002 = 2kHz
  - 0x0003 = 5kHz
  - 0x0004 = 10kHz
  - 0x0005 = 25kHz

#### 0x000C - Start Temperature

- **Range:** 0x0014 to 0x00A0 (-20°C to 120°C)
- **Offset:** +40
- **Actual Temperature = Register Value - 40**

#### 0x000D - Full Speed Temperature

- **Range:** 0x0014 to 0x00A0 (-20°C to 120°C)
- **Offset:** +40
- **Constraint:** Must be greater than start temperature

#### 0x000E - Fan Fault Code (Bitmap)

- **Bit 0:** Fan 1 (0=fault, 1=normal)
- **Bit 1:** Fan 2
- **Bit 2:** Fan 3
- **Bit 3:** Fan 4

#### 0x0020 - Reset Controller

- **Write 0x00AA to reset/restart controller**

---

## Modbus Command Examples (Hexadecimal)

### 1. Read Current Temperature (Function 0x03, Register 0x0000)

**Command Format:** `[Address] 03 00 00 00 01 [CRC16]`

**Example - Address 01, Temperature 31°C:**

```
Master TX: 01 03 00 00 00 01 84 0A
Slave RX:  01 03 02 00 47 F8 76
```

Calculation: 0x0047 = 71 decimal → 71 - 40 = 31°C

---

### 2. Read Fan Status (Function 0x03, Register 0x0001)

**Command Format:** `[Address] 03 00 01 00 01 [CRC16]`

**Example - Address 01, Fan1 running, Fan2-4 stopped:**

```
Master TX: 01 03 00 01 00 01 D5 CA
Slave RX:  01 03 02 00 01 79 84
```

Calculation: 0x0001 = binary 0000 0001 → Fan1=1 (running), others=0 (stopped)

---

### 3. Set Start & Full Speed Temperature (Function 0x06, Register 0x0004)

**Command Format:** `[Address] 06 00 04 [L+40] [H+40] [CRC16]`

**Example - Address 01, Start=30°C, Full=50°C:**

```
L = 30 + 40 = 70 (0x46)
H = 50 + 40 = 90 (0x5A)

Master TX: 01 06 00 04 46 5A [CRC]
Slave RX:  01 06 00 04 46 5A [CRC]
```

**Alternative:** Use registers 0x000C and 0x000D to set individually

---

### 4. Set Fan Quantity (Function 0x06, Register 0x0006)

**Command Format:** `[Address] 06 00 06 00 [Qty] [CRC16]`

**Example - Address 01, 4 fans:**

```
Master TX: 01 06 00 06 00 04 [CRC]
Slave RX:  01 06 00 06 00 04 [CRC]
```

---

### 5. Read Fan Fault Code (Function 0x03, Register 0x000E)

**Command Format:** `[Address] 03 00 0E 00 01 [CRC16]`

**Example - Address 01, Fan3 fault, others normal:**

```
Master TX: 01 03 00 0E 00 01 E5 C9
Slave RX:  01 03 02 00 FB F9 C7
```

Calculation: 0xFB = binary 1111 1011 → Fan3=0 (fault), others=1 (normal)

---

### 6. Manual Speed Control

#### 6.1 Set Manual Speed to 50% (Function 0x06, Register 0x0003)

**Command Format:** `[Address] 06 00 03 00 [Speed%] [CRC16]`

**Example - Address 01, 50% speed:**

```
50% = 0x32 decimal

Master TX: 01 06 00 03 00 32 [CRC]
Slave RX:  01 06 00 03 00 32 [CRC]
```

#### 6.2 Exit Manual Mode (Restore Temperature Control)

**Command Format:** `[Address] 06 00 03 FF FF [CRC16]`

```
Master TX: 01 06 00 03 FF FF [CRC]
Slave RX:  01 06 00 03 FF FF [CRC]
```

---

### 7. Read Fan2 Speed (Function 0x03, Register 0x0008)

**Command Format:** `[Address] 03 00 08 00 01 [CRC16]`

**Example:**

```
Master TX: 01 03 00 08 00 01 [CRC]
Slave RX:  01 03 02 [RPM_High] [RPM_Low] [CRC]
```

Result: Convert to decimal for RPM value

---

### 8. Read All Parameters (Function 0x03, Registers 0x0000-0x000E)

**Command Format:** `01 03 00 00 00 0F 05 CE`

**Example Response:**

```
Slave RX: 01 03 1E 00 32 00 01 00 01 00 14 46 5A 00 01 00 04
          02 76 00 00 00 00 00 00 00 05 00 46 00 5A 00 FF ED 1E
```

**Response breakdown:**

- `1E` = 30 bytes (15 registers × 2 bytes each)
- `00 32` = Register 0x0000 value
- `00 01` = Register 0x0001 value
- (continues for all 15 registers)

---
