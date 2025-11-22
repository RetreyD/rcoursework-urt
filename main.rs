// src/main.rs

mod sensor;
mod error;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use clap::Parser;
use embedded_hal_mock::eh0::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
use sensor::{TemperatureSensor, Resolution};
use crate::error::SensorError;
use rand::Rng;
use env_logger::Builder; 
use log::LevelFilter;  

/// Консольна утиліта для моніторингу температури процесора через I2C сенсор (симуляція)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Інтервал зчитування даних в секундах
    #[arg(short, long, default_value_t = 2)]
    delay: u64,

    /// I2C адреса сенсора (в десятковій системі)
    #[arg(short, long, default_value_t = 72)]
    address: u8,
}

fn main() {
    let args = Args::parse();

    Builder::new()
        .filter_level(LevelFilter::Info) 
        .init();
    
    log::info!("Запуск моніторингу. Натисніть Ctrl+C для завершення.");
    log::info!("Інтервал: {} с, Адреса сенсора: 0x{:02X} ({})", args.delay, args.address, args.address);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Помилка встановлення обробника Ctrl-C");

    let mut min_temp = f32::MAX;
    let mut max_temp = f32::MIN;
    let mut temp_sum = 0.0;
    let mut successful_readings = 0;

    // --- Основний цикл моніторингу ---
    while running.load(Ordering::SeqCst) {
        // 1. Очікувана транзакція запису конфігурації (з TemperatureSensor::new)
        let config_transaction = I2cTransaction::write(args.address, vec![0x01, 0b10]); 

        // 2. Очікувана транзакція читання температури (з read_temperature)
        let mut rng = rand::thread_rng();
        let temp_c: f32 = rng.gen_range(35.0..55.0);
        let raw_val = (temp_c * 256.0) as i16;
        let bytes = raw_val.to_be_bytes();
        let read_transaction = I2cTransaction::write_read(args.address, vec![0x00], vec![bytes[0], bytes[1]]);
        
        let expectations = [config_transaction, read_transaction];
        let i2c_mock = I2cMock::new(&expectations);

        // Ініціалізація сенсора
        let mut sensor = match TemperatureSensor::new(i2c_mock, args.address, Resolution::High) {
            Ok(s) => s,
            Err(_) => {
                log::error!("Помилка симуляції під час ініціалізації.");
                break;
            }
        };

        match sensor.read_temperature() {
            Ok(temp) => {
                println!("Поточна температура: {:.2}°C", temp);
                min_temp = min_temp.min(temp);
                max_temp = max_temp.max(temp);
                temp_sum += temp;
                successful_readings += 1;
            }
            Err(SensorError::InvalidData) => log::error!("Помилка: отримано невалідні дані з сенсора."),
            Err(SensorError::I2c(_)) => {
                log::error!("Критична помилка шини I2C. Перевірте підключення.");
                break;
            }
        }

        let mut i2c_mock = sensor.release();
        i2c_mock.done();
        
        for _ in 0..args.delay {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
    }

    // Вивід фінального звіту
    println!("\n--- Підсумковий звіт ---");
    if successful_readings > 0 {
        println!("Всього успішних зчитувань: {}", successful_readings);
        println!("Мінімальна температура: {:.2}°C", min_temp);
        println!("Максимальна температура: {:.2}°C", max_temp);
        println!("Середня температура:    {:.2}°C", temp_sum / successful_readings as f32);
    } else {
        println!("Не було отримано жодного валідного значення температури.");
    }
    println!("--------------------------\n");
    println!("Натисніть клавішу Enter, щоб закрити програму...");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("Не вдалося прочитати рядок");
    log::info!("Програма завершила роботу.");
}