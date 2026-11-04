# Comandos del proyecto. `make` sin argumentos lista los disponibles.

CARGO ?= cargo
FILE ?=
ARGS ?=
TASKSET ?= taskset -c 0

.DEFAULT_GOAL := help
.PHONY: help install build run test bench

help: ## Muestra esta ayuda
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-8s\033[0m %s\n", $$1, $$2}'

install: ## Instala el binario `rabi` en el sistema
	$(CARGO) install --path .

build: ## Compila el intérprete en modo release
	$(CARGO) build --release

run: ## Ejecuta un programa sin instalar: make run FILE=programa.rabi [ARGS=--scanning]
	$(CARGO) run --release --quiet -- $(FILE) $(ARGS)

test: ## Corre los tests de integración sobre tests/test-programs/
	$(CARGO) test --test interpreter --test vm

bench: ## Corre el benchmark (rabi vs Python vs Rust) fijado al CPU 0 e imprime los tiempos
	$(TASKSET) $(CARGO) bench
