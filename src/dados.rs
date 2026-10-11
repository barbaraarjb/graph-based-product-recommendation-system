use crate::grafo::{Grafo, IdVertice, TipoAresta, Vertice};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Resumo {
    pub produtos: usize,
    pub clientes: usize,
    pub compras: usize,
    pub similaridades: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErroCarga {
    pub arquivo: &'static str,
    pub linha: usize,
    pub motivo: String,
}

impl std::fmt::Display for ErroCarga {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (linha {}): {}", self.arquivo, self.linha, self.motivo)
    }
}

fn erro(arquivo: &'static str, linha: usize, motivo: impl Into<String>) -> ErroCarga {
    ErroCarga { arquivo, linha, motivo: motivo.into() }
}

fn linhas_de_dados(csv: &str) -> impl Iterator<Item = (usize, &str)> {
    csv.lines()
        .enumerate()
        .skip(1)
        .map(|(i, linha)| (i + 1, linha.trim()))
        .filter(|(_, linha)| !linha.is_empty())
}

pub fn carregar(grafo: &mut Grafo, produtos_csv: &str, compras_csv: &str) -> Result<Resumo, ErroCarga> {
    const ARQ_PRODUTOS: &str = "produtos.csv";
    const ARQ_COMPRAS: &str = "compras.csv";

    let mut produtos: HashMap<String, IdVertice> = HashMap::new();
    for (n, linha) in linhas_de_dados(produtos_csv) {
        let campos: Vec<&str> = linha.split(',').map(str::trim).collect();
        let [nome, preco] = campos[..] else {
            return Err(erro(ARQ_PRODUTOS, n, "esperado: nome,preco"));
        };
        let preco: f64 = preco
            .parse()
            .map_err(|_| erro(ARQ_PRODUTOS, n, format!("preço inválido: '{}'", preco)))?;
        if produtos.contains_key(nome) {
            return Err(erro(ARQ_PRODUTOS, n, format!("produto repetido: '{}'", nome)));
        }
        let id = grafo.cadastrar_produto(nome, preco).map_err(|e| erro(ARQ_PRODUTOS, n, e.to_string()))?;
        produtos.insert(nome.to_string(), id);
    }

    let mut clientes: HashMap<String, IdVertice> = HashMap::new();
    let mut ja_comprados: HashSet<(IdVertice, IdVertice)> = HashSet::new();
    let mut compras = 0;
    for (n, linha) in linhas_de_dados(compras_csv) {
        let campos: Vec<&str> = linha.split(',').map(str::trim).collect();
        let [nome_cliente, nome_produto] = campos[..] else {
            return Err(erro(ARQ_COMPRAS, n, "esperado: cliente,produto"));
        };
        let Some(&produto) = produtos.get(nome_produto) else {
            return Err(erro(ARQ_COMPRAS, n, format!("produto desconhecido: '{}'", nome_produto)));
        };
        let cliente = match clientes.get(nome_cliente) {
            Some(&id) => id,
            None => {
                let id = grafo.cadastrar_cliente(nome_cliente).map_err(|e| erro(ARQ_COMPRAS, n, e.to_string()))?;
                clientes.insert(nome_cliente.to_string(), id);
                id
            }
        };
        if !ja_comprados.insert((cliente, produto)) {
            continue; 
        }
        grafo
            .conectar(cliente, produto, TipoAresta::Compra)
            .expect("cliente e produto acabaram de ser cadastrados");
        compras += 1;
    }

    let similaridades = gerar_similaridades(grafo);
    Ok(Resumo { produtos: produtos.len(), clientes: clientes.len(), compras, similaridades })
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErroCompra {
    ClienteInvalido,
    ProdutoInvalido,
}

impl std::fmt::Display for ErroCompra {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroCompra::ClienteInvalido => write!(f, "cliente não encontrado"),
            ErroCompra::ProdutoInvalido => write!(f, "produto não encontrado"),
        }
    }
}

fn produtos_comprados(grafo: &Grafo, cliente: IdVertice) -> Vec<IdVertice> {
    let mut comprados: Vec<IdVertice> = grafo
        .vizinhos(cliente)
        .iter()
        .filter(|a| matches!(a.tipo, TipoAresta::Compra))
        .map(|a| a.destino)
        .collect();
    comprados.sort();
    comprados.dedup();
    comprados
}

