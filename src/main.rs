mod connection;
mod fit;
mod mqtt;
mod utilities;
use mysql::{prelude::Queryable, Opts, Pool, PooledConn};
use std::env;
use std::thread;
use std::time::Duration;

const SAMPLE_INTERVAL_SECS: u64 = 180;
const WINDOW_MINUTES: i32 = 15;
const SAMPLE_COUNT: usize = (WINDOW_MINUTES as usize * 60) / SAMPLE_INTERVAL_SECS as usize;
const DEFAULT_DB_HOST: &str = "db.r6.websupport.sk";

fn connect_database() -> Result<PooledConn, Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        let user = env::var("DB_USER").unwrap_or_else(|_| "girovedator".to_string());
        let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "!ZabezpeceneMySQL858".to_string());
        let database = env::var("DB_NAME").unwrap_or_else(|_| "Geiger_count".to_string());
        format!(
            "mysql://{}:{}@{}:3317/{}",
            user, password, DEFAULT_DB_HOST, database
        )
    });

    let opts = Opts::from_url(&database_url)?;
    let pool = Pool::new(opts)?;
    let database = pool.get_conn()?;
    Ok(database)
}

#[allow(unused)]
fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    let mut connection = connection::SerialConnection::new("/dev/ttyUSB0", 115200)?;
    let mut database = connect_database()?;

    database.query_drop(
        "CREATE TABLE IF NOT EXISTS radiation_aggregates (
            id BIGINT AUTO_INCREMENT PRIMARY KEY,
            collected_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            window_minutes INT NOT NULL,
            sample_count INT NOT NULL,
            average_nsv_h DOUBLE NOT NULL
        )",
    )?;
    
    connection.extract_config()?;
    connection.usv_calibration()?;
    let response = connection.get_version()?;
    println!("Version: {}", response.trim());

    loop
    {
        let mut samples = Vec::with_capacity(SAMPLE_COUNT);

        for sample_index in 0..SAMPLE_COUNT {
            connection.drain()?;
            connection.get_temperature()?;
            let radiation_nsv_h = connection.get_nSv(None)?;
            println!("Sample {}: {} nSv/h", sample_index + 1, radiation_nsv_h);
            samples.push(radiation_nsv_h as f64);

            if sample_index + 1 < SAMPLE_COUNT {
                thread::sleep(Duration::from_secs(SAMPLE_INTERVAL_SECS));
            }
        }

        let average_nsv_h = samples.iter().sum::<f64>() / samples.len() as f64;
        database.exec_drop(
            "INSERT INTO radiation_aggregates (window_minutes, sample_count, average_nsv_h) VALUES (?, ?, ?)",
            (WINDOW_MINUTES, samples.len() as i32, average_nsv_h),
        )?;

        println!(
            "Uploaded 15-minute average: {} nSv/h from {} samples",
            average_nsv_h,
            samples.len()
        );
    }

    connection.close()?;
    Ok(())
}