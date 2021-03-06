use std::io;
use std::io::{BufReader, BufRead, Write, BufWriter};
use std::fs::File;
use reqwest;
use std::env;
use ldap3::{LdapConn, Scope, SearchEntry};
use std::{thread};
use std::time::Duration;
use std::process::Command;

//TODO: alot
fn main() -> std::io::Result<()> {

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "ibutton" {
        let f = File::open("/dev/ttyACM0")?;
        let mut reader = BufReader::new(f);
        loop {
            ibutton_loop(&mut reader);
        }
    } else {
        loop {
            uid_loop();
        }
    }

    Ok(())

}

fn process_ibutton(id: &str) -> Result<String, &str> {
    let ldap = LdapConn::new("ldaps://stone.csh.rit.edu").expect("issue connecting to given URL");

    let bind_dn = env::var("LDAP_BIND_DN")
        .expect("An LDAP Bind DN could not be found! Make sure it's set as the environment variable 'LDAP_BIND_DN'");
    let bind_pw = env::var("LDAP_BIND_PW")
        .expect("An LDAP Bind password could not be found! Make sure it's set as the environment variable 'LDAP_BIND_PW'");

    ldap.simple_bind(&bind_dn, &bind_pw).expect("issue binding with user + pw");

    let (rs, _res) = ldap.search(
        "cn=users,cn=accounts,dc=csh,dc=rit,dc=edu",
        Scope::Subtree,
        &("ibutton=".to_owned() + id),
        vec!["uid"]
    ).expect("Issue with the search itself").success().expect("Issue with the contents of the search");

    if rs.len() > 1 {
        return Err("too many possible responses");
    }

    for entry in rs {
        for other_entry in SearchEntry::construct(entry).attrs.get("uid").expect("uid not found") {
            return Ok(other_entry.to_string());
        }
    }
    Err("ofug")
}

fn ibutton_loop(reader: &mut BufReader<File>) {

    let mut user = String::new();
    print!("Waiting for valid ibutton input! > ");
    io::stdout().flush().unwrap();
    reader.read_line(&mut user).expect("Error reading the line!");
    user.pop();
    //let dummy = user.split_off(2);
    //user = user + "000" + &dummy + "01";
    print!("{}\n", user);
    user = process_ibutton(&("*".to_string() + &user + "01")).unwrap();
    play_from_user(user);
    
    Command::new("bash")
        .arg("-c")
        .args(&["cat", "1", ">", "/dev/ACM0"])
        .output()
        .expect("Error writing to Arduino");
    
}

fn uid_loop() {

    let mut user = String::new();
    print!("Enter a valid username > ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut user)
        .expect("Error reading the line");
    user.pop();
    play_from_user(user);

    println!("uid_loop() is ending!");
}

fn play_from_user(name: String) {

    let auth_key = env::var("HAROLDAUTH")
        .expect("An auth key could not be found! Make sure it's set as the environment variable 'HAROLDAUTH'");

   let url = "https://audiophiler.csh.rit.edu/get_harold/".to_owned() + &name;        

   println!("The route being requested is {}", url);
   let client = reqwest::Client::new();
   let link = client.post(&url[..])
       .body(format!("{{\"auth_key\":\"{}\"}}", auth_key))
       .header("Content-Type", "application/json")
       .send().expect("Did not post right")
       .text().expect("link text was not valid");
   println!("the link given was {}", link);


   let full_play = "vlc -I dummy --play-and-exit --stop-time 30 '".to_owned() + &link + "'";

   println!("{}", full_play);

    Command::new("bash")
        .arg("-c")
        .arg(full_play)
        .output()
        .expect("failed to execute process");


   //thread::sleep(Duration::new(31, 0));

   //thread::spawn(|| {
   //    Command::new("bash")
   //         .arg("-c")
   //         .arg("killall 'vlc'")
   //         .output()
   //         .expect("failed to execute process");
   //});
                                                                                      
   //let ar = AccReader::new(reqwest::get(&link[..]).unwrap());
   //let source_opt = rodio::Decoder::new(ar);

   //if source_opt.is_err() {
   //    println!("Format wasn't supported!");
   //    sink.append(rodio::source::SineWave::new(440));
   //    thread::sleep(Duration::new(1, 0));
   //    return;
   //}

   //let source = source_opt.unwrap();

   //let mut duration = match source.total_duration() {
   //    Some(x) => x,
   //    None => Duration::new(30, 0)
   //};

   //if source.total_duration().is_none() {
   //    println!("Duration not given, blocking for 30s");
   //} else {
   //    let as_secs = duration.as_secs();
   //    println!("Duration was {}", as_secs);
   //    if as_secs > 30 {
   //        duration = Duration::new(30, 0);
   //        println!("Duration was over 30 seconds, capping at 30");
   //    }
   //}

   //let message = match source.current_frame_len() {
   //    Some(x) => format!("frame len: {}", x),
   //    None => "frame length not given".to_string()
   //};
   //println!("{}", message);

   //println!("sample rate is {}", source.sample_rate());

   //sink.append(source);
   //thread::sleep(duration);
}
