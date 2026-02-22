use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ===========================================
// CONFIGURAÇÃO INICIAL (USUÁRIOS)
// ===========================================

#[tauri::command]
fn config() -> String {
    // Garante que a pasta dados existe
    let db_path = "../dados";
    if !Path::new(db_path).exists() {
        fs::create_dir_all(db_path).expect("Erro ao criar pasta dados");
    }
    
    let conn = Connection::open("../dados/logins.db").unwrap();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS usuarios (
            id INTEGER PRIMARY KEY,
            nome TEXT NOT NULL UNIQUE,
            password TEXT NOT NULL
        )",
        [],
    ).unwrap();

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM usuarios", [], |row| row.get(0)).unwrap();
    
    if count == 0 {
        conn.execute(
            "INSERT INTO usuarios (nome, password) VALUES (?1, ?2)",
            ["Araquari", "741852963"], // DEFINIR USUARIO E SENHA
        ).unwrap();
        "Usuário padrão criado".to_string()
    } else {
        "Banco já configurado".to_string()
    }
}

#[tauri::command]
fn verifique(nome: String, password: String) -> bool {
    println!("Verificando: {} / {}", nome, password);
    
    let conn = Connection::open("../dados/logins.db").unwrap();
    
    let mut stmt = conn.prepare(
        "SELECT * FROM usuarios WHERE nome = ?1 AND password = ?2"
    ).unwrap();
    
    stmt.exists([nome, password]).unwrap()
}

// ===========================================
// FUNÇÃO LEGADO (CADASTRAR)
// ===========================================

#[tauri::command]
fn cadastrar(produto: String, secao: String) -> Result<String, String> {
    let conn = Connection::open("../dados/produtos.db").map_err(|e| e.to_string())?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS produtos (
            id INTEGER PRIMARY KEY,
            secao TEXT NOT NULL,
            nome TEXT NOT NULL UNIQUE
        )",
        [],
    ).map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO produtos (secao, nome) VALUES (?1, ?2)",
        params![secao, produto],
    ).map_err(|e| e.to_string())?;
    
    Ok(format!("{} cadastrado com sucesso!", produto))
}

// ===========================================
// ESTRUTURAS DE DADOS (para retornar ao frontend)
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

// ===========================================
// ESTRUTURA PARA RELATÓRIO EM ÁRVORE
// ===========================================

#[derive(Debug, Serialize, Deserialize)]
struct RelatorioItem {
    id: i32,
    nome: String,
    tipo: String,  // "secao", "tipo", "produto", "lote"
    total: i32,
    prateleira: i32,
    estoque: i32,
    filhos: Vec<RelatorioItem>,
}

// ===========================================
// INICIALIZAÇÃO DO BANCO DE DADOS (PRODUTOS)
// ===========================================

fn init_produtos_db() -> Result<Connection> {
    let conn = Connection::open("../dados/produtos.db")?;
    
    // Tabela de seções
    conn.execute(
        "CREATE TABLE IF NOT EXISTS secoes (
            id INTEGER PRIMARY KEY,
            nome TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    // Tabela de tipos
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

    // Tabela de produtos
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

    // Tabela de lotes
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
// CRUD DE SEÇÕES
// ===========================================

#[tauri::command]
fn criar_secao(nome: String) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO secoes (nome) VALUES (?1)",
        [nome],
    ).map_err(|e| e.to_string())?;
    
    Ok("Seção criada com sucesso".to_string())
}

#[tauri::command]
fn listar_secoes() -> Result<Vec<Secao>, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, nome FROM secoes ORDER BY nome").map_err(|e| e.to_string())?;
    let secoes = stmt.query_map([], |row| {
        Ok(Secao {
            id: row.get(0)?,
            nome: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut resultado = Vec::new();
    for secao in secoes {
        resultado.push(secao.map_err(|e| e.to_string())?);
    }
    
    Ok(resultado)
}

#[tauri::command]
fn deletar_secao(id: i32) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    conn.execute(
        "DELETE FROM secoes WHERE id = ?1",
        [id],
    ).map_err(|e| e.to_string())?;
    
    Ok("Seção deletada com sucesso".to_string())
}

// ===========================================
// CRUD DE TIPOS
// ===========================================

#[tauri::command]
fn criar_tipo(nome: String, secao_id: i32) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO tipos (nome, id_secao) VALUES (?1, ?2)",
        params![nome, secao_id],
    ).map_err(|e| e.to_string())?;
    
    Ok("Tipo criado com sucesso".to_string())
}

