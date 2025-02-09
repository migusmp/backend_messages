# backend_messages/Dockerfile
FROM rust:latest

WORKDIR /usr/src/app

# Copiar archivos de configuración y dependencias
COPY backend_messages/Cargo.toml backend_messages/Cargo.lock ./

# Instalar dependencias
RUN cargo build --release

# Copiar todo el código fuente
COPY backend_messages/ .

# Volver a compilar después de copiar el código
RUN cargo build --release

# Exponer el puerto del servicio
EXPOSE 3001

# Comando para ejecutar el servicio
CMD ["cargo", "run", "--release"]

