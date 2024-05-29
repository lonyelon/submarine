#include "Arduino.h"

#define OUT_PIN 2
#define SLEEP_TIME 1000

void setup() {
    pinMode(OUT_PIN, OUTPUT);
}

void loop() {
    digitalWrite(OUT_PIN, 1);
    delay(SLEEP_TIME/2);
    digitalWrite(OUT_PIN, 0);
    delay(SLEEP_TIME/2);
}
