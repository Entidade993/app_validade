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
  // LOGIN USANDO API (não Tauri)
  // ===========================================
  async function verificarLogin() {
    console.log("1. Verificando credenciais...");

    try {
      const valido = await API.verifique(user.value, password.value);

      if (valido) {
        console.log("2. Login válido, redirecionando...");
        window.location.href = "menu.html";
      } else {
        console.log("2. Login inválido");
        alert("Usuário ou senha inválidos");
      }
    } catch (error) {
      console.error("Erro no login:", error);
      alert("Erro de conexão: " + error);
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
      btn.disabled = false;
      btn.addEventListener("click", verificarLogin);
    }
  });
}