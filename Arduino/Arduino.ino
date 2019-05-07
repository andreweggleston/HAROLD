#include <OneWire.h>


OneWire ds(12);
byte addr[8];
String keyStatus="";
bool from_rs;

OneWire rs(13);


void setup(void) {
  digitalWrite(11, HIGH);
  Serial.begin(9600);
}


void loop(void) {
  if (digitalRead(11) == LOW) {
    digitalWrite(11,HIGH);
  }
  getKeyCode();
  if(keyStatus=="ok"){
      byte i;
      for( i = 5; i>0; i--) {
           Serial.print(addr[i], HEX);           
      }
      Serial.print(from_rs ? 'R' : 'S');
      Serial.print('\n');
      delay(3000);
  }
  else if (keyStatus!="") { Serial.print(keyStatus);}

  
  delay(1000);
}


void getKeyCode(){
  byte present = 0;
  byte data[12];
  keyStatus="";

  from_rs = false;
  
  if (!ds.search(addr)) {
      ds.reset_search();
      if (!rs.search(addr)) {
          rs.reset_search();
          return;
      }
      from_rs = true;
  }
  
  digitalWrite(11, LOW);
  if ( OneWire::crc8( addr, 7) != addr[7]) {
      keyStatus="CRC invalid";
      return;
  }
  
  if ( addr[0] != 0x01) {
      keyStatus="not DS1990A";
      return;
  }
  keyStatus="ok";
  ds.reset();
  rs.reset();
}
