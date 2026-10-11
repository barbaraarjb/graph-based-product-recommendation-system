pub mod dados;
pub mod grafo;
pub mod recomendacao;

pub use grafo::{Aresta, ErroCadastro, ErroConexao, Grafo, IdVertice, TipoAresta, Vertice};
pub use recomendacao::recomendar;
pub use dados::{carregar, gerar_similaridades, registrar_compra, ErroCarga, ErroCompra, Resumo};
