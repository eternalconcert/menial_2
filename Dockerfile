FROM alpine:latest

WORKDIR /app/

ADD target/release/main .
ADD default default/

RUN chmod +x main

CMD ["/main"]
