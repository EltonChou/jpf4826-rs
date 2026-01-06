# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`jpf4826-rs` is a cross-platform command-line tool for controlling JPF4826 fan controllers via Modbus-RTU protocol over RS485. The JPF4826 is an industrial-grade 4-channel PWM DC fan temperature controller with automatic temperature-based speed control and comprehensive fault detection.

## Context

- **Language**: Latest Stable `Rust`
- **Platform**: `Linux`, `macOS`, `Windows`
- **Strategy**: Test driven development (TDD)
- **Target**: Standalone driver + Command line tool

### Standalone Driver

- **IMPORTANT**: Use defined type or enum instead of raw value as arguments.
- It should exposes register address and function code as enum.
- It should exposes base general read, write methods with raw output which can be easily invoked by other method.
- It should exposes methods below:
  - `status`: Status of controller. Please refer to status output in @README.md
  - `reset`: Reset controller.
  - `set_mode`: Set mode of controller, `Temperature` or `Manual`.
  - `set_eco`: Set work mode (Fan shutdown mode = eco mode).
  - `fan_speed`: Get fan speed by given index (1-4).
  - `set_fan_speed`: Manual set speed of fans.
  - `fan_count`: Get quantity of fans.
  - `fan_status`: Output all fans' status. Includes `index`, `fault`, `speed`.
  - `disable_fault_detection`: Disable fault detection.
  - `set_fan_count`: Set quantity of fans.
  - `temperature`: Get current temperature.
  - `set_addr`: Set modbus address.
  - `set_pwm_frequency`: Set pwm frequency by given value.
  - `set_temperature_threshold`: Change start/full temperature by given range.
  - `read`: A general read method which is invoked by other methods or by user directly.
  - `write`: A general write method which is invoked by other methods or by user directly.

### Command line tool

Binary name: `jf4826ctl`
A command line wrapper to control the controller with standalone driver.

## Reference Documentation

- Command line interface is described in @jpf4826ctl/README.md
- Modbus protocol details are described in @jpf4826_driver/jpf4826_modbus.md