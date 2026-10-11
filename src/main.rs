use megastore::{carregar, recomendar, registrar_compra as comprar, Grafo, IdVertice, Vertice};
use std::fs::OpenOptions;
use std::io::{self, Read, Seek, SeekFrom, Write};

const PROFUNDIDADE_PADRAO: u32 = 2;

const LIMITE_PADRAO: usize = 5;

const ARQUIVO_PRODUTOS: &str = "data/produtos.csv";
const ARQUIVO_COMPRAS: &str = "data/compras.csv";

fn main() {
    let mut grafo = Grafo::novo();
    carregar_dados(&mut grafo);

    loop {
        exibir_menu();
        match ler_linha("Escolha uma opção: ").trim() {
            "1" => cadastrar_produto(&mut grafo),
            "2" => listar_produtos(&grafo),
            "3" => buscar_produto_por_nome(&grafo),
            "4" => registrar_compra(&mut grafo),
            "5" => recomendar_para_todos(&grafo),
            "0" => {
                println!("Saindo...");
                break;
            }
            _ => println!("Opcao invalida.\n"),
        }
    }
}

fn carregar_dados(grafo: &mut Grafo) {
    let ler = |caminho: &str| std::fs::read_to_string(caminho).map_err(|e| format!("{}: {}", caminho, e));
    let resultado = ler(ARQUIVO_PRODUTOS)
        .and_then(|produtos| ler(ARQUIVO_COMPRAS).map(|compras| (produtos, compras)))
        .and_then(|(produtos, compras)| carregar(grafo, &produtos, &compras).map_err(|e| e.to_string()));
    match resultado {
        Ok(r) => println!(
            "Dados carregados: {} produtos, {} clientes, {} compras, {} pares similares.\n",
            r.produtos, r.clientes, r.compras, r.similaridades
        ),
        Err(erro) => println!("Nao foi possivel carregar os dados ({}). Iniciando vazio.\n", erro),
    }
}

fn exibir_menu() {
    println!("=== Megastore ===");
    println!("1. Cadastrar produto");
    println!("2. Listar produtos");
    println!("3. Buscar produto por nome");
    println!("4. Registrar compra do cliente");
    println!("5. Listar recomendações");
    println!("0. Sair");
}

fn ler_linha(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("falha ao escrever no terminal");
    let mut entrada = String::new();
    io::stdin().read_line(&mut entrada).expect("falha ao ler entrada");
    entrada
}

fn acrescentar_linha(caminho: &str, cabecalho: &str, linha: &str) -> io::Result<()> {
    let mut arquivo = OpenOptions::new().read(true).append(true).create(true).open(caminho)?;
    let mut texto = String::new();
    if arquivo.metadata()?.len() == 0 {
        texto.push_str(cabecalho);
        texto.push('\n');
    } else {
        arquivo.seek(SeekFrom::End(-1))?;
        let mut ultimo = [0u8; 1];
        arquivo.read_exact(&mut ultimo)?;
        if ultimo[0] != b'\n' {
            texto.push('\n');
        }
    }
    texto.push_str(linha);
    texto.push('\n');
    arquivo.write_all(texto.as_bytes())
}

fn gravar(caminho: &str, cabecalho: &str, linha: &str) {
    if let Err(erro) = acrescentar_linha(caminho, cabecalho, linha) {
        println!("Aviso: nao foi possivel gravar em {}: {}\n", caminho, erro);
    }
}

