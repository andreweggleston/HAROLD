use rodio::Sink;
use std::io;
use std::io::Write;
use reqwest;
use acc_reader::AccReader;
use std::env;
use ldap3::{LdapConn, Scope, SearchEntry};

//TODO: alot
fn main() {

    //check for auth key
    let auth_key = env::var("HAROLDAUTH")
        .expect("An auth key could not be found! Make sure it's set as the environment variable 'HAROLDAUTH'");

    let args: Vec<String> = env::args().collect();

    let mut user = String::new();

    if args.len() > 1 && args[1] == "ibutton" {

        print!("Enter a valid ibutton ID > ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut user)
            .expect("Error reading the line");
        user.pop();
        user = process_ibutton(&user).unwrap();
    } else {
        print!("Whose HAROLD do you want? > ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut user)
            .expect("Error reading the line");
        user.pop();
    }

    let url = "https://audiophiler.csh.rit.edu/get_harold/".to_owned() + &user;
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
    //let ar = AccReader::new(TcpStream::connect((&link[..], 443)).unwrap());

    let source = rodio::Decoder::new(ar).unwrap();


    sink.append(source);
    sink.sleep_until_end();
    //let mut words = String::new();
    //print!("Hit enter at any time to cancel");
    //io::stdout().flush().unwrap();
    //io::stdin().read_line(&mut words).expect("fuck");
}

fn process_ibutton(id: &str) -> Result<String, &str> {
    let ldap = LdapConn::new("ldaps://stone.csh.rit.edu").expect("issue connecting to given URL");

    let bind_dn = env::var("LDAP_BIND_DN")
        .expect("An LDAP Bind DN could not be found! Make sure it's set as the environment variable 'LDAP_BIND_DN'");
    let bind_pw = env::var("LDAP_BIND_PW")
        .expect("An LDAP Bind password could not be found! Make sure it's set as the environment variable 'LDAP_BIND_PW'");

    ldap.simple_bind(&bind_dn, &bind_pw).expect("issue binding with user + pw");

    println!("valid bind!");

    let (rs, _res) = ldap.search(
        "cn=users,cn=accounts,dc=csh,dc=rit,dc=edu",
        Scope::Subtree,
        &("ibutton=".to_owned() + id),
        vec!["uid"]
    ).expect("Issue with the search itself").success().expect("Issue with the contents of the search");

    if rs.len() > 1 {
        return Err("too many possible responses");
    }

    //let mut valid = SearchEntry::construct(rs[0]);

    for entry in rs {
        for other_entry in SearchEntry::construct(entry).attrs.get("uid").expect("uid not found") {
            return Ok(other_entry.to_string());
        }
    }
    Err("ofug")
}
