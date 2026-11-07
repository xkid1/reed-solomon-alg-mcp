FROM rust:1-slim AS build
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
# Required by the MCP Registry to verify ownership of OCI packages
LABEL io.modelcontextprotocol.server.name="io.github.xkid1/reed-solomon-alg-mcp"
COPY --from=build /app/target/release/reed-solomon-mcp /reed-solomon-mcp
ENTRYPOINT ["/reed-solomon-mcp"]
