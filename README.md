# Rusted realm [![Build Status](https://travis-ci.com/dmarcuse/rusted_realm.svg?branch=master)](https://travis-ci.com/dmarcuse/rusted_realm)

Rusted realm is an experimental project to implement a standalone native client for Realm of the Mad God.
Right now, effort is concentrated on libraries tackling specific behavior needed for this, which will later be used by the client.


## Components

- rotmg_data - types representing miscellaneous client data, such as build parameters
- rotmg_packets - types representing ROTMG network packets
- rotmg_networking - implementation of ROTMG network protocol
- rotmg_extractor - utilities to extract data from the ROTMG client at runtime
- rusted_realm (not yet started) - the actual reverse-engineered game client
