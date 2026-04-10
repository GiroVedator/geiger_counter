//mod device;
mod connection;

fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    // create connection
    let mut port = connection::Connection::new("/dev/ttyUSB0", 115200)?;
    println!("Connected to serial port!");

    // loop 
    // {  
        
        
    //     // pass the connection to the device
    //     // use the device to get the version
    //     // continuosly read CPM and temperature and GYRO
    //     // store the data into mongoDB
    // }

    

    
    // //let mut device = device::Device::new("Geiger Reader")?;
    // //device.initialize(port)?;
    port.write(b"<GETVER>>")?;
    let version = port.read(&mut [0;32])?;
    //let version = device.get_version()?;
    
    println!("Version: {}", version);
    port.close()
}