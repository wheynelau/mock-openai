num_concurrent=(100 200 500 1000 2000 5000 10000 20000)
open_files=1024 # adjust this, some defaults are only 1024

for concurrent in ${num_concurrent[@]}; do
    echo "Running with $concurrent concurrent connections"
    oha -z 10s -c $open_files -q $concurrent  \
        --latency-correction \
        --disable-keepalive http://localhost:8079/v1/chat/completions \
        -T application/json \
        -d '{"stream":false}' \
        -m POST \
        --no-tui \
        -j | tee oha_$concurrent.json
done
