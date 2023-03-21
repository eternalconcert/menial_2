FROM debian:bullseye-slim

WORKDIR /app/

COPY target/release/main .
COPY default/pages default/pages
COPY menial.yml /etc/menial/menial.yml

COPY default/welcomepage/ /usr/share/menial/html/

RUN apt-get update && apt-get upgrade && apt-get install build-essential libssl-dev -y

RUN chmod +x main

CMD ["/app/main", "-f", "/etc/menial/menial.yml"]
