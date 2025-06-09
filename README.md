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

The server will start on `localhost:8079` by default. You can change the port by setting the `SERVER_PORT` environment variable.

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

The server can be configured with the following environment variables:

`SERVER_WORKERS`: Number of workers to use. Default is the number of available CPU cores.  
`SERVER_PORT=8079`: Port to listen on. Default is 8079.  
`SERVER_ADDRESS=0.0.0.0`: Address to listen on. Default is 0.0.0.0  
`SERVER_MAX_CONN_RATE=512`: Maximum connection rate. Default is 512.  
`SERVER_CLIENT_REQUEST_TIMEOUT=600`: Request timeout for the client. Default is 600 seconds.  

`RUST_LOG=info`: Set the log level. Default is warning. 

## TODO

- Optimise it further (How??)
- Cleanup the structs
- Considering downloading the sonnets within the code itself, wait for request

## Contributing

Please feel free to raise any issues or PRs.  
However as this is just a toy project, I may not have the time to maintain it.
