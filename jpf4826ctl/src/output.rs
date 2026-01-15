//! Output formatting for status command.

// Rust guideline compliant 2026-01-06

use jpf4826_driver::{ControllerStatus, FanStatus, Temperature, TemperatureUnit};

/// Formats controller status as human-readable text.
///
/// Output format matches the specification in README.md.
pub fn format_status_text(status: &ControllerStatus) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("ECO Mode\t{}\n", status.eco_mode));
    output.push_str(&format!(
        "Modbus Address\t0x{:04X}\n",
        status.modbus_address
    ));
    output.push_str(&format!(
        "PWM Frequency\t{} Hz\n",
        status.pwm_frequency.to_hz()
    ));
    output.push_str(&format!("Fan Quantity\t{}\n", status.fan_count));

    // Temperature section
    output.push_str(&format!(
        "Temperature\t{}\n",
        format_temperature(&status.temperature_current)
    ));
    output.push_str(&format!(
        "\tLow Threshold\t{}\n",
        format_temperature(&status.temperature_low_threshold)
    ));
    output.push_str(&format!(
        "\tHigh Threshold\t{}\n",
        format_temperature(&status.temperature_high_threshold)
    ));

    // Fan status section
    output.push_str("\nFan Status\n");
    for fan in &status.fans {
        output.push_str(&format!("\t{}\n", fan.index));
        let status_str = match fan.status {
            FanStatus::Normal => "Normal",
            FanStatus::Fault => "Fault",
        };
        output.push_str(&format!("\t\tStatus\t{}\n", status_str));
        output.push_str(&format!("\t\tSpeed (RPM)\t{}\n", fan.rpm));
    }

    output
}

/// Formats a temperature value with unit symbol.
fn format_temperature(temp: &Temperature) -> String {
    let symbol = match temp.unit {
        TemperatureUnit::Celsius => "℃",
        TemperatureUnit::Fahrenheit => "℉",
    };
    format!("{} {}", temp.value, symbol)
}

/// Converts controller status to JSON string.
///
/// Output matches the JSON schema in schemas/jpf4826-status-response.schema.json.
pub fn format_status_json(status: &ControllerStatus) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(status)
}

/// Converts temperatures from Celsius to Fahrenheit in status.
pub fn convert_to_fahrenheit(mut status: ControllerStatus) -> ControllerStatus {
    status.temperature_current = celsius_to_fahrenheit_temp(status.temperature_current);
    status.temperature_low_threshold = celsius_to_fahrenheit_temp(status.temperature_low_threshold);
    status.temperature_high_threshold =
        celsius_to_fahrenheit_temp(status.temperature_high_threshold);

    status
}

/// Converts a single temperature from Celsius to Fahrenheit.
fn celsius_to_fahrenheit_temp(temp: Temperature) -> Temperature {
    if temp.unit == TemperatureUnit::Celsius {
        Temperature {
            value: (temp.value * 9 / 5) + 32,
            unit: TemperatureUnit::Fahrenheit,
        }
    } else {
        temp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jpf4826_driver::{FanInfo, FanStatus, PwmFrequency};

    fn create_test_status() -> ControllerStatus {
        ControllerStatus {
            eco_mode: true,
            modbus_address: 1,
            pwm_frequency: PwmFrequency::Hz25000,
            fan_count: 4,
            temperature_current: Temperature {
                value: 26,
                unit: TemperatureUnit::Celsius,
            },
            temperature_low_threshold: Temperature {
                value: 27,
                unit: TemperatureUnit::Celsius,
            },
            temperature_high_threshold: Temperature {
                value: 40,
                unit: TemperatureUnit::Celsius,
            },
            fans: vec![
                FanInfo {
                    index: 1,
                    status: FanStatus::Normal,
                    rpm: 1400,
                },
                FanInfo {
                    index: 2,
                    status: FanStatus::Fault,
                    rpm: 0,
                },
            ],
        }
    }

    #[test]
    fn test_format_text_contains_key_fields() {
        let status = create_test_status();
        let output = format_status_text(&status);

        assert!(output.contains("ECO Mode\ttrue"));
        assert!(output.contains("Modbus Address\t0x0001"));
        assert!(output.contains("PWM Frequency\t25000 Hz"));
        assert!(output.contains("Fan Quantity\t4"));
        assert!(output.contains("Temperature\t26 ℃"));
        assert!(output.contains("Status\tNormal"));
        assert!(output.contains("Status\tFault"));
    }

    #[test]
    fn test_format_json_is_valid() {
        let status = create_test_status();
        let json = format_status_json(&status).unwrap();

        // Parse back to verify it's valid JSON
        let _parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(json.contains("\"eco_mode\""));
    }

    #[test]
    fn test_fahrenheit_conversion() {
        let status = create_test_status();
        let converted = convert_to_fahrenheit(status);

        assert_eq!(converted.temperature_current.value, 78); // 26°C = 78.8°F ≈ 78
        assert_eq!(
            converted.temperature_current.unit,
            TemperatureUnit::Fahrenheit
        );
    }

    #[test]
    fn test_json_output_matches_schema() {
        // Create a realistic status with all 4 fans
        let status = ControllerStatus {
            eco_mode: true,
            modbus_address: 1,
            pwm_frequency: PwmFrequency::Hz25000,
            fan_count: 4,
            temperature_current: Temperature {
                value: 26,
                unit: TemperatureUnit::Celsius,
            },
            temperature_low_threshold: Temperature {
                value: 27,
                unit: TemperatureUnit::Celsius,
            },
            temperature_high_threshold: Temperature {
                value: 40,
                unit: TemperatureUnit::Celsius,
            },
            fans: vec![
                FanInfo {
                    index: 1,
                    status: FanStatus::Normal,
                    rpm: 1400,
                },
                FanInfo {
                    index: 2,
                    status: FanStatus::Fault,
                    rpm: 0,
                },
                FanInfo {
                    index: 3,
                    status: FanStatus::Normal,
                    rpm: 1400,
                },
                FanInfo {
                    index: 4,
                    status: FanStatus::Normal,
                    rpm: 1400,
                },
            ],
        };

        // Format as JSON
        let json_str = format_status_json(&status).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Load schema from file
        let schema_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("schemas")
            .join("jpf4826-status-response.schema.json");

        let schema_str = std::fs::read_to_string(&schema_path).expect("Failed to read schema file");
        let schema_json: serde_json::Value =
            serde_json::from_str(&schema_str).expect("Failed to parse schema JSON");

        // Compile and validate
        let compiled_schema =
            jsonschema::validator_for(&schema_json).expect("Failed to compile schema");

        // Validate returns Result<(), ValidationError>
        if let Err(validation_error) = compiled_schema.validate(&json_value) {
            panic!("JSON output does not match schema:\n{}", validation_error);
        }
    }
}