fn cadastrar_produto(grafo: &mut Grafo) {
    let nome = ler_linha("Nome do produto: ");
    let nome = nome.trim();
    let preco_str = ler_linha("Preco do produto: ");

    if nome.contains(',') {
        println!("O nome do produto nao pode conter virgula.\n");
        return;
    }
    if grafo.buscar_produtos_por_nome(nome).iter().any(|p| p.nome().to_lowercase() == nome.to_lowercase()) {
        println!("Ja existe um produto com esse nome.\n");
        return;
    }

    let preco: f64 = match preco_str.trim().replace(',', ".").parse() {
        Ok(valor) => valor,
        Err(_) => {
            println!("Preco invalido, use um numero (ex: 29.90).\n");
            return;
        }
    };

    match grafo.cadastrar_produto(nome, preco) {
        Ok(id) => {
            println!("Produto cadastrado com id {}.\n", id);
            gravar(ARQUIVO_PRODUTOS, "nome,preco", &format!("{},{}", nome, preco));
        }
        Err(erro) => println!("Erro ao cadastrar: {}\n", erro),
    }
}

fn listar_produtos(grafo: &Grafo) {
    let produtos = grafo.listar_produtos();
    if produtos.is_empty() {
        println!("Nenhum produto cadastrado.\n");
        return;
    }
    println!("--- Produtos cadastrados ---");
    for produto in produtos {
        imprimir_produto(produto);
    }
    println!();
}

fn buscar_produto_por_nome(grafo: &Grafo) {
    let termo = ler_linha("Nome (ou parte) a buscar: ");
    let resultados = grafo.buscar_produtos_por_nome(termo.trim());
    if resultados.is_empty() {
        println!("Nenhum produto encontrado.\n");
        return;
    }
    for produto in resultados {
        imprimir_produto(produto);
    }
    println!();
}

fn imprimir_produto(produto: &Vertice) {
    if let Vertice::Produto { id, nome, preco } = produto {
        println!("[{}] {} - R$ {:.2}", id, nome, preco);
    }
}

fn ler_numero<T: std::str::FromStr>(prompt: &str) -> Option<T> {
    ler_linha(prompt).trim().parse().ok()
}

fn registrar_compra(grafo: &mut Grafo) {
    let clientes = grafo.listar_clientes();
    if clientes.is_empty() {
        println!("Nenhum cliente cadastrado.\n");
        return;
    }
    println!("--- Clientes ---");
    for cliente in clientes {
        println!("[{}] {}", cliente.id(), cliente.nome());
    }
    println!();

    let (Some(cliente), Some(produto)) = (
        ler_numero::<IdVertice>("Id do cliente: "),
        ler_numero::<IdVertice>("Id do produto comprado: "),
    ) else {
        println!("Id invalido.\n");
        return;
    };
    match comprar(grafo, cliente, produto) {
        Ok(nova) => {
            let nome = |id| grafo.buscar_por_id(id).map(|v| v.nome()).unwrap_or_default();
            gravar(ARQUIVO_COMPRAS, "cliente,produto", &format!("{},{}", nome(cliente), nome(produto)));
            if nova {
                println!("Compra registrada. As recomendacoes ja consideram a nova compra.\n");
            } else {
                println!("Compra registrada. O cliente ja tinha comprado este produto, entao as recomendacoes nao mudam.\n");
            }
        }
        Err(erro) => println!("Erro ao registrar compra: {}\n", erro),
    }
}

fn recomendar_para_todos(grafo: &Grafo) {
    let clientes = grafo.listar_clientes();
    if clientes.is_empty() {
        println!("Nenhum cliente cadastrado.\n");
        return;
    }
    let largura = clientes.iter().map(|c| c.nome().chars().count()).max().unwrap_or(0) + 1;

    println!("--- Recomendacoes por cliente ---");
    for cliente in clientes {
        let nomes: Vec<&str> = recomendar(grafo, cliente.id(), PROFUNDIDADE_PADRAO, LIMITE_PADRAO)
            .into_iter()
            .filter_map(|id| grafo.buscar_por_id(id))
            .map(|v| v.nome())
            .collect();
        let lista = if nomes.is_empty() { "(nada novo)".to_string() } else { nomes.join(", ") };
        println!("{:<largura$} {}", format!("{}:", cliente.nome()), lista);
    }
    println!();
}

