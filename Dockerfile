FROM debian:bullseye-slim

WORKDIR /app/

COPY target/release/main .
COPY default/pages default/pages
COPY menial.yml menial.yml

COPY default/welcomepage/ /usr/share/menial/html/

RUN chmod +x main

CMD ["/app/main", "-f", "menial.yml"]
