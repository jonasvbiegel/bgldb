FROM rust:1.92

WORKDIR usr/local/app/
COPY . .

EXPOSE 8000

CMD ["cargo", "run", "--release"]
