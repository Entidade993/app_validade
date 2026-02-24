// ===========================================
// TEMAS.JS - VERSÃO QUE NÃO INTERFERE EM NADA
// ===========================================
(function() {
    console.log('🔥 TEMAS.JS CARREGADO');
    
    // ===========================================
    // CORES DE CADA TEMA (usado para fallback)
    // ===========================================
    const cores = {
        escuro: { bg: '#000', texto: '#fff', tema: '#0ff' },
        claro: { bg: '#f5f5f5', texto: '#222', tema: '#00aaff' },
        matrix: { bg: '#0a0f0a', texto: '#0f0', tema: '#0f0' }
    };

    // ===========================================
    // FUNÇÃO PRINCIPAL - SÓ MUDA A CLASSE
    // ===========================================
    function aplicarTema(tema) {
        console.log('🎨 Aplicando tema:', tema);
        
        // Remove todas as classes de tema
        document.body.classList.remove('tema-escuro', 'tema-claro', 'tema-matrix');
        
        // Adiciona a nova classe
        document.body.classList.add(`tema-${tema}`);
        
        // Salva no localStorage
        localStorage.setItem('temaGlobal', tema);
        
        // Dispara evento para outras páginas
        window.dispatchEvent(new CustomEvent('temaAlterado', { detail: { tema } }));
    }

    // ===========================================
    // FUNÇÃO PARA CARREGAR TEMA SALVO
    // ===========================================
    function carregarTema() {
        let tema = localStorage.getItem('temaGlobal');
        if (!tema || !['escuro', 'claro', 'matrix'].includes(tema)) {
            tema = 'escuro';
        }
        aplicarTema(tema);
        return tema;
    }

    // ===========================================
    // EXPÕE FUNÇÕES GLOBAIS
    // ===========================================
    window.temas = {
        aplicar: aplicarTema,
        carregar: carregarTema,
        getTemaAtual: () => localStorage.getItem('temaGlobal') || 'escuro'
    };

    // ===========================================
    // APLICA TEMA QUANDO A PÁGINA CARREGA
    // ===========================================
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', carregarTema);
    } else {
        carregarTema();
    }

    console.log('✅ TEMAS.JS PRONTO - Tema atual:', localStorage.getItem('temaGlobal') || 'escuro');
})();