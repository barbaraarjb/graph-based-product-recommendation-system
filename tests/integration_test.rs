use megastore::{carregar, recomendar, registrar_compra, Grafo, TipoAresta, Vertice};

#[test]
fn fluxo_completo_cadastro_e_recomendacao() {
    let mut grafo = Grafo::novo();

    let cliente = grafo.adicionar_vertice(|id| Vertice::Cliente {
        id,
        nome: "Carlos".to_string(),
    });
    let produto1 = grafo.adicionar_vertice(|id| Vertice::Produto {
        id,
        nome: "Notebook".to_string(),
        preco: 3500.0,
    });
    let produto2 = grafo.adicionar_vertice(|id| Vertice::Produto {
        id,
        nome: "Mouse".to_string(),
        preco: 89.90,
    });
    let produto3 = grafo.adicionar_vertice(|id| Vertice::Produto {
        id,
        nome: "Teclado".to_string(),
        preco: 199.90,
    });

    grafo.adicionar_aresta(cliente, produto1, TipoAresta::Compra);
    grafo.adicionar_aresta(produto1, produto2, TipoAresta::Similaridade { pontuacao: 0.7 });
    grafo.adicionar_aresta(produto1, produto3, TipoAresta::Similaridade { pontuacao: 0.65 });

    assert_eq!(grafo.total_vertices(), 4);
    assert_eq!(grafo.total_arestas(), 3);

    let encontrado = grafo.buscar_por_id(produto1).expect("produto deveria existir");
    assert_eq!(encontrado.nome(), "Notebook");

    let recomendacoes = recomendar(&grafo, cliente, 2, 10);
    assert!(recomendacoes.contains(&produto2));
    assert!(recomendacoes.contains(&produto3));

    let mut unicos = recomendacoes.clone();
    unicos.sort();
    unicos.dedup();
    assert_eq!(recomendacoes.len(), unicos.len());
}

#[test]
fn recomendacao_a_partir_de_produto_tambem_funciona() {
    let mut grafo = Grafo::novo();
    let produto1 = grafo.adicionar_vertice(|id| Vertice::Produto {
        id,
        nome: "Tenis".to_string(),
        preco: 200.0,
    });
    let produto2 = grafo.adicionar_vertice(|id| Vertice::Produto {
        id,
        nome: "Meia".to_string(),
        preco: 20.0,
    });

    grafo.adicionar_aresta(produto1, produto2, TipoAresta::Similaridade { pontuacao: 0.9 });

    let recomendacoes = recomendar(&grafo, produto1, 1, 10);
    assert_eq!(recomendacoes, vec![produto2]);
}

#[test]
fn recomenda_para_cliente_do_csv_de_exemplo() {
    let produtos = std::fs::read_to_string("tests/fixtures/produtos.csv").unwrap();
    let compras = std::fs::read_to_string("tests/fixtures/compras.csv").unwrap();
    let mut grafo = Grafo::novo();
    let resumo = carregar(&mut grafo, &produtos, &compras).unwrap();
    assert_eq!((resumo.produtos, resumo.clientes), (30, 15));

    let ana = grafo.listar_clientes().into_iter().find(|c| c.nome() == "Ana").unwrap().id();
    let recs = recomendar(&grafo, ana, 2, 5);

    assert!(!recs.is_empty());
    assert!(recs.len() <= 5);

    let nomes: Vec<&str> = recs.iter().map(|&id| grafo.buscar_por_id(id).unwrap().nome()).collect();
    assert!(recs.iter().all(|&id| grafo.buscar_produto_por_id(id).is_some()));
    let mut unicos = recs.clone();
    unicos.sort();
    unicos.dedup();
    assert_eq!(recs.len(), unicos.len());
    for comprado in ["Notebook", "Mouse", "Tenis Esportivo"] {
        assert!(!nomes.contains(&comprado), "{} já foi comprado pela Ana", comprado);
    }

    let todas = recomendar(&grafo, ana, 2, 100);
    let todas: Vec<&str> = todas.iter().map(|&id| grafo.buscar_por_id(id).unwrap().nome()).collect();
    assert!(todas.contains(&"Teclado Mecanico"));
}

#[test]
fn compra_nova_muda_as_recomendacoes_do_cliente() {
    let produtos = std::fs::read_to_string("tests/fixtures/produtos.csv").unwrap();
    let compras = std::fs::read_to_string("tests/fixtures/compras.csv").unwrap();
    let mut grafo = Grafo::novo();
    carregar(&mut grafo, &produtos, &compras).unwrap();

    let id = |g: &Grafo, nome: &str| g.buscar_por_nome(nome).into_iter().find(|v| v.nome() == nome).unwrap().id();
    let (ana, cafeteira, cafe) = (id(&grafo, "Ana"), id(&grafo, "Cafeteira"), id(&grafo, "Cafe em Graos"));

    assert!(!recomendar(&grafo, ana, 2, 100).contains(&cafe));

    assert_eq!(registrar_compra(&mut grafo, ana, cafeteira), Ok(true));
    let recs = recomendar(&grafo, ana, 2, 100);
    assert!(recs.contains(&cafe));
    assert!(!recs.contains(&cafeteira));
}