pub fn registrar_compra(grafo: &mut Grafo, cliente: IdVertice, produto: IdVertice) -> Result<bool, ErroCompra> {
    if !matches!(grafo.buscar_por_id(cliente), Some(Vertice::Cliente { .. })) {
        return Err(ErroCompra::ClienteInvalido);
    }
    if grafo.buscar_produto_por_id(produto).is_none() {
        return Err(ErroCompra::ProdutoInvalido);
    }

    let anteriores = produtos_comprados(grafo, cliente);
    if anteriores.contains(&produto) {
        return Ok(false);
    }

    grafo
        .conectar(cliente, produto, TipoAresta::Compra)
        .expect("cliente e produto foram validados");
    for outro in anteriores {
        grafo.reforcar_similaridade(outro, produto, 1.0).expect("produtos existem");
    }
    Ok(true)
}

pub fn gerar_similaridades(grafo: &mut Grafo) -> usize {
    let mut em_comum: BTreeMap<(IdVertice, IdVertice), u32> = BTreeMap::new();

    for cliente in grafo.listar_clientes() {
        let comprados = produtos_comprados(grafo, cliente.id());
        for i in 0..comprados.len() {
            for j in (i + 1)..comprados.len() {
                *em_comum.entry((comprados[i], comprados[j])).or_insert(0) += 1;
            }
        }
    }

    let pares = em_comum.len();
    for ((a, b), clientes) in em_comum {
        let pontuacao = clientes as f64;
        grafo.conectar(a, b, TipoAresta::Similaridade { pontuacao }).expect("produtos existem");
        grafo.conectar(b, a, TipoAresta::Similaridade { pontuacao }).expect("produtos existem");
    }
    pares
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRODUTOS: &str = "nome,preco\nTenis,200.00\nMeia,20.00\nBone,50.00\n";

    fn pontuacao(g: &Grafo, de: IdVertice, para: IdVertice) -> Option<f64> {
        g.vizinhos(de).iter().find_map(|a| match a.tipo {
            TipoAresta::Similaridade { pontuacao } if a.destino == para => Some(pontuacao),
            _ => None,
        })
    }

    #[test]
    fn carrega_e_conta() {
        let mut g = Grafo::novo();
        let compras = "cliente,produto\nAna,Tenis\nAna,Meia\nBia,Tenis\n";
        let r = carregar(&mut g, PRODUTOS, compras).unwrap();
        assert_eq!(r, Resumo { produtos: 3, clientes: 2, compras: 3, similaridades: 1 });
    }

    #[test]
    fn peso_eh_o_numero_de_clientes_em_comum() {
        let mut g = Grafo::novo();
        let compras = "cliente,produto\nAna,Tenis\nAna,Meia\nBia,Tenis\nBia,Meia\nCaio,Tenis\nCaio,Bone\n";
        carregar(&mut g, PRODUTOS, compras).unwrap();
        let tenis = g.buscar_produtos_por_nome("tenis")[0].id();
        let meia = g.buscar_produtos_por_nome("meia")[0].id();
        let bone = g.buscar_produtos_por_nome("bone")[0].id();
        assert_eq!(pontuacao(&g, tenis, meia), Some(2.0));
        assert_eq!(pontuacao(&g, meia, tenis), Some(2.0));
        assert_eq!(pontuacao(&g, tenis, bone), Some(1.0));
        assert_eq!(pontuacao(&g, meia, bone), None);
    }

    #[test]
    fn compra_repetida_nao_conta_em_dobro() {
        let mut g = Grafo::novo();
        let compras = "cliente,produto\nAna,Tenis\nAna,Tenis\nAna,Meia\n";
        let r = carregar(&mut g, PRODUTOS, compras).unwrap();
        assert_eq!(r.compras, 2);
        let tenis = g.buscar_produtos_por_nome("tenis")[0].id();
        let meia = g.buscar_produtos_por_nome("meia")[0].id();
        assert_eq!(pontuacao(&g, tenis, meia), Some(1.0));
    }

    fn id_de(g: &Grafo, nome: &str) -> IdVertice {
        g.buscar_por_nome(nome).into_iter().find(|v| v.nome() == nome).unwrap().id()
    }

    fn similaridades_por_nome(g: &Grafo) -> BTreeMap<(String, String), f64> {
        let mut mapa = BTreeMap::new();
        for produto in g.listar_produtos() {
            for a in g.vizinhos(produto.id()) {
                if let TipoAresta::Similaridade { pontuacao } = a.tipo {
                    let destino = g.buscar_por_id(a.destino).unwrap().nome().to_string();
                    mapa.insert((produto.nome().to_string(), destino), pontuacao);
                }
            }
        }
        mapa
    }

    #[test]
    fn compra_nova_reforca_e_cria_similaridades() {
        let mut g = Grafo::novo();
        let compras = "cliente,produto\nAna,Tenis\nBia,Tenis\nBia,Meia\n";
        carregar(&mut g, PRODUTOS, compras).unwrap();
        let (ana, tenis, meia, bone) = (id_de(&g, "Ana"), id_de(&g, "Tenis"), id_de(&g, "Meia"), id_de(&g, "Bone"));
        assert_eq!(pontuacao(&g, tenis, meia), Some(1.0)); 

        assert_eq!(registrar_compra(&mut g, ana, meia), Ok(true));
        assert_eq!(pontuacao(&g, tenis, meia), Some(2.0)); 

        assert_eq!(registrar_compra(&mut g, ana, bone), Ok(true));
        assert_eq!(pontuacao(&g, tenis, bone), Some(1.0));
        assert_eq!(pontuacao(&g, bone, meia), Some(1.0));
        assert_eq!(pontuacao(&g, meia, bone), Some(1.0));
    }

    #[test]
    fn compra_repetida_nao_altera_nada() {
        let mut g = Grafo::novo();
        carregar(&mut g, PRODUTOS, "cliente,produto\nAna,Tenis\nAna,Meia\n").unwrap();
        let (ana, tenis, meia) = (id_de(&g, "Ana"), id_de(&g, "Tenis"), id_de(&g, "Meia"));
        let arestas = g.total_arestas();
        assert_eq!(registrar_compra(&mut g, ana, tenis), Ok(false));
        assert_eq!(g.total_arestas(), arestas);
        assert_eq!(pontuacao(&g, tenis, meia), Some(1.0));
    }

    #[test]
    fn registrar_compra_valida_cliente_e_produto() {
        let mut g = Grafo::novo();
        carregar(&mut g, PRODUTOS, "cliente,produto\nAna,Tenis\n").unwrap();
        let (ana, tenis) = (id_de(&g, "Ana"), id_de(&g, "Tenis"));
        assert_eq!(registrar_compra(&mut g, tenis, tenis), Err(ErroCompra::ClienteInvalido));
        assert_eq!(registrar_compra(&mut g, ana, ana), Err(ErroCompra::ProdutoInvalido));
        assert_eq!(registrar_compra(&mut g, 999, tenis), Err(ErroCompra::ClienteInvalido));
        assert_eq!(registrar_compra(&mut g, ana, 999), Err(ErroCompra::ProdutoInvalido));
    }

    #[test]
    fn incremental_gera_o_mesmo_resultado_que_o_calculo_completo() {
        let completo = "cliente,produto\nAna,Tenis\nAna,Meia\nAna,Bone\nBia,Tenis\nBia,Meia\nCaio,Meia\nCaio,Bone\n";
        let mut a = Grafo::novo();
        carregar(&mut a, PRODUTOS, completo).unwrap();

        let parcial = "cliente,produto\nAna,Tenis\nBia,Tenis\nCaio,Meia\n";
        let mut b = Grafo::novo();
        carregar(&mut b, PRODUTOS, parcial).unwrap();
        for (cliente, produto) in [("Ana", "Meia"), ("Ana", "Bone"), ("Bia", "Meia"), ("Caio", "Bone")] {
            let (c, p) = (id_de(&b, cliente), id_de(&b, produto));
            assert_eq!(registrar_compra(&mut b, c, p), Ok(true));
        }

        assert_eq!(similaridades_por_nome(&a), similaridades_por_nome(&b));
        assert!(!similaridades_por_nome(&a).is_empty());
    }

    #[test]
    fn erros_indicam_arquivo_e_linha() {
        let mut g = Grafo::novo();
        let e = carregar(&mut g, "nome,preco\nTenis,abc\n", "cliente,produto\n").unwrap_err();
        assert_eq!((e.arquivo, e.linha), ("produtos.csv", 2));

        let mut g = Grafo::novo();
        let e = carregar(&mut g, PRODUTOS, "cliente,produto\nAna,Tenis\nAna,Foguete\n").unwrap_err();
        assert_eq!((e.arquivo, e.linha), ("compras.csv", 3));

        let mut g = Grafo::novo();
        let e = carregar(&mut g, "nome,preco\nTenis,10\nTenis,20\n", "cliente,produto\n").unwrap_err();
        assert_eq!((e.arquivo, e.linha), ("produtos.csv", 3));

        let mut g = Grafo::novo();
        let e = carregar(&mut g, PRODUTOS, "cliente,produto\nAna\n").unwrap_err();
        assert_eq!((e.arquivo, e.linha), ("compras.csv", 2));
    }
}
