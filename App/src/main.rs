use axum::{
    extract::{Path as AxumPath, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
    Form,
};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tower_http::{services::ServeDir, cors::CorsLayer};
use std::net::SocketAddr;

// ===========================================
// ESTRUTURAS DE DADOS
// ===========================================

#[derive(Debug, Serialize, Deserialize)]
struct Secao {
    id: i32,
    nome: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tipo {
    id: i32,
    nome: String,
    id_secao: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Produto {
    id: i32,
    nome: String,
    id_tipo: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Lote {
    id: i32,
    id_produto: i32,
    validade: String,
    quantidade_total: i32,
    quantidade_prateleira: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct RelatorioItem {
    id: i32,
    nome: String,
    tipo: String,
    total: i32,
    prateleira: i32,
    estoque: i32,
    filhos: Vec<RelatorioItem>,
}

#[derive(Debug, Deserialize)]
struct LoginData {
    nome: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct ProdutoData {
    nome: String,
    tipo_id: i32,
}

#[derive(Debug, Deserialize)]
struct LoteData {
    produto_id: i32,
    validade: String,
    quantidade_total: i32,
    quantidade_prateleira: i32,
}

#[derive(Debug, Deserialize)]
struct VendaData {
    quantidade: i32,
}

// ===========================================
// INICIALIZAÇÃO DO BANCO DE DADOS
// ===========================================

fn init_db() -> Result<Connection, rusqlite::Error> {
    let db_path = "./dados";
    if !Path::new(db_path).exists() {
        fs::create_dir_all(db_path).expect("Erro ao criar pasta dados");
    }

    let conn = Connection::open("./dados/produtos.db")?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS secoes (
            id INTEGER PRIMARY KEY,
            nome TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tipos (
            id INTEGER PRIMARY KEY,
            nome TEXT NOT NULL,
            id_secao INTEGER NOT NULL,
            FOREIGN KEY (id_secao) REFERENCES secoes(id) ON DELETE CASCADE,
            UNIQUE(nome, id_secao)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS produtos (
            id INTEGER PRIMARY KEY,
            nome TEXT NOT NULL,
            id_tipo INTEGER NOT NULL,
            FOREIGN KEY (id_tipo) REFERENCES tipos(id) ON DELETE CASCADE,
            UNIQUE(nome, id_tipo)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS lotes (
            id INTEGER PRIMARY KEY,
            id_produto INTEGER NOT NULL,
            validade DATE NOT NULL,
            quantidade_total INTEGER NOT NULL,
            quantidade_prateleira INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (id_produto) REFERENCES produtos(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok(conn)
}

// ===========================================
// HANDLER DE LOGIN
// ===========================================

async fn login_handler(Form(login): Form<LoginData>) -> Json<bool> {
    let conn = match Connection::open("./dados/logins.db") {
        Ok(c) => c,
        Err(_) => return Json(false)
    };
    
    let existe: bool = conn.query_row(
        "SELECT 1 FROM usuarios WHERE nome = ?1 AND password = ?2",
        [&login.nome, &login.password],
        |_| Ok(true)
    ).unwrap_or(false);
    
    Json(existe)
}

// ===========================================
// HANDLERS DE SEÇÕES
// ===========================================

async fn listar_secoes_handler() -> Result<Json<Vec<Secao>>, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut stmt = conn.prepare("SELECT id, nome FROM secoes ORDER BY nome")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let secoes = stmt.query_map([], |row| {
        Ok(Secao {
            id: row.get(0)?,
            nome: row.get(1)?,
        })
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut resultado = Vec::new();
    for secao in secoes {
        resultado.push(secao.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }
    
    Ok(Json(resultado))
}

async fn criar_secao_handler(Form(secao): Form<Secao>) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    conn.execute(
        "INSERT INTO secoes (nome) VALUES (?1)",
        [&secao.nome],
    ).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok("Seção criada".to_string())
}

async fn deletar_secao_handler(AxumPath(id): AxumPath<i32>) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    conn.execute(
        "DELETE FROM secoes WHERE id = ?1",
        [id],
    ).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok("Seção deletada".to_string())
}

// ===========================================
// HANDLERS DE TIPOS
// ===========================================

async fn listar_tipos_handler(AxumPath(secao_id): AxumPath<i32>) -> Result<Json<Vec<Tipo>>, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, nome, id_secao FROM tipos WHERE id_secao = ?1 ORDER BY nome"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let tipos = stmt.query_map([secao_id], |row| {
        Ok(Tipo {
            id: row.get(0)?,
            nome: row.get(1)?,
            id_secao: row.get(2)?,
        })
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut resultado = Vec::new();
    for tipo in tipos {
        resultado.push(tipo.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }
    
    Ok(Json(resultado))
}

async fn criar_tipo_handler(Form(tipo): Form<Tipo>) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    conn.execute(
        "INSERT INTO tipos (nome, id_secao) VALUES (?1, ?2)",
        params![&tipo.nome, &tipo.id_secao],
    ).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok("Tipo criado".to_string())
}

async fn deletar_tipo_handler(AxumPath(id): AxumPath<i32>) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    conn.execute(
        "DELETE FROM tipos WHERE id = ?1",
        [id],
    ).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok("Tipo deletado".to_string())
}

// ===========================================
// HANDLERS DE PRODUTOS
// ===========================================

async fn listar_produtos_handler(AxumPath(tipo_id): AxumPath<i32>) -> Result<Json<Vec<Produto>>, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, nome, id_tipo FROM produtos WHERE id_tipo = ?1 ORDER BY nome"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let produtos = stmt.query_map([tipo_id], |row| {
        Ok(Produto {
            id: row.get(0)?,
            nome: row.get(1)?,
            id_tipo: row.get(2)?,
        })
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut resultado = Vec::new();
    for produto in produtos {
        resultado.push(produto.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }
    
    Ok(Json(resultado))
}

async fn criar_produto_handler(Form(produto): Form<ProdutoData>) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let nome_maiusculo = produto.nome.to_uppercase();
    
    conn.execute(
        "INSERT INTO produtos (nome, id_tipo) VALUES (?1, ?2)",
        params![nome_maiusculo, produto.tipo_id],
    ).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok("Produto criado".to_string())
}

async fn deletar_produto_handler(AxumPath(id): AxumPath<i32>) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    conn.execute(
        "DELETE FROM produtos WHERE id = ?1",
        [id],
    ).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok("Produto deletado".to_string())
}

// ===========================================
// HANDLERS DE LOTES
// ===========================================

async fn listar_lotes_handler(AxumPath(produto_id): AxumPath<i32>) -> Result<Json<Vec<Lote>>, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, id_produto, validade, quantidade_total, quantidade_prateleira 
         FROM lotes WHERE id_produto = ?1 ORDER BY validade"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let lotes = stmt.query_map([produto_id], |row| {
        Ok(Lote {
            id: row.get(0)?,
            id_produto: row.get(1)?,
            validade: row.get(2)?,
            quantidade_total: row.get(3)?,
            quantidade_prateleira: row.get(4)?,
        })
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut resultado = Vec::new();
    for lote in lotes {
        resultado.push(lote.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }
    
    Ok(Json(resultado))
}

async fn criar_lote_handler(Form(lote): Form<LoteData>) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if lote.quantidade_prateleira > lote.quantidade_total {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    conn.execute(
        "INSERT INTO lotes (id_produto, validade, quantidade_total, quantidade_prateleira) 
         VALUES (?1, ?2, ?3, ?4)",
        params![
            lote.produto_id,
            lote.validade,
            lote.quantidade_total,
            lote.quantidade_prateleira
        ],
    ).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok("Lote criado".to_string())
}

async fn deletar_lote_handler(AxumPath(id): AxumPath<i32>) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    conn.execute(
        "DELETE FROM lotes WHERE id = ?1",
        [id],
    ).map_err(|_| StatusCode::BAD_REQUEST)?;
    
    Ok("Lote deletado".to_string())
}

// ===========================================
// FUNÇÕES DE NEGÓCIO
// ===========================================

async fn vender_lote_handler(
    AxumPath(id): AxumPath<i32>,
    Form(venda): Form<VendaData>
) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let (na_prateleira,): (i32,) = conn.query_row(
        "SELECT quantidade_prateleira FROM lotes WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?,))
    ).map_err(|_| StatusCode::NOT_FOUND)?;
    
    if venda.quantidade > na_prateleira {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let nova_prateleira = na_prateleira - venda.quantidade;
    
    conn.execute(
        "UPDATE lotes SET quantidade_prateleira = ?1 WHERE id = ?2",
        params![nova_prateleira, id],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(format!("Vendido: {} unidades", venda.quantidade))
}

async fn abastecer_handler(
    AxumPath(id): AxumPath<i32>,
    Form(abastecimento): Form<VendaData>
) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let (na_prateleira, total): (i32, i32) = conn.query_row(
        "SELECT quantidade_prateleira, quantidade_total FROM lotes WHERE id = ?1",
        [id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).map_err(|_| StatusCode::NOT_FOUND)?;
    
    if na_prateleira + abastecimento.quantidade > total {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    let nova_prateleira = na_prateleira + abastecimento.quantidade;
    
    conn.execute(
        "UPDATE lotes SET quantidade_prateleira = ?1 WHERE id = ?2",
        params![nova_prateleira, id],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(format!("Abastecido: {} unidades", abastecimento.quantidade))
}

// ===========================================
// PESQUISA
// ===========================================

async fn pesquisar_handler(Query(params): Query<Vec<(String, String)>>) -> Result<Json<Vec<Produto>>, StatusCode> {
    let termo = params.iter()
        .find(|(k, _)| k == "q")
        .map(|(_, v)| v)
        .unwrap_or(&String::new())
        .to_uppercase();
    
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let termo_busca = format!("%{}%", termo);
    
    let mut stmt = conn.prepare(
        "SELECT id, nome, id_tipo FROM produtos WHERE UPPER(nome) LIKE ?1 ORDER BY nome"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let produtos = stmt.query_map([termo_busca], |row| {
        Ok(Produto {
            id: row.get(0)?,
            nome: row.get(1)?,
            id_tipo: row.get(2)?,
        })
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut resultado = Vec::new();
    for produto in produtos {
        resultado.push(produto.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }
    
    Ok(Json(resultado))
}

// ===========================================
// RELATÓRIO COMPLETO
// ===========================================

async fn relatorio_handler() -> Result<Json<Vec<RelatorioItem>>, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut stmt = conn.prepare(
        "SELECT s.id, s.nome, 
                COALESCE(SUM(l.quantidade_total), 0) as total,
                COALESCE(SUM(l.quantidade_prateleira), 0) as prateleira
         FROM secoes s
         LEFT JOIN tipos t ON s.id = t.id_secao
         LEFT JOIN produtos p ON t.id = p.id_tipo
         LEFT JOIN lotes l ON p.id = l.id_produto
         GROUP BY s.id"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut resultado = Vec::new();
    let secoes = stmt.query_map([], |row| {
        Ok(RelatorioItem {
            id: row.get(0)?,
            nome: row.get(1)?,
            tipo: "secao".to_string(),
            total: row.get(2)?,
            prateleira: row.get(3)?,
            estoque: row.get::<_, i32>(2)? - row.get::<_, i32>(3)?,
            filhos: Vec::new(),
        })
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for secao in secoes {
        let mut secao_item = secao.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let mut stmt_tipos = conn.prepare(
            "SELECT t.id, t.nome,
                    COALESCE(SUM(l.quantidade_total), 0) as total,
                    COALESCE(SUM(l.quantidade_prateleira), 0) as prateleira
             FROM tipos t
             LEFT JOIN produtos p ON t.id = p.id_tipo
             LEFT JOIN lotes l ON p.id = l.id_produto
             WHERE t.id_secao = ?1
             GROUP BY t.id"
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let tipos = stmt_tipos.query_map([secao_item.id], |row| {
            Ok(RelatorioItem {
                id: row.get(0)?,
                nome: row.get(1)?,
                tipo: "tipo".to_string(),
                total: row.get(2)?,
                prateleira: row.get(3)?,
                estoque: row.get::<_, i32>(2)? - row.get::<_, i32>(3)?,
                filhos: Vec::new(),
            })
        }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        for tipo in tipos {
            let mut tipo_item = tipo.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let mut stmt_produtos = conn.prepare(
                "SELECT p.id, p.nome,
                        COALESCE(SUM(l.quantidade_total), 0) as total,
                        COALESCE(SUM(l.quantidade_prateleira), 0) as prateleira
                 FROM produtos p
                 LEFT JOIN lotes l ON p.id = l.id_produto
                 WHERE p.id_tipo = ?1
                 GROUP BY p.id"
            ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let produtos = stmt_produtos.query_map([tipo_item.id], |row| {
                Ok(RelatorioItem {
                    id: row.get(0)?,
                    nome: row.get(1)?,
                    tipo: "produto".to_string(),
                    total: row.get(2)?,
                    prateleira: row.get(3)?,
                    estoque: row.get::<_, i32>(2)? - row.get::<_, i32>(3)?,
                    filhos: Vec::new(),
                })
            }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            for produto in produtos {
                let mut produto_item = produto.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                
                let mut stmt_lotes = conn.prepare(
                    "SELECT l.id, l.validade, l.quantidade_total, l.quantidade_prateleira
                     FROM lotes l
                     WHERE l.id_produto = ?1"
                ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                
                let lotes = stmt_lotes.query_map([produto_item.id], |row| {
                    Ok(RelatorioItem {
                        id: row.get(0)?,
                        nome: format!("Lote {}", row.get::<_, String>(1)?),
                        tipo: "lote".to_string(),
                        total: row.get(2)?,
                        prateleira: row.get(3)?,
                        estoque: row.get::<_, i32>(2)? - row.get::<_, i32>(3)?,
                        filhos: Vec::new(),
                    })
                }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                
                for lote in lotes {
                    produto_item.filhos.push(lote.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
                }
                
                tipo_item.filhos.push(produto_item);
            }
            
            secao_item.filhos.push(tipo_item);
        }
        
        resultado.push(secao_item);
    }
    
    Ok(Json(resultado))
}

// ===========================================
// CSV
// ===========================================

async fn exportar_csv_handler() -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut stmt = conn.prepare(
        "SELECT s.nome, t.nome, p.nome, l.validade, l.quantidade_total, l.quantidade_prateleira
         FROM secoes s
         JOIN tipos t ON s.id = t.id_secao
         JOIN produtos p ON t.id = p.id_tipo
         JOIN lotes l ON p.id = l.id_produto
         ORDER BY s.nome, t.nome, p.nome"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut csv = String::from("Seção,Tipo,Produto,Validade,Total,Prateleira\n");
    let linhas = stmt.query_map([], |row| {
        Ok(format!(
            "{},{},{},{},{},{}\n",
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i32>(4)?,
            row.get::<_, i32>(5)?
        ))
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    for linha in linhas {
        csv += &linha.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    Ok(csv)
}

async fn importar_csv_handler(csv_data: String) -> Result<String, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    conn.execute("DELETE FROM lotes", []).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    conn.execute("DELETE FROM produtos", []).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    conn.execute("DELETE FROM tipos", []).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    conn.execute("DELETE FROM secoes", []).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut linhas_importadas = 0;
    let mut erros = 0;
    
    for (i, line) in csv_data.lines().enumerate() {
        if i == 0 { continue; }
        
        let line = line.trim();
        if line.is_empty() { continue; }
        
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 6 {
            erros += 1;
            continue;
        }
        
        let secao_nome = cols[0].trim();
        let tipo_nome = cols[1].trim();
        let produto_nome = cols[2].trim().to_uppercase();
        let validade = cols[3].trim();
        let quantidade_total = cols[4].trim().parse::<i32>().unwrap_or(0);
        let quantidade_prateleira = cols[5].trim().parse::<i32>().unwrap_or(0);
        
        conn.execute(
            "INSERT INTO secoes (nome) VALUES (?1)",
            [secao_nome],
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let secao_id: i32 = conn.query_row(
            "SELECT id FROM secoes WHERE nome = ?1",
            [secao_nome],
            |row| row.get(0)
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        conn.execute(
            "INSERT INTO tipos (nome, id_secao) VALUES (?1, ?2)",
            params![tipo_nome, secao_id],
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let tipo_id: i32 = conn.query_row(
            "SELECT id FROM tipos WHERE nome = ?1 AND id_secao = ?2",
            params![tipo_nome, secao_id],
            |row| row.get(0)
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        conn.execute(
            "INSERT INTO produtos (nome, id_tipo) VALUES (?1, ?2)",
            params![produto_nome, tipo_id],
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let produto_id: i32 = conn.query_row(
            "SELECT id FROM produtos WHERE nome = ?1 AND id_tipo = ?2",
            params![produto_nome, tipo_id],
            |row| row.get(0)
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        conn.execute(
            "INSERT INTO lotes (id_produto, validade, quantidade_total, quantidade_prateleira) 
             VALUES (?1, ?2, ?3, ?4)",
            params![produto_id, validade, quantidade_total, quantidade_prateleira],
        ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        linhas_importadas += 1;
    }
    
    Ok(format!("Importados {} lotes, {} erros", linhas_importadas, erros))
}

// ===========================================
// PRODUTOS A VENCER
// ===========================================

async fn produtos_a_vencer_handler(AxumPath(dias): AxumPath<i32>) -> Result<Json<Vec<Lote>>, StatusCode> {
    let conn = init_db().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut stmt = conn.prepare(
        "SELECT l.id, l.id_produto, l.validade, l.quantidade_total, l.quantidade_prateleira
         FROM lotes l
         WHERE julianday(l.validade) - julianday('now') <= ?1"
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let lotes = stmt.query_map([dias], |row| {
        Ok(Lote {
            id: row.get(0)?,
            id_produto: row.get(1)?,
            validade: row.get(2)?,
            quantidade_total: row.get(3)?,
            quantidade_prateleira: row.get(4)?,
        })
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut resultado = Vec::new();
    for lote in lotes {
        resultado.push(lote.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    }
    
    Ok(Json(resultado))
}

// ===========================================
// MAIN
// ===========================================

#[tokio::main]
async fn main() {
    let app = Router::new()
        // Login
        .route("/api/login", post(login_handler))
        
        // Seções
        .route("/api/secoes", get(listar_secoes_handler))
        .route("/api/secoes", post(criar_secao_handler))
        .route("/api/secoes/:id", delete(deletar_secao_handler))
        
        // Tipos
        .route("/api/tipos/secao/:secao_id", get(listar_tipos_handler))
        .route("/api/tipos", post(criar_tipo_handler))
        .route("/api/tipos/:id", delete(deletar_tipo_handler))
        
        // Produtos
        .route("/api/produtos/tipo/:tipo_id", get(listar_produtos_handler))
        .route("/api/produtos", post(criar_produto_handler))
        .route("/api/produtos/:id", delete(deletar_produto_handler))
        
        // Lotes
        .route("/api/lotes/produto/:produto_id", get(listar_lotes_handler))
        .route("/api/lotes", post(criar_lote_handler))
        .route("/api/lotes/:id", delete(deletar_lote_handler))
        
        // Negócio
        .route("/api/vender/:id", post(vender_lote_handler))
        .route("/api/abastecer/:id", post(abastecer_handler))
        
        // Pesquisa
        .route("/api/pesquisar", get(pesquisar_handler))
        
        // Relatório
        .route("/api/relatorio", get(relatorio_handler))
        
        // CSV
        .route("/api/exportar", get(exportar_csv_handler))
        .route("/api/importar", post(importar_csv_handler))
        
        // Validade
        .route("/api/vencer/:dias", get(produtos_a_vencer_handler))
        
        // Arquivos estáticos
        .fallback_service(ServeDir::new("dist"))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Servidor completo rodando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
