@echo off
SET PROTOC=I:\workspace\protobuf\bin\protoc.exe
echo Using protoc at: %PROTOC%
cargo build
