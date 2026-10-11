use crate::grafo::{Grafo, IdVertice, TipoAresta, Vertice};
use std::collections::{HashMap, HashSet, VecDeque};

fn peso(tipo: &TipoAresta) -> f64 {
    match tipo {
        TipoAresta::Compra => 1.0,
        TipoAresta::Similaridade { pontuacao } => *pontuacao,
    }
}

pub fn recomendar(grafo: &Grafo, origem: IdVertice, profundidade_max: u32, limite: usize) -> Vec<IdVertice> {
    let mut visitados: HashSet<IdVertice> = HashSet::new();
    let mut fila: VecDeque<(IdVertice, u32)> = VecDeque::new();
    let mut pontuacoes: HashMap<IdVertice, f64> = HashMap::new();

    let ja_comprados: HashSet<IdVertice> = grafo
        .vizinhos(origem)
        .iter()
        .filter(|a| matches!(a.tipo, TipoAresta::Compra))
        .map(|a| a.destino)
        .collect();

    visitados.insert(origem);
    fila.push_back((origem, 0));

    while let Some((atual, profundidade)) = fila.pop_front() {
        if profundidade >= profundidade_max {
            continue;
        }

        for aresta in grafo.vizinhos(atual) {
            let vizinho = aresta.destino;

            let eh_produto = matches!(grafo.buscar_por_id(vizinho), Some(Vertice::Produto { .. }));
            if eh_produto && vizinho != origem && !ja_comprados.contains(&vizinho) {
                *pontuacoes.entry(vizinho).or_insert(0.0) += peso(&aresta.tipo);
            }

            if visitados.insert(vizinho) {
                fila.push_back((vizinho, profundidade + 1));
            }
        }
    }

    let mut ranking: Vec<(IdVertice, f64)> = pontuacoes.into_iter().collect();
    ranking.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    ranking.into_iter().take(limite).map(|(id, _)| id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grafo::TipoAresta;

    fn grafo_com_cadeia() -> (Grafo, IdVertice) {
        let mut g = Grafo::novo();
        let cliente = g.adicionar_vertice(|id| Vertice::Cliente { id, nome: "Ana".to_string() });
        let p1 = g.adicionar_vertice(|id| Vertice::Produto { id, nome: "Tenis".to_string(), preco: 299.90 });
        let p2 = g.adicionar_vertice(|id| Vertice::Produto { id, nome: "Meia".to_string(), preco: 29.90 });
        let p3 = g.adicionar_vertice(|id| Vertice::Produto { id, nome: "Camiseta".to_string(), preco: 79.90 });

        g.adicionar_aresta(cliente, p1, TipoAresta::Compra);
        g.adicionar_aresta(p1, p2, TipoAresta::Similaridade { pontuacao: 0.9 });
        g.adicionar_aresta(p2, p3, TipoAresta::Similaridade { pontuacao: 0.6 });
        g.adicionar_aresta(p3, p1, TipoAresta::Similaridade { pontuacao: 0.4 });

        (g, cliente)
    }

    #[test]
    fn recomenda_produtos_ate_profundidade_maxima() {
        let (g, cliente) = grafo_com_cadeia();
        let recs = recomendar(&g, cliente, 2, 10);
        assert!(recs.contains(&2));
        assert!(!recs.contains(&3));

        let recs = recomendar(&g, cliente, 3, 10);
        assert!(recs.contains(&3));
    }

    #[test]
    fn nao_recomenda_o_que_o_cliente_ja_comprou() {
        let (g, cliente) = grafo_com_cadeia();
        let recs = recomendar(&g, cliente, 5, 10);
        assert!(!recs.contains(&1)); 
        assert_eq!(recs, vec![2, 3]);
    }

    #[test]
    fn cliente_so_com_compras_nao_recebe_recomendacao() {
        let mut g = Grafo::novo();
        let c = g.cadastrar_cliente("Ana").unwrap();
        let p = g.cadastrar_produto("Tenis", 100.0).unwrap();
        g.conectar(c, p, TipoAresta::Compra).unwrap();
        assert!(recomendar(&g, c, 3, 10).is_empty());
    }

    #[test]
    fn nao_recomenda_produtos_duplicados_mesmo_com_ciclo() {
        let (g, cliente) = grafo_com_cadeia();
        let recs = recomendar(&g, cliente, 5, 10);
        let mut unicos = recs.clone();
        unicos.sort();
        unicos.dedup();
        assert_eq!(recs.len(), unicos.len());
    }

    #[test]
    fn respeita_limite_de_recomendacoes() {
        let (g, cliente) = grafo_com_cadeia();
        let recs = recomendar(&g, cliente, 5, 1);
        assert_eq!(recs.len(), 1);
    }

    #[test]
    fn cliente_sem_conexoes_nao_recomenda_nada() {
        let mut g = Grafo::novo();
        let cliente_isolado = g.adicionar_vertice(|id| Vertice::Cliente { id, nome: "Bia".to_string() });
        let recs = recomendar(&g, cliente_isolado, 3, 10);
        assert!(recs.is_empty());
    }

    fn produto(g: &mut Grafo, nome: &str) -> IdVertice {
        g.cadastrar_produto(nome, 10.0).unwrap()
    }

    #[test]
    fn ordena_pelo_peso_da_similaridade() {
        let mut g = Grafo::novo();
        let c = g.cadastrar_cliente("Ana").unwrap();
        let notebook = produto(&mut g, "Notebook");
        let webcam = produto(&mut g, "Webcam");
        let teclado = produto(&mut g, "Teclado");
        let mouse = produto(&mut g, "Mouse");
        g.conectar(c, notebook, TipoAresta::Compra).unwrap();
        // webcam é inserida primeiro, mas tem o menor peso
        g.conectar(notebook, webcam, TipoAresta::Similaridade { pontuacao: 1.0 }).unwrap();
        g.conectar(notebook, teclado, TipoAresta::Similaridade { pontuacao: 5.0 }).unwrap();
        g.conectar(notebook, mouse, TipoAresta::Similaridade { pontuacao: 9.0 }).unwrap();

        assert_eq!(recomendar(&g, c, 2, 10), vec![mouse, teclado, webcam]);
        assert_eq!(recomendar(&g, c, 2, 1), vec![mouse]);
    }

    #[test]
    fn soma_os_pesos_de_varios_caminhos_ate_o_mesmo_produto() {
        let mut g = Grafo::novo();
        let c = g.cadastrar_cliente("Ana").unwrap();
        let notebook = produto(&mut g, "Notebook");
        let mouse = produto(&mut g, "Mouse");
        let teclado = produto(&mut g, "Teclado");
        let webcam = produto(&mut g, "Webcam");
        g.conectar(c, notebook, TipoAresta::Compra).unwrap();
        g.conectar(c, mouse, TipoAresta::Compra).unwrap();
        // webcam tem um caminho forte (6), teclado tem dois caminhos médios (4 + 4)
        g.conectar(notebook, webcam, TipoAresta::Similaridade { pontuacao: 6.0 }).unwrap();
        g.conectar(notebook, teclado, TipoAresta::Similaridade { pontuacao: 4.0 }).unwrap();
        g.conectar(mouse, teclado, TipoAresta::Similaridade { pontuacao: 4.0 }).unwrap();

        assert_eq!(recomendar(&g, c, 2, 10), vec![teclado, webcam]);
    }

    #[test]
    fn empate_de_pontuacao_desempata_pelo_id() {
        let mut g = Grafo::novo();
        let c = g.cadastrar_cliente("Ana").unwrap();
        let base = produto(&mut g, "Base");
        let a = produto(&mut g, "A");
        let b = produto(&mut g, "B");
        g.conectar(c, base, TipoAresta::Compra).unwrap();
        g.conectar(base, b, TipoAresta::Similaridade { pontuacao: 2.0 }).unwrap();
        g.conectar(base, a, TipoAresta::Similaridade { pontuacao: 2.0 }).unwrap();

        assert_eq!(recomendar(&g, c, 2, 10), vec![a, b]);
    }

    #[test]
    fn recomendando_a_partir_de_produto_nao_devolve_o_proprio_produto() {
        let mut g = Grafo::novo();
        let a = produto(&mut g, "A");
        let b = produto(&mut g, "B");
        g.conectar(a, b, TipoAresta::Similaridade { pontuacao: 1.0 }).unwrap();
        g.conectar(b, a, TipoAresta::Similaridade { pontuacao: 1.0 }).unwrap();

        assert_eq!(recomendar(&g, a, 3, 10), vec![b]);
    }
}
