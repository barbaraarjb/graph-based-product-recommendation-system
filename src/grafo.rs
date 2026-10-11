use std::collections::HashMap;

pub type IdVertice = u32;

#[derive(Debug, Clone, PartialEq)]
pub enum Vertice {
    Produto { id: IdVertice, nome: String, preco: f64 },
    Cliente { id: IdVertice, nome: String },
}

impl Vertice {
    pub fn id(&self) -> IdVertice {
        match self {
            Vertice::Produto { id, .. } => *id,
            Vertice::Cliente { id, .. } => *id,
        }
    }

    pub fn nome(&self) -> &str {
        match self {
            Vertice::Produto { nome, .. } => nome,
            Vertice::Cliente { nome, .. } => nome,
        }
    }

    pub fn eh_produto(&self) -> bool {
        matches!(self, Vertice::Produto { .. })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoAresta {
    Compra,
    Similaridade { pontuacao: f64 },
}

#[derive(Debug, Clone)]
pub struct Aresta {
    pub origem: IdVertice,
    pub destino: IdVertice,
    pub tipo: TipoAresta,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErroCadastro {
    NomeVazio,
    PrecoInvalido,
}

impl std::fmt::Display for ErroCadastro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroCadastro::NomeVazio => write!(f, "o nome do produto não pode ser vazio"),
            ErroCadastro::PrecoInvalido => write!(f, "o preço do produto deve ser maior que zero"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErroConexao {
    OrigemInexistente,
    DestinoInexistente,
}

impl std::fmt::Display for ErroConexao {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroConexao::OrigemInexistente => write!(f, "o vértice de origem não existe"),
            ErroConexao::DestinoInexistente => write!(f, "o vértice de destino não existe"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Grafo {
    vertices: HashMap<IdVertice, Vertice>,
    adjacencia: HashMap<IdVertice, Vec<Aresta>>,
    proximo_id: IdVertice,
}

impl Grafo {
    pub fn novo() -> Self {
        Grafo {
            vertices: HashMap::new(),
            adjacencia: HashMap::new(),
            proximo_id: 0,
        }
    }

    pub fn adicionar_vertice(&mut self, construtor: impl FnOnce(IdVertice) -> Vertice) -> IdVertice {
        let id = self.proximo_id;
        let vertice = construtor(id);
        self.vertices.insert(id, vertice);
        self.adjacencia.insert(id, Vec::new());
        self.proximo_id += 1;
        id
    }

    pub fn adicionar_aresta(&mut self, origem: IdVertice, destino: IdVertice, tipo: TipoAresta) {
        let aresta = Aresta { origem, destino, tipo };
        self.adjacencia.entry(origem).or_insert_with(Vec::new).push(aresta);
    }

    pub fn cadastrar_produto(&mut self, nome: &str, preco: f64) -> Result<IdVertice, ErroCadastro> {
        let nome = nome.trim();
        if nome.is_empty() {
            return Err(ErroCadastro::NomeVazio);
        }
        if preco <= 0.0 {
            return Err(ErroCadastro::PrecoInvalido);
        }
        let nome = nome.to_string();
        Ok(self.adicionar_vertice(|id| Vertice::Produto { id, nome, preco }))
    }

    pub fn cadastrar_cliente(&mut self, nome: &str) -> Result<IdVertice, ErroCadastro> {
        let nome = nome.trim();
        if nome.is_empty() {
            return Err(ErroCadastro::NomeVazio);
        }
        let nome = nome.to_string();
        Ok(self.adicionar_vertice(|id| Vertice::Cliente { id, nome }))
    }

    pub fn conectar(&mut self, origem: IdVertice, destino: IdVertice, tipo: TipoAresta) -> Result<(), ErroConexao> {
        if !self.vertices.contains_key(&origem) {
            return Err(ErroConexao::OrigemInexistente);
        }
        if !self.vertices.contains_key(&destino) {
            return Err(ErroConexao::DestinoInexistente);
        }
        self.adicionar_aresta(origem, destino, tipo);
        Ok(())
    }

    pub fn reforcar_similaridade(&mut self, a: IdVertice, b: IdVertice, incremento: f64) -> Result<(), ErroConexao> {
        if !self.vertices.contains_key(&a) {
            return Err(ErroConexao::OrigemInexistente);
        }
        if !self.vertices.contains_key(&b) {
            return Err(ErroConexao::DestinoInexistente);
        }
        for (de, para) in [(a, b), (b, a)] {
            let arestas = self.adjacencia.entry(de).or_default();
            let existente = arestas
                .iter()
                .position(|ar| ar.destino == para && matches!(ar.tipo, TipoAresta::Similaridade { .. }));
            match existente {
                Some(i) => {
                    if let TipoAresta::Similaridade { pontuacao } = &mut arestas[i].tipo {
                        *pontuacao += incremento;
                    }
                }
                None => arestas.push(Aresta {
                    origem: de,
                    destino: para,
                    tipo: TipoAresta::Similaridade { pontuacao: incremento },
                }),
            }
        }
        Ok(())
    }

    pub fn buscar_por_id(&self, id: IdVertice) -> Option<&Vertice> {
        self.vertices.get(&id)
    }

    fn ordenado_por_id(mut vertices: Vec<&Vertice>) -> Vec<&Vertice> {
        vertices.sort_by_key(|v| v.id());
        vertices
    }

    pub fn buscar_por_nome(&self, termo: &str) -> Vec<&Vertice> {
        let termo_lower = termo.to_lowercase();
        Self::ordenado_por_id(
            self.vertices
                .values()
                .filter(|v| v.nome().to_lowercase().contains(&termo_lower))
                .collect(),
        )
    }

    pub fn listar_produtos(&self) -> Vec<&Vertice> {
        Self::ordenado_por_id(self.vertices.values().filter(|v| v.eh_produto()).collect())
    }

    pub fn listar_clientes(&self) -> Vec<&Vertice> {
        Self::ordenado_por_id(
            self.vertices.values().filter(|v| matches!(v, Vertice::Cliente { .. })).collect(),
        )
    }

    pub fn buscar_produto_por_id(&self, id: IdVertice) -> Option<&Vertice> {
        self.buscar_por_id(id).filter(|v| v.eh_produto())
    }

    pub fn buscar_produtos_por_nome(&self, termo: &str) -> Vec<&Vertice> {
        let termo_lower = termo.to_lowercase();
        Self::ordenado_por_id(
            self.vertices
                .values()
                .filter(|v| v.eh_produto() && v.nome().to_lowercase().contains(&termo_lower))
                .collect(),
        )
    }

    pub fn vizinhos(&self, id: IdVertice) -> &[Aresta] {
        self.adjacencia.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn total_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn total_arestas(&self) -> usize {
        self.adjacencia.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grafo_exemplo() -> (Grafo, IdVertice, IdVertice, IdVertice) {
        let mut g = Grafo::novo();
        let cliente = g.adicionar_vertice(|id| Vertice::Cliente { id, nome: "Ana".to_string() });
        let p1 = g.adicionar_vertice(|id| Vertice::Produto { id, nome: "Tenis".to_string(), preco: 299.90 });
        let p2 = g.adicionar_vertice(|id| Vertice::Produto { id, nome: "Meia".to_string(), preco: 29.90 });
        g.adicionar_aresta(cliente, p1, TipoAresta::Compra);
        g.adicionar_aresta(p1, p2, TipoAresta::Similaridade { pontuacao: 0.85 });
        (g, cliente, p1, p2)
    }

    #[test]
    fn adiciona_e_busca_vertice_por_id() {
        let (g, _cliente, p1, _p2) = grafo_exemplo();
        let v = g.buscar_por_id(p1).unwrap();
        assert_eq!(v.nome(), "Tenis");
    }

    #[test]
    fn busca_por_id_inexistente_retorna_none() {
        let (g, ..) = grafo_exemplo();
        assert!(g.buscar_por_id(9999).is_none());
    }

    #[test]
    fn busca_por_nome_eh_case_insensitive() {
        let (g, ..) = grafo_exemplo();
        let resultado = g.buscar_por_nome("tenis");
        assert_eq!(resultado.len(), 1);
    }

    #[test]
    fn listar_produtos_ignora_clientes() {
        let (g, ..) = grafo_exemplo();
        let produtos = g.listar_produtos();
        assert_eq!(produtos.len(), 2);
    }

    #[test]
    fn vizinhos_retorna_arestas_corretas() {
        let (g, cliente, p1, _p2) = grafo_exemplo();
        let arestas = g.vizinhos(cliente);
        assert_eq!(arestas.len(), 1);
        assert_eq!(arestas[0].destino, p1);
    }

    #[test]
    fn cadastrar_produto_valido() {
        let mut g = Grafo::novo();
        let id = g.cadastrar_produto("Tenis", 299.90).unwrap();
        let v = g.buscar_produto_por_id(id).unwrap();
        assert_eq!(v.nome(), "Tenis");
    }

    #[test]
    fn cadastrar_produto_com_nome_vazio_falha() {
        let mut g = Grafo::novo();
        assert_eq!(g.cadastrar_produto("   ", 10.0), Err(ErroCadastro::NomeVazio));
    }

    #[test]
    fn cadastrar_produto_com_preco_invalido_falha() {
        let mut g = Grafo::novo();
        assert_eq!(g.cadastrar_produto("Tenis", 0.0), Err(ErroCadastro::PrecoInvalido));
        assert_eq!(g.cadastrar_produto("Tenis", -5.0), Err(ErroCadastro::PrecoInvalido));
    }

    #[test]
    fn buscar_produto_por_id_ignora_nao_produtos() {
        let (g, cliente, ..) = grafo_exemplo();
        assert!(g.buscar_produto_por_id(cliente).is_none());
    }

    #[test]
    fn cadastrar_cliente_valido_e_nome_vazio() {
        let mut g = Grafo::novo();
        let id = g.cadastrar_cliente(" Ana ").unwrap();
        assert_eq!(g.buscar_por_id(id).unwrap().nome(), "Ana");
        assert_eq!(g.cadastrar_cliente(""), Err(ErroCadastro::NomeVazio));
    }

    #[test]
    fn conectar_valida_existencia_dos_vertices() {
        let mut g = Grafo::novo();
        let c = g.cadastrar_cliente("Ana").unwrap();
        let p = g.cadastrar_produto("Tenis", 100.0).unwrap();
        assert!(g.conectar(c, p, TipoAresta::Compra).is_ok());
        assert_eq!(g.total_arestas(), 1);
        assert_eq!(g.conectar(99, p, TipoAresta::Compra), Err(ErroConexao::OrigemInexistente));
        assert_eq!(g.conectar(c, 99, TipoAresta::Compra), Err(ErroConexao::DestinoInexistente));
        assert_eq!(g.total_arestas(), 1);
    }

    #[test]
    fn reforcar_similaridade_cria_e_depois_soma() {
        let mut g = Grafo::novo();
        let a = g.cadastrar_produto("A", 1.0).unwrap();
        let b = g.cadastrar_produto("B", 1.0).unwrap();
        g.reforcar_similaridade(a, b, 1.0).unwrap();
        g.reforcar_similaridade(b, a, 2.0).unwrap();

        assert_eq!(g.total_arestas(), 2);
        for (de, para) in [(a, b), (b, a)] {
            match g.vizinhos(de)[0].tipo {
                TipoAresta::Similaridade { pontuacao } => assert_eq!(pontuacao, 3.0),
                _ => panic!("esperava similaridade"),
            }
            assert_eq!(g.vizinhos(de)[0].destino, para);
        }
        assert_eq!(g.reforcar_similaridade(a, 99, 1.0), Err(ErroConexao::DestinoInexistente));
        assert_eq!(g.reforcar_similaridade(99, a, 1.0), Err(ErroConexao::OrigemInexistente));
    }

    #[test]
    fn buscar_produtos_por_nome_ignora_nao_produtos() {
        let mut g = Grafo::novo();
        g.adicionar_vertice(|id| Vertice::Cliente { id, nome: "Tenis da Ana".to_string() });
        g.cadastrar_produto("Tenis Esportivo", 299.90).unwrap();

        let resultado = g.buscar_produtos_por_nome("tenis");
        assert_eq!(resultado.len(), 1);
        assert!(resultado[0].eh_produto());
    }
}
