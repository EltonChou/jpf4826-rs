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
Usage: jpf4826ctl [OPTION]... [COMMAND]
An utility to control JPF4826 through serial port by modbus protocol.

OPTION
  -p, --port=PORT      serial port (falls back to JPF4826_PORT env var)
  -a, --addr=ADDR      modbus address (falls back to JPF4826_ADDR env var)
  --help          display this help and exit
  --version       output version information and exit

COMMAND
  status          display controller status and exit
  set             set registors with flags
  reset           reset the controller
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

- `--temp_unit`: Temperature unit
  - `0`: Celsius
  - `1`: Fahrenheit

- `--help`: Display help for status command

#### Help Output

```
Usage: jpf4826ctl status [OPTION]...
Display current controller status.

OPTION
  --json          output in JSON format
  --temp_unit     temperature unit (0=Celsius, 1=Fahrenheit)
  --help          display this help and exit
```

#### Output

##### Normal

```
Mode                   Temperature
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
  "mode": "TEMPERATURE",
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
  --modbus_addr=5 \
  --low_temp=25 \
  --high_temp=38 \
  --eco=1 \
  --fan_qty=3 \
  --pwm_freq=5000
```

#### Options

- `--help`: Display help for set command

#### Help Output

```
Usage: jpf4826ctl set [OPTION]...
Set controller registers with specified values.

OPTION
  --mode          operating mode (0=Temperature, 1=Manual)
  --modbus_addr   modbus address (1-254)
  --low_temp      start temperature threshold (-20 to 120°C)
  --high_temp     full speed temperature threshold (-20 to 120°C)
  --eco           ECO/work mode (0=Shutdown, 1=Minimum speed)
  --fan_qty       number of fans (1-4, 0=disable fault detection)
  --pwm_freq      PWM frequency (500, 1000, 2000, 5000, 10000, 25000 Hz)
  --manual_speed  manual speed percentage (0-100, only for manual mode)
  --help          display this help and exit
```

#### Option Details

- `--mode`: Operating mode
  - `0`: Temperature mode (automatic speed control based on temperature)
  - `1`: Manual mode (requires `--manual_speed` to set fixed speed percentage)
  - Maps to register 0x0003 (write 0xFFFF for temperature mode, or 0-100 for manual mode)

- `--modbus_addr`: Modbus address of the controller
  - Range: `1-254`
  - Maps to register `0x0002`

- `--low_temp`: Start temperature threshold (°C)
  - Range: `-20` to `120`
  - Fan starts spinning at this temperature
  - Maps to register `0x000C` (stored with +40 offset)

- `--high_temp`: Full speed temperature threshold (°C)
  - Range: `-20` to `120`
  - Fan reaches 100% speed at this temperature
  - Must be greater than `--low_temp`
  - Maps to register `0x000D` (stored with +40 offset)

- `--eco`: ECO mode / Work mode
  - `0`: Shutdown mode (fan stops completely below low_temp - 3°C)
  - `1`: Minimum speed mode (fan maintains 20% speed below low_temp - 3°C)
  - Maps to register `0x0005`

- `--fan_qty`: Number of fans connected
  - Range: `1-4`
  - Set to `0` to disable fault detection
  - Maps to register `0x0006`

- `--pwm_freq`: PWM frequency in Hz
  - Valid values: `500`, `1000`, `2000`, `5000`, `10000`, `25000`
  - Default: `25000` Hz
  - Maps to register `0x000B`

- `--manual_speed`: Manual speed percentage (only valid when `--mode=1`)
  - Range: `0-100`
  - Disables temperature-based control
  - Maps to register `0x0003`

### `reset`

Reset the controller.

```shell
jpf4826ctl reset
```
