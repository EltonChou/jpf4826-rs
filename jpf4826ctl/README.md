# jpf4826-rs

This project is a cross-platform command-line tool for JPF4826 fan controller by modbus protocol.

binary name is `jpf4826ctl`

## Global Options

### `-p`, `--port=PORT`

Specify `PORT` to connect to the JPF4826 controller.

**Required**: Yes (unless `JPF4826_PORT` environment variable is set)

**Examples**:
- Linux: `/dev/ttyUSB0`
- macOS: `/dev/tty.usbserial-XXXXXXXX`
- Windows: `COM3`

If not provided, falls back to the `JPF4826_PORT` environment variable.

### `-a`, `--addr=ADDR`

Specify the Modbus address of the controller.

**Range**: `1-254` (hexadecimal `0x0001` to `0x00FE`)

**Required**: Yes (unless `JPF4826_ADDR` environment variable is set)

If not provided, falls back to the `JPF4826_ADDR` environment variable.

### `--help`

Display available commands and usage information.

#### Output

```
Control JPF4826 fan controller via Modbus-RTU

Usage: jpf4826ctl [OPTIONS] <COMMAND>

Commands:
  status  Display controller status
  set     Set controller registers
  reset   Reset the controller
  help    Print this message or the help of the given subcommand(s)

Options:
  -p, --port <PORT>  Serial port (falls back to JPF4826_PORT env var) [env: JPF4826_PORT=]
  -a, --addr <ADDR>  Modbus address (falls back to JPF4826_ADDR env var) [env: JPF4826_ADDR=]
  -v, --verbose      Enable verbose logging (debug output)
  -h, --help         Print help
  -V, --version      Print version
```

### `--version`

#### Output

Display version.

## Commands

### `status`

Show current status of controller.

```shell
jpf4826ctl status
```

#### Options

- `--json`: Output as JSON

- `--temp-unit`: Temperature unit
  - `0`: Celsius
  - `1`: Fahrenheit

- `--help`: Display help for status command

#### Help Output

```
Display controller status

Usage: jpf4826ctl status [OPTIONS]

Options:
      --json                   Output in JSON format
      --temp-unit <TEMP_UNIT>  Temperature unit (0=Celsius, 1=Fahrenheit)
  -h, --help                   Print help
```

#### Output

##### Normal

```
ECO Mode               True
Modbus Address         0x0001
PWM Frequency          25000 Hz
Fan Quantity           4
Temperature            26 ℃
    Low Threshold      27 ℃
    High Threshold     40 ℃

Fan Status
    1
        Status         Normal
        Speed (RPM)    1400
    2
        Status         Fault
        Speed (RPM)    0
    3
        Status         Normal
        Speed (RPM)    1400
    4
        Status         Normal
        Speed (RPM)    1400
```

##### JSON

**Schema**: [`schemas/jpf4826-status-response.schema.json`](schemas/jpf4826-status-response.schema.json)

```json
{
  "eco_mode": true,
  "modbus_address": 1,
  "pwm_frequency": {
    "value": 25000,
    "unit": "Hz"
  },
  "fan_count": 4,
  "temperature": {
    "current": {
      "value": 26,
      "unit": "CELSIUS"
    },
    "low_threshold": {
      "value": 27,
      "unit": "CELSIUS"
    },
    "high_threshold": {
      "value": 40,
      "unit": "CELSIUS"
    }
  },
  "fans": [
    {
      "index": 1,
      "status": "NORMAL",
      "rpm": 1400
    },
    {
      "index": 2,
      "status": "FAULT",
      "rpm": 0
    },
    {
      "index": 3,
      "status": "NORMAL",
      "rpm": 1400
    },
    {
      "index": 4,
      "status": "NORMAL",
      "rpm": 1400
    }
  ]
}
```

### `set`

Set registers of controller by arguments

```bash
jpf4826ctl set \
  --mode=0 \
  --modbus-addr=5 \
  --low-temp=25 \
  --high-temp=38 \
  --eco=1 \
  --fan-qty=3 \
  --pwm-freq=5000
```

#### Options

- `--help`: Display help for set command

#### Help Output

```
Set controller registers

Usage: jpf4826ctl set [OPTIONS]

Options:
      --mode <MODE>                  Operating mode (0=Temperature, 1=Manual)
      --modbus-addr <MODBUS_ADDR>    Modbus address (1-254)
      --low-temp <LOW_TEMP>          Start temperature threshold (-20 to 120°C)
      --high-temp <HIGH_TEMP>        Full speed temperature threshold (-20 to 120°C)
      --eco <ECO>                    ECO/work mode (0=Shutdown, 1=Minimum speed)
      --fan-qty <FAN_QTY>            Number of fans (1-4, 0=disable fault detection)
      --pwm-freq <PWM_FREQ>          PWM frequency (500, 1000, 2000, 5000, 10000, 25000 Hz)
      --manual-speed <MANUAL_SPEED>  Manual speed percentage (0-100, only for manual mode)
  -h, --help                         Print help
```

#### Option Details

- `--mode`: Operating mode
  - `0`: Temperature mode (automatic speed control based on temperature)
  - `1`: Manual mode (requires `--manual-speed` to set fixed speed percentage)
  - Maps to register 0x0003 (write 0xFFFF for temperature mode, or 0-100 for manual mode)

- `--modbus-addr`: Modbus address of the controller
  - Range: `1-254`
  - Maps to register `0x0002`

- `--low-temp`: Start temperature threshold (°C)
  - Range: `-20` to `120`
  - Fan starts spinning at this temperature
  - Maps to register `0x000C` (stored with +40 offset)

- `--high-temp`: Full speed temperature threshold (°C)
  - Range: `-20` to `120`
  - Fan reaches 100% speed at this temperature
  - Must be greater than `--low-temp`
  - Maps to register `0x000D` (stored with +40 offset)

- `--eco`: ECO mode / Work mode
  - `0`: Shutdown mode (fan stops completely below low_temp - 3°C)
  - `1`: Minimum speed mode (fan maintains 20% speed below low_temp - 3°C)
  - Maps to register `0x0005`

- `--fan-qty`: Number of fans connected
  - Range: `1-4`
  - Set to `0` to disable fault detection
  - Maps to register `0x0006`

- `--pwm-freq`: PWM frequency in Hz
  - Valid values: `500`, `1000`, `2000`, `5000`, `10000`, `25000`
  - Default: `25000` Hz
  - Maps to register `0x000B`

- `--manual-speed`: Manual speed percentage (only valid when `--mode=1`)
  - Range: `0-100`
  - Disables temperature-based control
  - Maps to register `0x0003`

### `reset`

Reset the controller.

```shell
jpf4826ctl reset
```
