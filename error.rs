// src/error.rs

use std::fmt;

#[derive(Debug)]
pub enum SensorError<E> {
    I2c(E),
    #[allow(dead_code)] 
    InvalidData,
}

impl<E: fmt::Debug> fmt::Display for SensorError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorError::I2c(e) => write!(f, "Помилка шини I2C: {:?}", e),
            SensorError::InvalidData => write!(f, "Отримано невалідні дані з сенсора"),
        }
    }
}

impl<E> From<E> for SensorError<E> {
    fn from(e: E) -> Self {
        SensorError::I2c(e)
    }
}