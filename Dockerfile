FROM debian:bullseye-slim

WORKDIR /app/

ADD target/release/main .
ADD default default/

RUN chmod +x main

CMD ["/app/main", "-p", "80", "-h", "0.0.0.0"]
