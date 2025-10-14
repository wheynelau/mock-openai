# mock-openai

- [mock-openai](#mock-openai)
  - [Description](#description)
  - [Installation](#installation)
    - [Cargo](#cargo)
  - [Usage](#usage)
    - [Endpoints](#endpoints)
    - [Python](#python)
    - [Curl](#curl)
  - [Testing](#testing)
  - [Configuration](#configuration)
  - [TODO](#todo)
  - [Contributing](#contributing)

## Description

This is an OpenAI compatible mock server for experimentation and testing.  

## Installation

Check the releases tab. 

**Note**: You will need to download sonnets.txt into assets/sonnets.txt, relative to the binary.
The file is not included in the release. You can download the file [here](https://github.com/martin-gorner/tensorflow-rnn-shakespeare/blob/master/shakespeare/sonnets.txt)

```bash
# quick commands
mkdir -p assets
wget -O assets/sonnets.txt https://raw.githubusercontent.com/martin-gorner/tensorflow-rnn-shakespeare/master/shakespeare/sonnets.txt
```

### Cargo

```bash
git clone "https://github.com/wheynelau/mock-openai"
cd mock-openai
cargo build --release
```

## Usage

Start the server with the following command:
```bash
cargo run --release
# or if you have the binary
mock-openai
```

The server will start on `0.0.0.0:8079` by default. You can configure the server using command line arguments:

```bash
# Custom port and address
mock-openai --port 3000 --address 127.0.0.1

# Set worker count and timeout with human-readable duration
mock-openai --workers 4 --client-request-timeout 30m

# Download sonnets automatically and continue running
mock-openai --download-sonnets

# Show all available options
mock-openai --help

# Show version
mock-openai --version
```

### Endpoints

The server has the basic OpenAI compatible endpoints. They both route to the same handler internally  
and take in the same parameters.

- `POST /v1/chat/completions`: Chat completions endpoint.
- `POST /v1/completions`: Completions endpoint.
- `POST /echo`: Echo endpoint for testing.

- `GET /tokens`: Get the max tokens, this is the default `max_tokens` if you don't pass it in the request.
- `GET /hello`: Hello world endpoint.

### Python 

The code below is an example of how to use the mock server with the OpenAI Python client.

```python
import openai

client = openai.OpenAI(api_key="empty",
base_url='http://localhost:8079/v1/')

# non streaming
response = client.chat.completions.create(
    model="gpt-3.5-turbo",
    messages=[
        {'role': 'user', 'content': "What's 1+1? Answer in one word."}
    ],
    temperature=0,
    max_tokens=10,
)
print(response)

# streaming
response = client.chat.completions.create(
    model="gpt-3.5-turbo",
    messages=[
        {'role': 'user', 'content': "What's 1+1? Answer in one word."}
    ],
    temperature=0,
    max_tokens=10,
    stream=True,
    stream_options= {"include_usage": True}

)
for chunk in response:
    if chunk.usage:
        print(chunk.usage)
    else:
        print(chunk.choices[0].delta.content)
```

Note that the only important inputs are `max_tokens`, `stream` and `stream_options`.  
The rest of the parameters will pass through or are needed by OpenAI. The API key is needed due to  
the OpenAI client implementation. To demonstrate the pass through, look below at the curl example.

### Curl

```bash
curl "http://localhost:8079/v1/chat/completions" \
    -H "Content-Type: application/json" \
    -d '{}'
```

Streaming

```bash
curl -N http://localhost:8079/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "max_tokens": 5,
    "stream": true,
    "stream_options": { "include_usage": true }
  }'
```

## Testing

oha works well on the non streaming endpoint, but seems to error out on the streaming endpoint after adding the sleeps.

When the below tested on a t2.nano instance with 1 vcpu and 0.5GB of memory, the node was happy at about 50% CPU usage. But due to the  
network throttles, it occasionally falls to 10% CPU usage and the throughput decreases. Note that streaming does not work with oha.
For stable testing, its recommended to use an instance with fixed resources. 

```bash
oha -z 10m -c 512 -q 3000  --latency-correction --disable-keepalive \
  -T application/json -d '{"stream":false, "max_tokens": 2048}' -m POST \
  http://IP_ADDRESS/v1/chat/completions
```

## Configuration

The server can be configured with the following command line arguments:

```bash
Options:
  -w, --workers <WORKERS>              Number of worker threads to spawn [default: CPU count]
  --max-connection-rate <MAX_CONN_RATE> Maximum number of connections per second [default: 512]
  -p, --port <PORT>                    Port to listen on [default: 8079]
  -a, --address <ADDRESS>              Address to bind to [default: 0.0.0.0]
      --client-request-timeout <TIMEOUT> Client request timeout (e.g., "600s", "10m", "1h") [default: 600s]
      --download-sonnets              Download sonnets.txt
  -h, --help                          Print help
  -V, --version                       Print version
```

### Examples

```bash
# Use custom port and workers
mock-openai --port 8080 --workers 8

# Set timeout to 10 minutes
mock-openai --client-request-timeout 10m

# Use different address and connection rate
mock-openai --address 127.0.0.1 --max-connection-rate 1000

# Download sonnets and run with custom settings
mock-openai --download-sonnets --port 3000
```

The server also supports the `RUST_LOG` environment variable to set the log level. Default is warning:
```bash
RUST_LOG=info mock-openai
``` 

## TODO

- Optimise it further (How??)
- Cleanup the structs
- [DONE] Automatic sonnets download with --download-sonnets flag

## Contributing

Please feel free to raise any issues or PRs.  
However as this is just a toy project, I may not have the time to maintain it.
