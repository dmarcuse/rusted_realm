#!/usr/bin/env bash

mkdir deploy

if [[ "$TRAVIS_OS_NAME" == "windows" ]]; then
    cp target/release/rotmg_extractor.exe deploy/extractor_windows.exe
else
    cp target/release/rotmg_extractor deploy/extractor_$TRAVIS_OS_NAME
fi
