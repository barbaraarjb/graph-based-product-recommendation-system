# Resultados do benchmark

Gerados por `cargo run --release --bin benchmark` (dados sintéticos em memória, semente fixa).

## Ambiente

- CPU: AMD Ryzen 5 2600X (6 núcleos, 12 threads)
- Memória: 16 GB
- SO: Linux 7.0.0-34-generic
- Compilador: rustc 1.98.1, perfil `release`

## Parâmetros

- Volumes: 1.000, 10.000 e 100.000 produtos (o mesmo número de clientes)
- Cada cliente compra de 5 a 15 produtos distintos
- `recomendar`: média de 200 clientes, profundidade 2, limite 5
- `compra nova`: média de 200 compras novas por `registrar_compra`
- Distribuição uniforme: todos os produtos têm a mesma chance de compra
- Distribuição concentrada: poucos produtos concentram a maioria das compras

Os tempos variam um pouco entre execuções e entre máquinas; o que vale é a ordem de grandeza e a tendência.

## Antes: recomendação na ordem da BFS (para ao achar `limite` produtos)

```
=== Distribuicao de compras: uniforme ===
 produtos  clientes   vertices    arestas |  construcao similaridades |    recomendar    compra nova
     1000      1000       2000     106960 |     2.50 ms      12.81 ms |          2 us           2 us
    10000     10000      20000    1096035 |    17.09 ms     216.14 ms |          3 us           6 us
   100000    100000     200000   10995104 |   312.39 ms        3.58 s |          4 us           7 us

=== Distribuicao de compras: concentrada (poucos produtos populares) ===
 produtos  clientes   vertices    arestas |  construcao similaridades |    recomendar    compra nova
     1000      1000       2000      84041 |     1.44 ms       9.13 ms |          2 us           2 us
    10000     10000      20000     971319 |    14.22 ms     168.06 ms |          2 us           8 us
   100000    100000     200000   10457688 |   241.29 ms        3.55 s |          4 us          25 us

```

## Depois: recomendação ordenada pela soma dos pesos de similaridade

```
=== Distribuicao de compras: uniforme ===
 produtos  clientes   vertices    arestas |  construcao similaridades |    recomendar    compra nova
     1000      1000       2000     106960 |     1.91 ms      15.88 ms |        123 us           3 us
    10000     10000      20000    1096035 |    18.52 ms     205.91 ms |        158 us           6 us
   100000    100000     200000   10995104 |   332.53 ms        3.71 s |        269 us          13 us

=== Distribuicao de compras: concentrada (poucos produtos populares) ===
 produtos  clientes   vertices    arestas |  construcao similaridades |    recomendar    compra nova
     1000      1000       2000      84041 |     1.62 ms      10.06 ms |        227 us           3 us
    10000     10000      20000     971319 |    14.89 ms     207.07 ms |       1.25 ms           8 us
   100000    100000     200000   10457688 |   284.08 ms        4.16 s |       6.08 ms          25 us

```

## Leitura

- Construção e cálculo de similaridades crescem de forma aproximadamente linear com o volume.
- `recomendar` ficou de 50 a 1.500 vezes mais lento com o ranking, porque deixou de parar ao achar `limite` produtos e passou a somar as arestas de todos os produtos comprados pelo cliente.
- No cenário concentrado com 100.000 produtos, a recomendação passou de 4 µs para cerca de 6 ms: produtos populares têm milhares de arestas.
- O tempo de `recomendar` depende do grau dos produtos comprados pelo cliente, não do tamanho do catálogo.
