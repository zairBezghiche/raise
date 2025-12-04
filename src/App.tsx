// src/App.tsx

import './styles/variables.css'
import './styles/globals.css'

// 1. Ajouter l'import du Chat
import { ChatInterface } from '@/components/ai-chat/ChatInterface'
import { JsonDbTester } from '@/components/JsonDbTester'

export default function App() {
  return (
    <main className="container" style={{ height: '100vh', display: 'flex', flexDirection: 'column' }}>
      <header style={{ marginBottom: 16, textAlign: 'center' }}>
        <h1 className="text-primary">GenAptitude</h1>
        <p className="text-gray">Plateforme d'Ingénierie Multi-Domaines</p>
      </header>

      {/* 2. Créer une grille pour afficher Chat + DB Tester côte à côte */}
      <div style={{ 
        display: 'grid', 
        gridTemplateColumns: '1fr 1fr', 
        gap: '20px', 
        flex: 1, 
        minHeight: 0, 
        paddingBottom: '20px' 
      }}>
        
        {/* Zone de Chat à gauche */}
        <ChatInterface />
        
        {/* Zone de Debug DB à droite */}
        <JsonDbTester />
        
      </div>
      
      <div style={{ marginTop: 'auto', padding: '10px 0' }}>
        <p>Liens utiles :</p>
        <ul>
            <li><a href="/pages/dark-mode-demo.html">Demo Mode Dark</a></li>
            <li><a href="/pages/charte-graphique.html">Charte Graphique</a></li>
        </ul>
      </div>
    </main>
  )
}