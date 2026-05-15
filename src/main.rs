mod connection;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    // create connection
    let mut connection = connection::SerialConnection::new("/dev/ttyUSB0", 115200)?;
    
    connection.extract_config()?;
    connection.usv_calibration()?;
    let response = connection.get_version()?;
    println!("Version: {}", response.trim());

    loop
    {
        connection.drain()?;
        thread::sleep(Duration::from_secs(1));
        let cpm = connection.get_cpm()?;
        println!("CPM: {}", cpm);
        let nSv = connection.get_nSv()?;
        println!("Radiation: {} nSv/h", nSv);
        let temp = connection.get_temperature()?;
        println!("Temperature: {}°C", temp);
        thread::sleep(Duration::from_secs(60));
    }
    
    //let gyro = connection.get_gyro()?;
    //println!("Gyro: {}", gyro);

    //let voltage = connection.get_voltage()?;
    //println!("Voltage: {}V", voltage);

    connection.close()?;
    Ok(())
}