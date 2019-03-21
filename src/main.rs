use rodio::Sink;
use std::io;
use std::io::Write;
use reqwest;
use acc_reader::AccReader;
use std::env;

// TODO: alot
fn main() {
    let auth_key = env::var("HAROLDAUTH").unwrap();
    print!("Whose HAROLD do you want? > ");
    io::stdout().flush().unwrap();
    let mut user = String::new();
    io::stdin().read_line(&mut user)
        .expect("Error reading the line");
    user.pop();
    let url = "https://audiophiler.csh.rit.edu/get_harold/".to_owned() + &user;
    let client = reqwest::Client::new();
    let link = client.post(&url[..])
        .body(format!("{{\"auth_key\":\"{}\"}}", auth_key))
        .header("Content-Type", "application/json")
        .send().expect("Did not post right")
        .text().unwrap();
    println!("the link given was {}", link);

    let device = rodio::default_output_device().expect("Audio device failed to find");
    let sink = Sink::new(&device);

    let ar = AccReader::new(reqwest::get(&link[..]).unwrap());
    //let ar = AccReader::new(TcpStream::connect((&link[..], 443)).unwrap());

    let source = rodio::Decoder::new(ar).unwrap();


    sink.append(source);
    let mut words = String::new();
    print!("Hit enter at any time to cancel");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut words).expect("fuck");
}

