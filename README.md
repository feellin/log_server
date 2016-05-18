# log_server
a configable udp log collecting server written in Rust.

## Quick start

client protocol:
[ver:u8][cmd:u8][len:u32][data:String]
data must be sent by udp.

config file is ./conf/config.toml.
log files are to be configured to be "log_file_name" -> "cmd":

```
[log_files]
"test1.log" = 1
"test2.log" = 2
```

thoes incoming cmd data which is not configured will be dropped.

## License
MIT

