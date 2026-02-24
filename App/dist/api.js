// ===========================================
// COOKIES HELPER
// ===========================================
const Cookies = {
  set(nome, valor, dias = 365) {
    const data = new Date();
    data.setTime(data.getTime() + dias * 24 * 60 * 60 * 1000);
    document.cookie = `${nome}=${valor}; expires=${data.toUTCString()}; path=/`;
  },

  get(nome) {
    const cookies = document.cookie.split("; ");
    for (let cookie of cookies) {
      const [key, value] = cookie.split("=");
      if (key === nome) return value;
    }
    return null;
  },

  remove(nome) {
    this.set(nome, "", -1);
  },
};

// ===========================================
// API HELPER - VERSÃO CORRIGIDA COM ID
// ===========================================
window.API = {
  // ===========================================
  // REQUISIÇÃO BASE
  // ===========================================
  async request(rota, dados = null, metodo = null) {
    const options = {
      method: metodo || (dados ? "POST" : "GET"),
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      },
    };

    if (dados && options.method !== "GET") {
      const params = new URLSearchParams();

      if (typeof dados === "object" && !Array.isArray(dados)) {
        for (let [key, value] of Object.entries(dados)) {
          params.append(key, value);
        }
      } else if (typeof dados === "string") {
        params.append("nome", dados);
      }

      options.body = params.toString();
    }

    try {
      const response = await fetch(`/api/${rota}`, options);
      const text = await response.text();

      try {
        return JSON.parse(text);
      } catch {
        return text;
      }
    } catch (error) {
      console.error(`❌ Erro na requisição para /api/${rota}:`, error);
      throw error;
    }
  },

  // ===========================================
  // LOGIN
  // ===========================================
  verifique: function (nome, password) {
    return this.request("login", { nome, password });
  },

  // ===========================================
  // SEÇÕES (CORRIGIDO - com ID)
  // ===========================================
  listar_secoes: function () {
    return this.request("secoes");
  },

  criar_secao: function (nome) {
    // Backend exige campo 'id'
    return this.request("secoes", { id: 0, nome: nome });
  },

  deletar_secao: function (id) {
    return this.request(`secoes/${id}`, null, "DELETE");
  },

  // ===========================================
  // TIPOS (CORRIGIDO - com ID)
  // ===========================================
  listar_tipos: function (secaoId) {
    return this.request(`tipos/secao/${secaoId}`);
  },

  criar_tipo: function (nome, secaoId) {
    // Backend exige campo 'id'
    return this.request("tipos", { id: 0, nome: nome, id_secao: secaoId });
  },

  deletar_tipo: function (id) {
    return this.request(`tipos/${id}`, null, "DELETE");
  },

  // ===========================================
  // PRODUTOS (CORRIGIDO - com ID)
  // ===========================================
  listar_produtos: function (tipoId) {
    return this.request(`produtos/tipo/${tipoId}`);
  },

  criar_produto: function (nome, tipoId) {
    return this.request("produtos", { 
        id: 0, 
        nome: nome, 
        tipo_id: tipoId  // ← tentando os dois formatos
    });
},

  deletar_produto: function (id) {
    return this.request(`produtos/${id}`, null, "DELETE");
  },

  // ===========================================
  // LOTES (CORRIGIDO - com ID)
  // ===========================================
  listar_lotes: function (produtoId) {
    return this.request(`lotes/produto/${produtoId}`);
  },

  criar_lote: function (dados) {
    return this.request("lotes", {
        id: 0,
        produto_id: dados.produto_id || dados.produtoId,
        validade: dados.validade,
        quantidade_total: dados.quantidade_total || dados.quantidadeTotal,
        quantidade_prateleira: dados.quantidade_prateleira || dados.quantidadePrateleira,
    });
},

  deletar_lote: function (id) {
    return this.request(`lotes/${id}`, null, "DELETE");
  },

  atualizar_prateleira: function (loteId, quantidade) {
    return this.request("atualizar_prateleira", {
      lote_id: loteId,
      quantidade_prateleira: quantidade,
    });
  },

  // ===========================================
  // OPERAÇÕES
  // ===========================================
  vender_lote: function (id, quantidade) {
    return this.request(`vender/${id}`, { quantidade });
  },

  abastecer_prateleira: function (id, quantidade) {
    return this.request(`abastecer/${id}`, { quantidade });
  },

  // ===========================================
  // RELATÓRIOS
  // ===========================================
  relatorio_completo: function () {
    return this.request("relatorio");
  },

  produtos_a_vencer: function (dias) {
    return this.request(`vencer/${dias}`);
  },

  // ===========================================
  // CSV
  // ===========================================
  exportar_csv: function () {
    return fetch("/api/exportar").then((r) => r.text());
  },

  importar_csv: function (csvData) {
    return this.request("importar", { csvData });
  },

  // ===========================================
  // TEMA POR DISPOSITIVO
  // ===========================================
  salvar_tema_dispositivo: async function (tema) {
    let dispositivoId = Cookies.get("dispositivo_id");
    if (!dispositivoId) {
      dispositivoId = "dev_" + Math.random().toString(36).substring(2, 15);
      Cookies.set("dispositivo_id", dispositivoId);
    }
    return this.request("salvar_tema", { dispositivo_id: dispositivoId, tema });
  },

  carregar_tema_dispositivo: async function () {
    const dispositivoId = Cookies.get("dispositivo_id");
    if (!dispositivoId) return "escuro";

    try {
      return await this.request("carregar_tema", {
        dispositivo_id: dispositivoId,
      });
    } catch {
      return "escuro";
    }
  },
};

// ===========================================
// COMPATIBILIDADE
// ===========================================
const API = window.API;

console.log("✅ API Helper carregado com sucesso!");
console.log("📡 API disponível:", Object.keys(window.API));
