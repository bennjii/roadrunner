### `roadrunner`
*WebSocket Code Executor Engine*

### How it works

Connect to the following URL:
```python
wss://[host_url]/ws
```

Once connected, Simply provide a script to the engine in the following format:

```js
{
    "language": string (e.g. "python"),
    "source": string (e.g. "import time\nfor i in range(1000):\n    time.sleep(0.1)\n    print(i)"),
    "nonce": string (Identifying Value Here)
    "standard_input": string (e.g. "Hello!"),
    "commandline_arguments": string (e.g. "--help")
}
```

> *No activity will result in disconnection, as this is intended for immediate use, with the websocket nature allowing for instant messaging upon event.*


## Valid languages
| Language   | Provoked-Execution | Pre-Delivered Execution |
|------------|--------------------|-------------------------|
| C          | ✅                 | 🚧                       |
| C++        | ✅                 | 🚧                       |
| Rust       | ✅                 | 🚧                       |
| Go         | ❌                 | 🚧                       |
| Javascript | ✅                 | ⚠️                        |
| Python     | ✅                 | ⚠️                        |

 ⚠️ *Compilation Works Differently*  

#### Provoked-Execution
This is the standard form, where source-code is provided as-is. Used in cases such as code playgrounds or simple lightweight code execution tasks. 

#### Pre-Delivered Execution
A future feature in which code can be uploaded, compiled and stored. In such case, the program can be called or launched remotely when needed, and killed as such when not.

A neat idea which permits `roadrunner` to be more extensibly used as a barebones microservice manager.

## Other
> #### A note about safety.
> This engine is not "safe", in the sense that the source of the code is considered trusted. This allows for greater flexibility in the execution of code, such as running microservices where the execution time is not pre-limited to prevent overflowing tasks.
> 
> However, although dockerized, provides a linch-pin for the host system from the remote code, enabling any format of "Remote Code Execution" - so, use with caution if the host system is required for the performance of other tasks. Ideally, `roadrunner` is hosted on a lightweight lambda or otherwise where **no important data or access is permitted or stored**, as such this is not of significance.
