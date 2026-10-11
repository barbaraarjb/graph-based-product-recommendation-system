use megastore::{gerar_similaridades, recomendar, registrar_compra, Grafo, IdVertice, TipoAresta};
use std::collections::HashSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

const VOLUMES: [usize; 3] = [1_000, 10_000, 100_000];
const COMPRAS_MIN: usize = 5;
const COMPRAS_MAX: usize = 15;
const RECOMENDACOES_MEDIDAS: usize = 200;
const COMPRAS_NOVAS_MEDIDAS: usize = 200;

struct Aleatorio(u64);

impl Aleatorio {
    fn proximo(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn fracao(&mut self) -> f64 {
        (self.proximo() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn escolher(&mut self, n: usize, concentrado: bool) -> usize {
        let f = self.fracao();
        let f = if concentrado { f.powi(3) } else { f };
        ((n as f64 * f) as usize).min(n - 1)
    }
}

struct Cenario {
    grafo: Grafo,
    clientes: Vec<IdVertice>,
    produtos: Vec<IdVertice>,
    tempo_construcao: Duration,
    tempo_similaridades: Duration,
}

fn construir(n_produtos: usize, concentrado: bool) -> Cenario {
    let mut rng = Aleatorio(0x9E37_79B9_7F4A_7C15);
    let mut grafo = Grafo::novo();

    let inicio = Instant::now();
    let produtos: Vec<IdVertice> = (0..n_produtos)
        .map(|i| grafo.cadastrar_produto(&format!("Produto {}", i), 10.0 + (i % 500) as f64).unwrap())
        .collect();
    let clientes: Vec<IdVertice> = (0..n_produtos)
        .map(|i| grafo.cadastrar_cliente(&format!("Cliente {}", i)).unwrap())
        .collect();
    for &cliente in &clientes {
        let quantas = COMPRAS_MIN + rng.escolher(COMPRAS_MAX - COMPRAS_MIN + 1, false);
        let mut escolhidos = HashSet::new();
        while escolhidos.len() < quantas {
            escolhidos.insert(produtos[rng.escolher(n_produtos, concentrado)]);
        }
        for produto in escolhidos {
            grafo.conectar(cliente, produto, TipoAresta::Compra).unwrap();
        }
    }
    let tempo_construcao = inicio.elapsed();

    let inicio = Instant::now();
    gerar_similaridades(&mut grafo);
    let tempo_similaridades = inicio.elapsed();

    Cenario { grafo, clientes, produtos, tempo_construcao, tempo_similaridades }
}

fn medir_recomendacao(c: &Cenario) -> Duration {
    let passo = (c.clientes.len() / RECOMENDACOES_MEDIDAS).max(1);
    let amostra: Vec<IdVertice> = c.clientes.iter().step_by(passo).take(RECOMENDACOES_MEDIDAS).copied().collect();
    let inicio = Instant::now();
    for &cliente in &amostra {
        black_box(recomendar(&c.grafo, cliente, 2, 5));
    }
    inicio.elapsed() / amostra.len() as u32
}

fn medir_compra_nova(c: &mut Cenario, concentrado: bool) -> Duration {
    let mut rng = Aleatorio(42);
    let mut total = Duration::ZERO;
    let mut feitas = 0;
    while feitas < COMPRAS_NOVAS_MEDIDAS {
        let cliente = c.clientes[rng.escolher(c.clientes.len(), false)];
        let produto = c.produtos[rng.escolher(c.produtos.len(), concentrado)];
        let inicio = Instant::now();
        let nova = registrar_compra(&mut c.grafo, cliente, produto).unwrap();
        let tempo = inicio.elapsed();
        if nova {
            total += tempo;
            feitas += 1;
        }
    }
    total / feitas as u32
}

fn formatar(d: Duration) -> String {
    if d.as_secs() >= 1 {
        format!("{:.2} s", d.as_secs_f64())
    } else if d.as_micros() >= 1000 {
        format!("{:.2} ms", d.as_secs_f64() * 1000.0)
    } else {
        format!("{} us", d.as_micros())
    }
}

fn main() {
    if cfg!(debug_assertions) {
        println!("Aviso: rode com --release, senao os tempos ficam distorcidos.\n");
    }
    for (rotulo, concentrado) in [("uniforme", false), ("concentrada (poucos produtos populares)", true)] {
        println!("=== Distribuicao de compras: {} ===", rotulo);
        println!(
            "{:>9} {:>9} {:>10} {:>10} | {:>11} {:>13} | {:>13} {:>14}",
            "produtos", "clientes", "vertices", "arestas", "construcao", "similaridades", "recomendar", "compra nova"
        );
        for &n in &VOLUMES {
            let mut c = construir(n, concentrado);
            let (vertices, arestas) = (c.grafo.total_vertices(), c.grafo.total_arestas());
            let recomendar_medio = medir_recomendacao(&c);
            let compra_media = medir_compra_nova(&mut c, concentrado);
            println!(
                "{:>9} {:>9} {:>10} {:>10} | {:>11} {:>13} | {:>13} {:>14}",
                n,
                c.clientes.len(),
                vertices,
                arestas,
                formatar(c.tempo_construcao),
                formatar(c.tempo_similaridades),
                formatar(recomendar_medio),
                formatar(compra_media)
            );
        }
        println!();
    }
}
