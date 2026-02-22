// temas.js - Gerenciamento centralizado de temas

const Temas = {
    // Temas disponíveis
    temas: {
        escuro: {
            nome: 'Escuro',
            cor: '#0ff',
            bg: '#000'
        },
        claro: {
            nome: 'Claro',
            cor: '#00aaff',
            bg: '#f5f5f5'
        },
        matrix: {
            nome: 'Matrix',
            cor: '#0f0',
            bg: '#0a0f0a'
        }
    },

    // Aplicar tema
    aplicar(tema) {
        // Remove todas as classes de tema
        document.body.classList.remove('tema-escuro', 'tema-claro', 'tema-matrix');
        
        // Adiciona a classe do tema selecionado
        if (tema) {
            document.body.classList.add(`tema-${tema}`);
            localStorage.setItem('temaGlobal', tema);
            
            // Dispara evento personalizado para outras páginas
            window.dispatchEvent(new CustomEvent('temaAlterado', { 
                detail: { tema: tema } 
            }));
        }
    },

    // Carregar tema salvo
    carregar() {
        const temaSalvo = localStorage.getItem('temaGlobal') || 'escuro';
        this.aplicar(temaSalvo);
        return temaSalvo;
    },

    // Obter tema atual
    getTemaAtual() {
        return localStorage.getItem('temaGlobal') || 'escuro';
    }
};

// Exporta para uso global
window.temas = Temas;

// Carrega o tema automaticamente quando o script é incluído
document.addEventListener('DOMContentLoaded', () => {
    Temas.carregar();
});