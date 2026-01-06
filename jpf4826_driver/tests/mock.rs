//! Mock Modbus context for testing without hardware.
//!
//! This module provides test utilities for simulating JPF4826 controller
//! responses without requiring actual serial hardware.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock Modbus register storage for testing.
///
/// Simulates a JPF4826 controller's register state in memory.
#[derive(Debug, Clone)]
pub struct MockController {
    pub registers: Arc<Mutex<HashMap<u16, u16>>>,
}

impl Default for MockController {
    fn default() -> Self {
        Self {
            registers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MockController {
    /// Creates a new mock controller with default values.
    pub fn new() -> Self {
        let controller = Self::default();
        controller.set_defaults();
        controller
    }

    /// Sets realistic default register values.
    fn set_defaults(&self) {
        let mut registers = self.registers.lock().unwrap();
        // Current temperature: 31°C (register value 71 = 31 + 40)
        registers.insert(0x0000, 71);

        // Fan status: All fans running (0b1111 = 0x000F)
        registers.insert(0x0001, 0x000F);

        // Modbus address: 1
        registers.insert(0x0002, 0x0001);

        // Mode: Temperature mode (0xFFFF)
        registers.insert(0x0003, 0xFFFF);

        // Combined temperature: Start 30°C (70), Full 50°C (90) = 0x465A
        registers.insert(0x0004, 0x465A);

        // Work mode: Minimum speed (1)
        registers.insert(0x0005, 0x0001);

        // Fan quantity: 4
        registers.insert(0x0006, 0x0004);

        // Fan speeds (RPM)
        registers.insert(0x0007, 1400); // Fan 1
        registers.insert(0x0008, 1400); // Fan 2
        registers.insert(0x0009, 1400); // Fan 3
        registers.insert(0x000A, 1400); // Fan 4

        // PWM frequency: 25kHz (0x0005)
        registers.insert(0x000B, 0x0005);

        // Start temperature: 30°C (70)
        registers.insert(0x000C, 70);

        // Full speed temperature: 50°C (90)
        registers.insert(0x000D, 90);

        // Fan fault code: All normal (0b1111 = 0x000F)
        registers.insert(0x000E, 0x000F);
    }

    /// Reads a single register.
    pub fn read_register(&self, addr: u16) -> Option<u16> {
        self.registers.lock().unwrap().get(&addr).copied()
    }

    /// Reads multiple consecutive registers.
    pub fn read_registers(&self, start_addr: u16, count: u16) -> Vec<u16> {
        (start_addr..start_addr + count)
            .map(|addr| self.read_register(addr).unwrap_or(0))
            .collect()
    }

    /// Writes a single register.
    pub fn write_register(&self, addr: u16, value: u16) {
        self.registers.lock().unwrap().insert(addr, value);
    }

    /// Sets fan fault for testing.
    ///
    /// # Arguments
    /// * `fan_index` - Fan number (1-4)
    /// * `has_fault` - true to set fault, false to clear
    pub fn set_fan_fault(&self, fan_index: u8, has_fault: bool) {
        if !(1..=4).contains(&fan_index) {
            return;
        }

        let current = self.read_register(0x000E).unwrap_or(0x000F);
        let bit_mask = 1u16 << (fan_index - 1);

        let new_value = if has_fault {
            current & !bit_mask // Clear bit = fault
        } else {
            current | bit_mask // Set bit = normal
        };

        self.write_register(0x000E, new_value);
    }

    /// Sets fan running status for testing.
    pub fn set_fan_running(&self, fan_index: u8, is_running: bool) {
        if !(1..=4).contains(&fan_index) {
            return;
        }

        let current = self.read_register(0x0001).unwrap_or(0x0000);
        let bit_mask = 1u16 << (fan_index - 1);

        let new_value = if is_running {
            current | bit_mask // Set bit = running
        } else {
            current & !bit_mask // Clear bit = stopped
        };

        self.write_register(0x0001, new_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_controller_defaults() {
        let controller = MockController::new();
        assert_eq!(controller.read_register(0x0000), Some(71)); // 31°C
        assert_eq!(controller.read_register(0x0001), Some(0x000F)); // All fans running
        assert_eq!(controller.read_register(0x0006), Some(4)); // 4 fans
    }

    #[test]
    fn test_read_multiple_registers() {
        let controller = MockController::new();
        let values = controller.read_registers(0x0000, 3);
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], 71); // Temperature
        assert_eq!(values[1], 0x000F); // Fan status
        assert_eq!(values[2], 0x0001); // Modbus addr
    }

    #[test]
    fn test_write_register() {
        let controller = MockController::new();
        controller.write_register(0x0002, 5);
        assert_eq!(controller.read_register(0x0002), Some(5));
    }

    #[test]
    fn test_set_fan_fault() {
        let controller = MockController::new();

        // Set Fan 3 fault
        controller.set_fan_fault(3, true);
        let fault_code = controller.read_register(0x000E).unwrap();
        assert_eq!(fault_code, 0x000B); // 0b1011 (bit 2 cleared)

        // Clear Fan 3 fault
        controller.set_fan_fault(3, false);
        let fault_code = controller.read_register(0x000E).unwrap();
        assert_eq!(fault_code, 0x000F); // 0b1111 (all normal)
    }

    #[test]
    fn test_set_fan_running() {
        let controller = MockController::new();

        // Stop Fan 2
        controller.set_fan_running(2, false);
        let status = controller.read_register(0x0001).unwrap();
        assert_eq!(status, 0x000D); // 0b1101 (bit 1 cleared)

        // Start Fan 2
        controller.set_fan_running(2, true);
        let status = controller.read_register(0x0001).unwrap();
        assert_eq!(status, 0x000F); // 0b1111 (all running)
    }
}
