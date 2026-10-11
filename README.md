# MegaStore: recomendação de produtos baseada em grafos

Sistema de recomendação de produtos em Rust, feito para a disciplina Data Structure Strategy and Implementation. O catálogo e o histórico de compras são modelados como um grafo, e as recomendações saem de uma busca em largura com pontuação.

## Como funciona

**Grafo**
- Vértices: `Produto` (id, nome, preço) e `Cliente` (id, nome).
- Arestas: `Compra` (do cliente para o produto) e `Similaridade { pontuacao }` (entre dois produtos, nos dois sentidos). O peso da similaridade é o número de clientes que compraram os dois produtos.
- Armazenamento em lista de adjacência: `HashMap<id, Vec<Aresta>>`, mais um `HashMap<id, Vertice>`.

**Recomendação** (`src/recomendacao.rs`)
1. Busca em largura a partir do cliente, até uma profundidade máxima (padrão 2).
2. Cada aresta que chega a um produto soma o peso dele na pontuação do candidato.
3. Produtos já comprados não entram.
4. Ordena da maior para a menor pontuação (empate decidido pelo id) e devolve os melhores (padrão 5).

**Compra nova:** só as similaridades entre o produto novo e os que o cliente já tinha são reforçadas, sem recalcular o grafo inteiro.

## Como rodar

Requer o [Rust](https://www.rust-lang.org/tools/install) (cargo).

```
cargo run
```

O programa lê `data/produtos.csv` e `data/compras.csv` ao iniciar e mostra um menu:

| Opção | O que faz |
|---|---|
| 1 | Cadastrar produto |
| 2 | Listar produtos |
| 3 | Buscar produto por nome |
| 4 | Registrar compra de um cliente |
| 5 | Listar recomendações de todos os clientes |
| 0 | Sair |

Cadastros e compras feitos no menu são gravados no CSV.

### Formato dos dados

- `data/produtos.csv`: `nome,preco`
- `data/compras.csv`: `cliente,produto` (os clientes são criados a partir deste arquivo)

Nomes de produto não podem conter vírgula.

## Testes

```
cargo test
```

São 35 testes: 31 unitários (no fim de `src/grafo.rs`, `src/dados.rs` e `src/recomendacao.rs`) e 4 de integração (`tests/integration_test.rs`, que usa os CSVs de `tests/fixtures/`).

## Medição de desempenho

```
cargo run --release --bin benchmark
```

Mede o tempo de construção do grafo, de geração das similaridades, de recomendação e de compra nova, com 1 mil, 10 mil e 100 mil produtos (dados sintéticos). Leva cerca de 11 segundos. Resultados e metodologia em [`docs/resultados-benchmark.md`](docs/resultados-benchmark.md).
