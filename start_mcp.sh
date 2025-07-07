source .env

cargo run -- --apollo-graph-ref $APOLLO_GRAPH_REF --apollo-key $APOLLO_KEY --http-address 127.0.0.1 --http-port 5002 --introspection