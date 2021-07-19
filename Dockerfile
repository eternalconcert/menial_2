FROM debian:bullseye-slim

WORKDIR /app/

COPY target/release/main .
COPY default default/

RUN chmod +x main

CMD ["/app/main", "-p", "80", "-h", "0.0.0.0"]
