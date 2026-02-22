const { invoke } = window.__TAURI__.core;

// ===========================================
// DETECTA QUAL PÁGINA ESTÁ CARREGADA
// ===========================================
const isLoginPage = document.getElementById("login-area") !== null;

// ===========================================
// ========== PÁGINA DE LOGIN (index.html) ==========
// ===========================================
if (isLoginPage) {
  console.log(">>> PÁGINA DE LOGIN");

  let user;
  let password;

  // ===========================================
  // LOGIN
  // ===========================================
  async function verificarLogin() {
    console.log("1. Verificando credenciais...");

    const valido = await invoke("verifique", {
      nome: user.value,
      password: password.value,
    });

    if (valido) {
      console.log("2. Login válido, redirecionando...");
      window.location.href = "menu.html";
    } else {
      console.log("2. Login inválido");
      alert("Usuário ou senha inválidos");
    }
  }

  // ===========================================
  // INICIALIZAÇÃO
  // ===========================================
  window.addEventListener("DOMContentLoaded", async () => {
    user = document.getElementById("user");
    password = document.getElementById("password");
    const btn = document.getElementById("submit-button");

    if (btn) {
      await invoke("config");
      console.log(">>> Banco de usuários pronto");
      btn.disabled = false;
      btn.addEventListener("click", verificarLogin);
    }
  });
}

// ===========================================
// NOTA: O CÓDIGO DO MENU FOI REMOVIDO
// Cada seção agora tem seu próprio HTML
// ===========================================
