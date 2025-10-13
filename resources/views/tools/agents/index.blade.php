<x-tool-layout
    title="Visualizador de Teoría de Agentes"
    :breadcrumbs="[
        ['name' => 'Inicio', 'url' => route('home')],
        ['name' => 'Herramientas', 'url' => route('home')],
        ['name' => 'Teoría de Agentes', 'url' => null]
    ]"
    icon="conv"
>
    <x-slot:actions>
        <x-app-ui.secondary-button x-data="{ isLoading: false }">
            Volver
        </x-app-ui.secondary-button>
        <x-app-ui.button class="ml-3" x-data="{ isLoading: false }">
            Exportar
        </x-app-ui.button>
    </x-slot>
    {{-- Componente de pestañas --}}
    <div x-data="agentState()" x-init="init()">
    <x-tabbed-tool :tabs="['Configuración del Entorno', 'Visualización del Grafo', 'Búsqueda en Amplitud (BFS)', 'Búsqueda en Profundidad (DFS)']">

        {{-- Pestaña 0: Configuración del Entorno --}}
        <x-slot:tab-0>
            @include('tools.agents.partials.tab-0-environment')
        </x-slot:tab-0>

        {{-- Pestaña 1: Visualización del Grafo --}}
        <x-slot:tab-1>
            @include('tools.agents.partials.tab-1-graph')
        </x-slot:tab-1>

        {{-- Pestaña 2: Búsqueda en Amplitud (BFS) --}}
        <x-slot:tab-2>
            @include('tools.agents.partials.tab-2-bfs')
        </x-slot:tab-2>

        {{-- Pestaña 3: Búsqueda en Profundidad (DFS) --}}
        <x-slot:tab-3>
            @include('tools.agents.partials.tab-3-dfs')
        </x-slot:tab-3>

    </x-tabbed-tool>
    </div>

    {{-- Scripts para la herramienta --}}
    <x-slot:scripts>
        @vite('resources/js/agents/app.ts')
    </x-slot>
</x-tool-layout>