#[tauri::command]
fn listar_tipos(secao_id: i32) -> Result<Vec<Tipo>, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, nome, id_secao FROM tipos WHERE id_secao = ?1 ORDER BY nome"
    ).map_err(|e| e.to_string())?;
    
    let tipos = stmt.query_map([secao_id], |row| {
        Ok(Tipo {
            id: row.get(0)?,
            nome: row.get(1)?,
            id_secao: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut resultado = Vec::new();
    for tipo in tipos {
        resultado.push(tipo.map_err(|e| e.to_string())?);
    }
    
    Ok(resultado)
}

#[tauri::command]
fn deletar_tipo(id: i32) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    conn.execute(
        "DELETE FROM tipos WHERE id = ?1",
        [id],
    ).map_err(|e| e.to_string())?;
    
    Ok("Tipo deletado com sucesso".to_string())
}

// ===========================================
// CRUD DE PRODUTOS
// ===========================================

#[tauri::command]
fn criar_produto(nome: String, tipo_id: i32) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    // Garantir que o nome seja maiúsculo
    let nome_maiusculo = nome.to_uppercase();
    
    conn.execute(
        "INSERT INTO produtos (nome, id_tipo) VALUES (?1, ?2)",
        params![nome_maiusculo, tipo_id],
    ).map_err(|e| e.to_string())?;
    
    Ok("Produto criado com sucesso".to_string())
}

#[tauri::command]
fn listar_produtos(tipo_id: i32) -> Result<Vec<Produto>, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, nome, id_tipo FROM produtos WHERE id_tipo = ?1 ORDER BY nome"
    ).map_err(|e| e.to_string())?;
    
    let produtos = stmt.query_map([tipo_id], |row| {
        Ok(Produto {
            id: row.get(0)?,
            nome: row.get(1)?,
            id_tipo: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut resultado = Vec::new();
    for produto in produtos {
        resultado.push(produto.map_err(|e| e.to_string())?);
    }
    
    Ok(resultado)
}

#[tauri::command]
fn deletar_produto(id: i32) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    conn.execute(
        "DELETE FROM produtos WHERE id = ?1",
        [id],
    ).map_err(|e| e.to_string())?;
    
    Ok("Produto deletado com sucesso".to_string())
}

// ===========================================
// CRUD DE LOTES
// ===========================================

#[tauri::command]
fn criar_lote(
    produto_id: i32,
    validade: String,
    quantidade_total: i32,
    quantidade_prateleira: i32,
) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    if quantidade_prateleira > quantidade_total {
        return Err("Quantidade na prateleira não pode ser maior que o total".to_string());
    }
    
    conn.execute(
        "INSERT INTO lotes (id_produto, validade, quantidade_total, quantidade_prateleira) 
         VALUES (?1, ?2, ?3, ?4)",
        params![produto_id, validade, quantidade_total, quantidade_prateleira],
    ).map_err(|e| e.to_string())?;
    
    Ok("Lote criado com sucesso".to_string())
}

#[tauri::command]
fn listar_lotes(produto_id: i32) -> Result<Vec<Lote>, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, id_produto, validade, quantidade_total, quantidade_prateleira 
         FROM lotes 
         WHERE id_produto = ?1 
         ORDER BY validade"
    ).map_err(|e| e.to_string())?;
    
    let lotes = stmt.query_map([produto_id], |row| {
        Ok(Lote {
            id: row.get(0)?,
            id_produto: row.get(1)?,
            validade: row.get(2)?,
            quantidade_total: row.get(3)?,
            quantidade_prateleira: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut resultado = Vec::new();
    for lote in lotes {
        resultado.push(lote.map_err(|e| e.to_string())?);
    }
    
    Ok(resultado)
}

#[tauri::command]
fn deletar_lote(id: i32) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    conn.execute(
        "DELETE FROM lotes WHERE id = ?1",
        [id],
    ).map_err(|e| e.to_string())?;
    
    Ok("Lote deletado com sucesso".to_string())
}

// ===========================================
// FUNÇÕES DE NEGÓCIO (VENDA E REPOSIÇÃO)
// ===========================================

#[tauri::command]
fn vender_lote(id: i32, quantidade: i32) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT quantidade_prateleira, quantidade_total FROM lotes WHERE id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let (na_prateleira, total): (i32, i32) = stmt.query_row([id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?;
    
    if quantidade > na_prateleira {
        return Err(format!("Só há {} unidades na prateleira", na_prateleira));
    }
    
    let nova_prateleira = na_prateleira - quantidade;
    
    conn.execute(
        "UPDATE lotes SET quantidade_prateleira = ?1 WHERE id = ?2",
        params![nova_prateleira, id],
    ).map_err(|e| e.to_string())?;
    
    Ok(format!("Venda registrada: {} unidades", quantidade))
}

#[tauri::command]
fn abastecer_prateleira(id: i32, quantidade: i32) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT quantidade_prateleira, quantidade_total FROM lotes WHERE id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let (na_prateleira, total): (i32, i32) = stmt.query_row([id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?;
    
    if na_prateleira + quantidade > total {
        return Err("Não há estoque suficiente para abastecer".to_string());
    }
    
    let nova_prateleira = na_prateleira + quantidade;
    
    conn.execute(
        "UPDATE lotes SET quantidade_prateleira = ?1 WHERE id = ?2",
        params![nova_prateleira, id],
    ).map_err(|e| e.to_string())?;
    
    Ok(format!("Prateleira abastecida com {} unidades", quantidade))
}

// ===========================================
// PESQUISA
// ===========================================

#[tauri::command]
fn pesquisar_produtos(termo: String) -> Result<Vec<Produto>, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    let termo_busca = format!("%{}%", termo.to_uppercase());
    
    let mut stmt = conn.prepare(
        "SELECT id, nome, id_tipo FROM produtos WHERE UPPER(nome) LIKE ?1 ORDER BY nome"
    ).map_err(|e| e.to_string())?;
    
    let produtos = stmt.query_map([termo_busca], |row| {
        Ok(Produto {
            id: row.get(0)?,
            nome: row.get(1)?,
            id_tipo: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut resultado = Vec::new();
    for produto in produtos {
        resultado.push(produto.map_err(|e| e.to_string())?);
    }
    
    Ok(resultado)
}

// ===========================================
// ATUALIZAR PRATELEIRA
// ===========================================

#[tauri::command]
fn atualizar_prateleira(lote_id: i32, quantidade_prateleira: i32) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    conn.execute(
        "UPDATE lotes SET quantidade_prateleira = ?1 WHERE id = ?2",
        params![quantidade_prateleira, lote_id],
    ).map_err(|e| e.to_string())?;
    
    Ok("Prateleira atualizada".to_string())
}

// ===========================================
// FUNÇÃO DE RELATÓRIO COMPLETO
// ===========================================

#[tauri::command]
fn relatorio_completo() -> Result<Vec<RelatorioItem>, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    // Busca todas as seções com seus totais
    let mut stmt = conn.prepare(
        "SELECT s.id, s.nome, 
                COALESCE(SUM(l.quantidade_total), 0) as total,
                COALESCE(SUM(l.quantidade_prateleira), 0) as prateleira
         FROM secoes s
         LEFT JOIN tipos t ON s.id = t.id_secao
         LEFT JOIN produtos p ON t.id = p.id_tipo
         LEFT JOIN lotes l ON p.id = l.id_produto
         GROUP BY s.id"
    ).map_err(|e| e.to_string())?;
    
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
    }).map_err(|e| e.to_string())?;
    
    for secao in secoes {
        let mut secao_item = secao.map_err(|e| e.to_string())?;
        
        // Busca os tipos desta seção
        let mut stmt_tipos = conn.prepare(
            "SELECT t.id, t.nome,
                    COALESCE(SUM(l.quantidade_total), 0) as total,
                    COALESCE(SUM(l.quantidade_prateleira), 0) as prateleira
             FROM tipos t
             LEFT JOIN produtos p ON t.id = p.id_tipo
             LEFT JOIN lotes l ON p.id = l.id_produto
             WHERE t.id_secao = ?1
             GROUP BY t.id"
        ).map_err(|e| e.to_string())?;
        
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
        }).map_err(|e| e.to_string())?;
        
        for tipo in tipos {
            let mut tipo_item = tipo.map_err(|e| e.to_string())?;
            
            // Busca os produtos deste tipo
            let mut stmt_produtos = conn.prepare(
                "SELECT p.id, p.nome,
                        COALESCE(SUM(l.quantidade_total), 0) as total,
                        COALESCE(SUM(l.quantidade_prateleira), 0) as prateleira
                 FROM produtos p
                 LEFT JOIN lotes l ON p.id = l.id_produto
                 WHERE p.id_tipo = ?1
                 GROUP BY p.id"
            ).map_err(|e| e.to_string())?;
            
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
            }).map_err(|e| e.to_string())?;
            
            for produto in produtos {
                let mut produto_item = produto.map_err(|e| e.to_string())?;
                
                // Busca os lotes deste produto
                let mut stmt_lotes = conn.prepare(
                    "SELECT l.id, l.validade, l.quantidade_total, l.quantidade_prateleira
                     FROM lotes l
                     WHERE l.id_produto = ?1"
                ).map_err(|e| e.to_string())?;
                
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
                }).map_err(|e| e.to_string())?;
                
                for lote in lotes {
                    produto_item.filhos.push(lote.map_err(|e| e.to_string())?);
                }
                
                tipo_item.filhos.push(produto_item);
            }
            
            secao_item.filhos.push(tipo_item);
        }
        
        resultado.push(secao_item);
    }
    
    Ok(resultado)
}

