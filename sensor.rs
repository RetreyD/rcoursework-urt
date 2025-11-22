// src/sensor.rs

use embedded_hal::blocking::i2c::{Write, Read, WriteRead};
use crate::error::SensorError;
use log::{info, warn};

// Уявні регістри сенсора
const REG_TEMP: u8 = 0x00;       // Регістр для читання температури
const REG_CONFIG: u8 = 0x01;     // Регістр для налаштувань

// Налаштування сенсора (імітація)
#[allow(dead_code)]
pub enum Resolution {
    Low,    // 0b00
    Medium, // 0b01
    High,   // 0b10
}

pub struct TemperatureSensor<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> TemperatureSensor<I2C>
where
    I2C: Write<Error = E> + Read<Error = E> + WriteRead<Error = E>,
{
    // Функція для ініціалізації сенсора з певними налаштуваннями
    pub fn new(mut i2c: I2C, address: u8, resolution: Resolution) -> Result<Self, SensorError<E>> {
        info!("Ініціалізація сенсора за адресою 0x{:02X}", address);

        let config_byte = match resolution {
            Resolution::Low => 0b00,
            Resolution::Medium => 0b01,
            Resolution::High => 0b10,
        };
        
        i2c.write(address, &[REG_CONFIG, config_byte])?;
        info!("Конфігурацію (роздільна здатність) записано: 0b{:02b}", config_byte);

        Ok(TemperatureSensor { i2c, address })
    }

    // Функція для зчитування температури
    pub fn read_temperature(&mut self) -> Result<f32, SensorError<E>> {
        let mut buffer = [0u8; 2];
        
        self.i2c.write_read(self.address, &[REG_TEMP], &mut buffer)?;
        
        let raw_temp = i16::from_be_bytes(buffer); 

        const ERROR_VALUE: i16 = i16::MIN; 
        if raw_temp == ERROR_VALUE {
            warn!("Сенсор повернув значення помилки! [0x{:04X}]", raw_temp as u16);
            return Err(SensorError::InvalidData);
        }

        let temperature = raw_temp as f32 / 256.0;

        info!("Зчитано сирі дані: [0x{:02X}, 0x{:02X}], температура: {:.2}°C", buffer[0], buffer[1], temperature);
        Ok(temperature)
    }

    pub fn release(self) -> I2C {
        self.i2c
    }
}