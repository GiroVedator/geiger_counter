//mod device;
mod connection;

fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    let mut port = connection::Connection::new("/dev/ttyUSB0", 115200)?;

    println!("Connected to serial port!");
    //let mut device = device::Device::new("Geiger Reader")?;
    //device.initialize(port)?;
    port.write(b"<GETVER>>")?;
    let version = port.read(&mut [0;16])?;
    //let version = device.get_version()?;
    
    println!("Version: {}", version);

    Ok(())
}