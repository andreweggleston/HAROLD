use rodio::Sink;
use rodio::Source;
use std::io;
use std::io::Write;
use reqwest;
use acc_reader::AccReader;
use std::env;
use ldap3::{LdapConn, Scope, SearchEntry};
use std::{thread};
use std::time::Duration;

//TODO: alot
fn main() {


    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "ibutton" {
        loop {
            ibutton_loop();
        }
    } else {
        loop {
            uid_loop();
        }
    }

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

fn ibutton_loop() {

    let mut user = String::new();
    print!("Enter a valid ibutton ID > ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut user)
        .expect("Error reading the line");
    user.pop();
    user = process_ibutton(&user).unwrap();
    play_from_user(user);

}

fn uid_loop() {

    let mut user = String::new();
    print!("Enter a valid username > ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut user)
        .expect("Error reading the line");
    user.pop();
    play_from_user(user);

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
                                                                                      
   let device = rodio::default_output_device().expect("Audio device failed to find");
   let sink = Sink::new(&device);
                                                                                      
   let ar = AccReader::new(reqwest::get(&link[..]).unwrap());
   let source = rodio::Decoder::new(ar).unwrap();

   let mut duration = match source.total_duration() {
       Some(x) => x,
       None => Duration::new(30, 0)
   };

   if source.total_duration().is_none() {
       println!("Duration not given, blocking for 30s");
   } else {
       let as_secs = duration.as_secs();
       println!("Duration was {}", as_secs);
       if as_secs > 30 {
           duration = Duration::new(30, 0);
           println!("Duration was over 30 seconds, capping at 30");
       }
   }

   let message = match source.current_frame_len() {
       Some(x) => format!("frame len: {}", x),
       None => "frame length not given".to_string()
   };
   println!("{}", message);

   println!("sample rate is {}", source.sample_rate());

   sink.append(source);
   thread::sleep(duration);
}
