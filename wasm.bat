@echo off
wasm-pack build --target web -d public/pkg --no-typescript --profiling