// ===========================================
// EXPORTAR CSV
// ===========================================

#[tauri::command]
fn exportar_csv() -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT s.nome, t.nome, p.nome, l.validade, l.quantidade_total, l.quantidade_prateleira
         FROM secoes s
         JOIN tipos t ON s.id = t.id_secao
         JOIN produtos p ON t.id = p.id_tipo
         JOIN lotes l ON p.id = l.id_produto
         ORDER BY s.nome, t.nome, p.nome"
    ).map_err(|e| e.to_string())?;
    
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
    }).map_err(|e| e.to_string())?;
    
    for linha in linhas {
        csv += &linha.map_err(|e| e.to_string())?;
    }
    
    Ok(csv)
}

// ===========================================
// IMPORTAR CSV (SUBSTITUI TODOS OS DADOS)
// ===========================================

#[tauri::command]
fn importar_csv(csv_data: String) -> Result<String, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    // 🔥 PASSO 1: APAGA TUDO (em ordem reversa por causa das chaves estrangeiras)
    conn.execute("DELETE FROM lotes", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM produtos", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tipos", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM secoes", []).map_err(|e| e.to_string())?;
    
    println!(">>> Dados antigos removidos. Iniciando importação...");
    
    // 🔥 PASSO 2: IMPORTA OS NOVOS DADOS
    let mut linhas_importadas = 0;
    let mut erros = 0;
    
    for (i, line) in csv_data.lines().enumerate() {
        // Pula o cabeçalho (primeira linha)
        if i == 0 {
            if !line.contains("Seção") && !line.contains("Secao") {
                return Err("Arquivo CSV inválido: cabeçalho não encontrado".to_string());
            }
            continue;
        }
        
        let line = line.trim();
        if line.is_empty() { continue; }
        
        // Divide a linha (formato esperado: Seção,Tipo,Produto,Validade,Total,Prateleira)
        let cols: Vec<&str> = line.split(',').collect();
        if cols.len() < 6 {
            println!(">>> Linha {} ignorada: formato inválido", i+1);
            erros += 1;
            continue;
        }
        
        // Limpa os campos (remove espaços extras)
        let secao_nome = cols[0].trim();
        let tipo_nome = cols[1].trim();
        let produto_nome = cols[2].trim().to_uppercase();
        let validade = cols[3].trim();
        let quantidade_total = cols[4].trim().parse::<i32>().unwrap_or(0);
        let quantidade_prateleira = cols[5].trim().parse::<i32>().unwrap_or(0);
        
        if secao_nome.is_empty() || tipo_nome.is_empty() || produto_nome.is_empty() || validade.is_empty() {
            println!(">>> Linha {} ignorada: campos vazios", i+1);
            erros += 1;
            continue;
        }
        
        // 1. Insere seção
        conn.execute(
            "INSERT INTO secoes (nome) VALUES (?1)",
            [secao_nome],
        ).map_err(|e| e.to_string())?;
        
        // Pega id da seção
        let secao_id: i32 = conn.query_row(
            "SELECT id FROM secoes WHERE nome = ?1",
            [secao_nome],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;
        
        // 2. Insere tipo
        conn.execute(
            "INSERT INTO tipos (nome, id_secao) VALUES (?1, ?2)",
            params![tipo_nome, secao_id],
        ).map_err(|e| e.to_string())?;
        
        // Pega id do tipo
        let tipo_id: i32 = conn.query_row(
            "SELECT id FROM tipos WHERE nome = ?1 AND id_secao = ?2",
            params![tipo_nome, secao_id],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;
        
        // 3. Insere produto
        conn.execute(
            "INSERT INTO produtos (nome, id_tipo) VALUES (?1, ?2)",
            params![produto_nome, tipo_id],
        ).map_err(|e| e.to_string())?;
        
        // Pega id do produto
        let produto_id: i32 = conn.query_row(
            "SELECT id FROM produtos WHERE nome = ?1 AND id_tipo = ?2",
            params![produto_nome, tipo_id],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;
        
        // 4. Insere lote
        conn.execute(
            "INSERT INTO lotes (id_produto, validade, quantidade_total, quantidade_prateleira) 
             VALUES (?1, ?2, ?3, ?4)",
            params![produto_id, validade, quantidade_total, quantidade_prateleira],
        ).map_err(|e| e.to_string())?;
        
        linhas_importadas += 1;
    }
    
    Ok(format!(
        "✅ Importação concluída! {} lotes importados. {} erros ignorados.", 
        linhas_importadas, erros
    ))
}

#[tauri::command]
fn produtos_a_vencer(dias: i32) -> Result<Vec<Lote>, String> {
    let conn = init_produtos_db().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT l.id, l.id_produto, l.validade, l.quantidade_total, l.quantidade_prateleira
         FROM lotes l
         WHERE julianday(l.validade) - julianday('now') <= ?1"
    ).map_err(|e| e.to_string())?;
    
    let lotes = stmt.query_map([dias], |row| {
        Ok(Lote {
            id: row.get(0)?,
            id_produto: row.get(1)?,
            validade: row.get(2)?,
            quantidade_total: row.get(3)?,
            quantidade_prateleira: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut resultado = Vec::new();
    for lote in lotes {
        resultado.push(lote.map_err(|e| e.to_string())?);
    }
    
    Ok(resultado)
}

// Enviar notificaçoes
#[tauri::command]
fn enviar_notificacao(titulo: String, mensagem: String) -> Result<String, String> {
    use tauri::Manager;
    
    // Obtém a janela principal e envia notificação
    // Nota: Em produção, você precisaria do AppHandle
    
    Ok("Notificação enviada".to_string())
}
// ===========================================
// RUN (INVOKE HANDLER COMPLETO)
// ===========================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())  // ← ADICIONE ESTA LINHA
        .invoke_handler(tauri::generate_handler![
            // Login / Config
            config,
            verifique,
            
            // Função legado
            cadastrar,
            
            // Seções
            criar_secao,
            listar_secoes,
            deletar_secao,
            
            // Tipos
            criar_tipo,
            listar_tipos,
            deletar_tipo,
            
            // Produtos
            criar_produto,
            listar_produtos,
            deletar_produto,
            
            // Lotes
            criar_lote,
            listar_lotes,
            deletar_lote,
            
            // Negócio
            vender_lote,
            abastecer_prateleira,
            
            // Pesquisa
            pesquisar_produtos,

            // Atualizar prateleira
            atualizar_prateleira,
            
            // Relatório
            relatorio_completo,

            // Planilhas
            exportar_csv,
            importar_csv,

            // Validade painel
            produtos_a_vencer,

            // Notificaçoes
            enviar_notificacao
])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}