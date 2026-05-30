//! Driver do Acelerômetro e Giroscópio MPU-6050 via I2C.

use crate::traits::{SensorData, SensorError, SensorReader};

const MPU6050_ADDR: u8 = 0x68;
const REG_PWR_MGMT_1: u8 = 0x6B;
const REG_ACCEL_XOUT_H: u8 = 0x3B;

/// Driver de leitura concreta para o sensor de movimento MPU-6050.
pub struct Mpu6050Driver {
    i2c_path: String,
    device_address: u8,
    is_initialized: bool,
}

impl Mpu6050Driver {
    /// Inicializa e configura uma nova instância associada a um barramento I2C físico do Linux.
    pub fn new(i2c_path: &str) -> Self {
        Self {
            i2c_path: i2c_path.to_string(),
            device_address: MPU6050_ADDR,
            is_initialized: false,
        }
    }

    /// Inicializa e tira o sensor do modo sleep gravando no registro PWR_MGMT_1.
    pub fn init_sensor(&mut self) -> Result<(), SensorError> {
        let mut i2c = linux_embedded_hal::I2cdev::new(&self.i2c_path)
            .map_err(|e| SensorError::BusError(e.to_string()))?;

        // Grava 0x00 no PWR_MGMT_1 para acordar o sensor
        i2c.write(self.device_address, &[REG_PWR_MGMT_1, 0x00])
            .map_err(|e| SensorError::SensorOffline(e.to_string()))?;

        self.is_initialized = true;
        Ok(())
    }
}

impl SensorReader for Mpu6050Driver {
    #[allow(clippy::similar_names)]
    async fn read(&mut self) -> Result<SensorData, SensorError> {
        if !self.is_initialized {
            self.init_sensor()?;
        }

        let mut i2c = linux_embedded_hal::I2cdev::new(&self.i2c_path)
            .map_err(|e| SensorError::BusError(e.to_string()))?;

        // Lê 14 bytes sequenciais do sensor a partir de ACCEL_XOUT_H
        // 6 bytes Accel (X, Y, Z) + 2 bytes Temp + 6 bytes Gyro (X, Y, Z)
        let mut data = [0u8; 14];
        i2c.write_read(self.device_address, &[REG_ACCEL_XOUT_H], &mut data)
            .map_err(|e| SensorError::BusError(e.to_string()))?;

        // Conversão raw -> valores físicos (escala default ±2g para Accel e ±250 deg/s para Gyro)
        let ax_raw = i16::from_be_bytes([data[0], data[1]]);
        let ay_raw = i16::from_be_bytes([data[2], data[3]]);
        let az_raw = i16::from_be_bytes([data[4], data[5]]);

        let gx_raw = i16::from_be_bytes([data[8], data[9]]);
        let gy_raw = i16::from_be_bytes([data[10], data[11]]);
        let gz_raw = i16::from_be_bytes([data[12], data[13]]);

        // Escalas físicas padrão
        let accel_scale = 16384.0; // LSB/g
        let gyro_scale = 131.0; // LSB/(deg/s)
        let g_to_mps2 = 9.80665; // 1g = 9.80665 m/s²

        Ok(SensorData::Imu {
            accel_x: (f32::from(ax_raw) / accel_scale) * g_to_mps2,
            accel_y: (f32::from(ay_raw) / accel_scale) * g_to_mps2,
            accel_z: (f32::from(az_raw) / accel_scale) * g_to_mps2,
            gyro_x: f32::from(gx_raw) / gyro_scale,
            gyro_y: f32::from(gy_raw) / gyro_scale,
            gyro_z: f32::from(gz_raw) / gyro_scale,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpu6050_conversions() {
        let ax_raw: i16 = 16384; // 1g
        let accel_scale = 16384.0;
        let g_to_mps2 = 9.80665;
        let result = (f32::from(ax_raw) / accel_scale) * g_to_mps2;
        assert!((result - 9.80665).abs() < 1e-5);
    }
}
