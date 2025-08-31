sudo cargo flamegraph --profile profiling --bin hantei-cli --features "hantei-cli" -- \
--benchmark 100000 \
data/flow_siag.json \
data/qualities.json \
data/sample_data_2.json
