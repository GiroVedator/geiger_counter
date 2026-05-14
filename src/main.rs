mod connection;

fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    // create connection
    let mut connection = connection::SerialConnection::new("/dev/ttyUSB0", 115200)?;

    connection.drain()?;
    connection.extract_config()?;
    connection.usv_calibration()?;
    let response = connection.get_version()?;
    println!("Version: {}", response.trim());

    let cpm = connection.get_cpm()?;
    println!("CPM: {}", cpm);

    let gyro = connection.get_gyro()?;
    println!("Gyro: {}", gyro);

    let voltage = connection.get_voltage()?;
    println!("Voltage: {}V", voltage);

    let nSv = connection.get_nSv()?;
    println!("Radiation: {} nSv/h", nSv);
    
    let temp = connection.get_temperature()?;
    println!("Temperature: {}°C", temp);

    connection.close()?;
    Ok(())
}