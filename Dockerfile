FROM ubuntu:22.04

WORKDIR /app/

RUN apt-get update && apt-get install -y libssl-dev

COPY target/release/main .
COPY default/pages default/pages
COPY menial.yml /etc/menial/menial.yml

COPY default/welcomepage/ /usr/share/menial/html/

RUN chmod +x main

CMD ["/app/main", "-f", "/etc/menial/menial.yml"]
