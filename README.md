# mock-openai

- [mock-openai](#mock-openai)
  - [Description](#description)
  - [Installation](#installation)
    - [Cargo](#cargo)
  - [Usage](#usage)
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

The server will start on `localhost:8079` by default. You can change the port by setting the `PORT` environment variable.

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

```bash
oha -z 10s -c 2000 -q 2000  --latency-correction --disable-keepalive http://localhost:8079/v1/chat/completions -T application/json -d '{"stream":false}' -m POST
```

## Configuration

The server can be configured with the following environment variables:

`ACTIX_WORKERS=1`: Number of workers to use. Default is 1.  
`PORT=8079`: Port to listen on. Default is 8079.  
`ADDRESS=0.0.0.0`: Address to listen on. Default is 0.0.0.0  
`ACTIX_MAX_CONN_RATE=256`: Maximum connection rate. Default is 256 (Still testing this).  
`ACTIX_CLIENT_REQUEST_TIMEOUT=600`: Request timeout for the client. Default is 600 seconds.  

LOG    
`RUST_LOG=info`: Set the log level. Default is warning. 

## TODO

- Optimise it further
- Cleanup the structs

## Contributing

Please feel free to raise any issues or PRs.  
However as this is just a toy project, I may not have the time to maintain it.  
