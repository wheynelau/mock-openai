# mock-openai

- [mock-openai](#mock-openai)
  - [Description](#description)
  - [Installation](#installation)
    - [Cargo](#cargo)
  - [Usage](#usage)
    - [Python](#python)
    - [Curl](#curl)

## Description

This is an OpenAI compatible mock server for testing purposes. Mostly meant for throughput testing.

## Installation

Binaries in progress...

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
print(response.choices[0].text)

# streaming
response = client.chat.completions.create(
    model="gpt-3.5-turbo",
    messages=[
        {'role': 'user', 'content': "What's 1+1? Answer in one word."}
    ],
    temperature=0,
    max_tokens=2,
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