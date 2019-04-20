use std::io;
use std::io::{BufReader, BufRead, Write};
use std::fs::File;
use reqwest;
use std::env;
use ldap3::{LdapConn, Scope, SearchEntry};
use std::process::Command;

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

    //Ok(())

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
    Err("No user found!")
}

fn ibutton_loop(reader: &mut BufReader<File>) {

    let mut input = String::new();
    print!("Waiting for valid ibutton input! > ");
    io::stdout().flush().unwrap();
    reader.read_line(&mut input).expect("Error reading the line!");
    input.pop();
    //let dummy = user.split_off(2);
    //user = user + "000" + &dummy + "01";
    print!("{}\n", input);
    input = "*".to_string() + &input + "01";
    let user = process_ibutton(&input);
    if user.is_ok() {
        play_from_user(user.unwrap());
    }
    
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
}

